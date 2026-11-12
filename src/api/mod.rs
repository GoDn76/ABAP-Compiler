pub mod handlers;
pub mod types;

use axum::{
    routing::{get, post},
    Router,
};
use handlers::*;

pub fn api_routes(state: AppState) -> Router {
    Router::new()
        // System and file endpoints
        .route("/api/health", get(health_handler))
        .route("/api/config", get(config_handler))
        .route("/api/executed", get(executed_handler))
        .route("/api/code", get(code_handler))
        .route("/api/examples", get(examples_handler))
        .route("/api/example", get(example_handler))
        // Compilation and execution endpoints
        .route("/api/compile", post(compile_handler))
        .route("/api/execute", post(execute_handler))
        .route("/api/selection-preview", post(selection_preview_handler))
        .route("/api/selection-event", post(selection_event_handler))
        // Dictionary, GUI tables, and platform catalog
        .route("/api/gui/tables", get(gui_tables_handler))
        .route("/api/platform/catalog", get(platform_catalog_handler))
        .route("/api/platform/create-object", post(platform_create_object_handler))
        .route(
            "/api/platform/object",
            get(platform_get_object_handler).delete(platform_delete_object_handler),
        )
        // CDS endpoints
        .route("/api/cds/views", get(cds_views_handler))
        .route("/api/cds/activate", post(cds_activate_handler))
        .route("/api/cds/query", post(cds_query_handler))
        // RAP endpoints
        .route("/api/rap/read", post(rap_read_handler))
        .route("/api/rap/modify", post(rap_modify_handler))
        .route("/api/rap/commit", post(rap_commit_handler))
        .route("/api/rap/rollback", post(rap_rollback_handler))
        .route("/api/stop", post(stop_handler))
        .with_state(state)
}
