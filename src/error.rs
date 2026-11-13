use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Forbidden(String),
    Internal(String),
    TooManyRequests { message: String, retry_after: u64 },
    Timeout(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal Error: {}", msg),
            AppError::TooManyRequests { message, retry_after } => {
                write!(f, "Too Many Requests (retry after {}s): {}", retry_after, message)
            }
            AppError::Timeout(msg) => write!(f, "Timeout: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))).into_response(),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, Json(json!({ "error": msg }))).into_response(),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, Json(json!({ "error": msg }))).into_response(),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": msg }))).into_response(),
            AppError::TooManyRequests { message, retry_after } => (
                StatusCode::TOO_MANY_REQUESTS,
                [(axum::http::header::RETRY_AFTER, retry_after.to_string())],
                Json(json!({
                    "error": "Too Many Requests",
                    "message": message,
                    "retry_after": retry_after
                })),
            )
                .into_response(),
            AppError::Timeout(msg) => (
                StatusCode::REQUEST_TIMEOUT,
                Json(json!({
                    "error": "Execution Timeout",
                    "message": msg
                })),
            )
                .into_response(),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::BadRequest(format!("Invalid JSON: {}", err))
    }
}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Internal(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::Internal(msg.to_string())
    }
}
