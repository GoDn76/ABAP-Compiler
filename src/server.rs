use crate::api::handlers::AppState;
use crate::api::api_routes;
use crate::config::AppConfig;
use crate::db::{PostgresClient, RedisSessionStore};
use crate::error::AppError;
use crate::platform::LocalPlatform;
use axum::{
    body::Body,
    extract::Request,
    http::{header, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;

pub fn create_router(state: AppState) -> Router {
    let app_dir = state.config.app_dir.clone();
    let max_body = state.config.max_body_bytes;

    let api = api_routes(state.clone());
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(api)
        .route("/sapgui", get(move || serve_named_file(app_dir.clone(), "gui.html")))
        .route("/transactions", get({
            let app_dir = state.config.app_dir.clone();
            move || serve_named_file(app_dir, "gui.html")
        }))
        .fallback(get(move |req: Request| serve_static_file(state.config.app_dir.clone(), req)))
        .layer(cors)
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(RequestBodyLimitLayer::new(max_body))
}

async fn security_headers_middleware(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
}

async fn serve_named_file(app_dir: String, name: &str) -> Response {
    let file_path = Path::new(&app_dir).join(name);
    match fs::read(&file_path).await {
        Ok(bytes) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            bytes,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}

async fn serve_static_file(app_dir: String, req: Request) -> Response {
    let path_str = req.uri().path();
    let clean = path_str.split('?').next().unwrap_or("/");
    let relative = if clean == "/" {
        "index.html"
    } else {
        clean.trim_start_matches('/')
    };

    let base_canonical = match fs::canonicalize(&app_dir).await {
        Ok(p) => p,
        Err(_) => return (StatusCode::NOT_FOUND, "Not found").into_response(),
    };

    let target_path = Path::new(&app_dir).join(relative);
    let target_canonical = match fs::canonicalize(&target_path).await {
        Ok(p) => p,
        Err(_) => return (StatusCode::NOT_FOUND, "Not found").into_response(),
    };

    // Prevent directory traversal attacks
    if !target_canonical.starts_with(&base_canonical) {
        return (StatusCode::FORBIDDEN, "Forbidden").into_response();
    }

    match fs::read(&target_canonical).await {
        Ok(bytes) => {
            let ext = target_canonical
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();

            let mime_type = match ext.as_str() {
                "html" => "text/html; charset=utf-8",
                "css" => "text/css; charset=utf-8",
                "js" => "text/javascript; charset=utf-8",
                "svg" => "image/svg+xml",
                "json" => "application/json; charset=utf-8",
                _ => "application/octet-stream",
            };

            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime_type)],
                bytes,
            )
                .into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}
