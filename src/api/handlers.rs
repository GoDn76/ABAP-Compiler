use super::types::*;
use crate::compiler::diagnostics::format_diagnostic;
use crate::compiler::ir::RequestedEvent;
use crate::compiler::policy::{LanguageMode, ReleasedApiManifest};
use crate::compiler::{compile_source, CompileOptions};
use crate::config::AppConfig;
use crate::db::{PostgresClient, RedisSessionStore};
use crate::error::AppError;
use crate::platform::LocalPlatform;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::Local;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub platform: Arc<RwLock<LocalPlatform>>,
    pub redis: RedisSessionStore,
    pub postgres: PostgresClient,
}

impl AppState {
    pub async fn save_platform_state(&self) {
        let platform = self.platform.read().await;
        let json_val = platform.export_state();
        if let Ok(content) = serde_json::to_string_pretty(&json_val) {
            let _ = fs::write(&self.config.platform_state_path, content).await;
        }
    }
}

pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        name: "ABAPC Studio".to_string(),
        version: "5.0.1".to_string(),
    })
}

pub async fn config_handler(State(state): State<AppState>) -> Json<ConfigResponse> {
    let manifest = read_manifest(&state.config.manifest_path).await;
    Json(ConfigResponse {
        version: state.config.version.clone(),
        manifest,
        limits: LimitsConfig {
            max_body_bytes: state.config.max_body_bytes,
            max_steps: state.config.max_steps,
        },
        local_only: true,
    })
}

async fn read_manifest(path: &str) -> ReleasedApiManifest {
    if let Ok(content) = fs::read_to_string(path).await {
        if let Ok(m) = serde_json::from_str::<ReleasedApiManifest>(&content) {
            return m;
        }
    }
    ReleasedApiManifest::default()
}

pub async fn executed_handler(State(state): State<AppState>) -> Result<Json<ExecutedListResponse>, AppError> {
    let dir = Path::new(&state.config.executed_dir);
    if !dir.exists() {
        let _ = fs::create_dir_all(dir).await;
    }
    let mut entries = fs::read_dir(dir).await?;
    let mut files = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.to_ascii_lowercase().ends_with(".abap") {
            if let Ok(metadata) = entry.metadata().await {
                if metadata.is_file() {
                    let modified: chrono::DateTime<Local> = metadata.modified()?.into();
                    files.push(ExecutedFile {
                        name,
                        size: metadata.len(),
                        modified: modified.to_rfc3339(),
                    });
                }
            }
        }
    }

    files.sort_by(|a, b| b.modified.cmp(&a.modified));
    files.truncate(300);
    Ok(Json(ExecutedListResponse { files }))
}

#[derive(serde::Deserialize)]
pub struct NameQuery {
    pub name: Option<String>,
}

pub async fn code_handler(
    State(state): State<AppState>,
    Query(query): Query<NameQuery>,
) -> Result<Json<CodeFileResponse>, AppError> {
    let name = query.name.ok_or_else(|| AppError::BadRequest("Invalid file name.".to_string()))?;
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(AppError::BadRequest("Invalid file name.".to_string()));
    }
    let path = Path::new(&state.config.executed_dir).join(&name);
    match fs::read_to_string(&path).await {
        Ok(source) => Ok(Json(CodeFileResponse { name, source })),
        Err(_) => Err(AppError::NotFound("Executed code file not found.".to_string())),
    }
}

pub async fn examples_handler(State(state): State<AppState>) -> Result<Json<ExamplesListResponse>, AppError> {
    let dir = Path::new(&state.config.examples_dir);
    let mut entries = fs::read_dir(dir).await?;
    let mut files = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.to_ascii_lowercase().ends_with(".abap") {
            if let Ok(metadata) = entry.metadata().await {
                if metadata.is_file() {
                    files.push(name);
                }
            }
        }
    }

    files.sort();
    Ok(Json(ExamplesListResponse { files }))
}

pub async fn example_handler(
    State(state): State<AppState>,
    Query(query): Query<NameQuery>,
) -> Result<Json<CodeFileResponse>, AppError> {
    let name = query.name.ok_or_else(|| AppError::BadRequest("Invalid example name.".to_string()))?;
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(AppError::BadRequest("Invalid example name.".to_string()));
    }
    let path = Path::new(&state.config.examples_dir).join(&name);
    match fs::read_to_string(&path).await {
        Ok(source) => Ok(Json(CodeFileResponse { name, source })),
        Err(_) => Err(AppError::NotFound("Example not found.".to_string())),
    }
}

fn safe_file_name(name: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' { c } else { '_' })
        .take(120)
        .collect();
    if clean.is_empty() {
        "program".to_string()
    } else {
        clean
    }
}

fn current_timestamp() -> String {
    Local::now().format("%Y-%m-%d_%H-%M-%S-%3f").to_string()
}

pub async fn compile_handler(
    State(state): State<AppState>,
    Json(payload): Json<CompilePayloadRequest>,
) -> Result<Json<crate::compiler::CompileResult>, AppError> {
    let manifest = read_manifest(&state.config.manifest_path).await;
    let mode = if payload.mode.as_deref() == Some("cloud") {
        LanguageMode::Cloud
    } else {
        LanguageMode::Standard
    };

    let ext_types = {
        let p = state.platform.read().await;
        p.get_compiler_types()
    };

    let result = compile_source(
        &payload.source,
        CompileOptions {
            mode: Some(mode),
            released_apis: Some(manifest),
            run: false,
            max_steps: payload.max_steps.unwrap_or(2_000_000),
            parameter_values: payload.parameter_values.unwrap_or_default(),
            preview_selection: false,
            selection_event: None,
            external_types: ext_types,
        },
    );
    Ok(Json(result))
}

pub async fn execute_handler(
    State(state): State<AppState>,
    Json(payload): Json<CompilePayloadRequest>,
) -> Result<Json<ExecuteResponse>, AppError> {
    let source = payload.source;
    let mode_str = if payload.mode.as_deref() == Some("cloud") {
        "cloud"
    } else {
        "standard"
    };

    let raw_filename = payload.filename.as_deref().unwrap_or("program.abap");
    let safe_name = safe_file_name(raw_filename);
    let final_name = if safe_name.ends_with(".abap") {
        safe_name
    } else {
        format!("{}.abap", safe_name)
    };

    let saved_name = format!("{}_{}_{}", current_timestamp(), mode_str, final_name);
    let exec_dir = Path::new(&state.config.executed_dir);
    let _ = fs::create_dir_all(exec_dir).await;
    let saved_path = exec_dir.join(&saved_name);
    let _ = fs::write(&saved_path, &source).await;

    let relative_path = format!("executed codes/{}", saved_name);

    let manifest = read_manifest(&state.config.manifest_path).await;
    let mode = if mode_str == "cloud" {
        LanguageMode::Cloud
    } else {
        LanguageMode::Standard
    };

    let ext_types = {
        let p = state.platform.read().await;
        p.get_compiler_types()
    };

    let max_steps = payload
        .max_steps
        .map(|s| s.clamp(1_000, 10_000_000))
        .unwrap_or(2_000_000);

    let result = compile_source(
        &source,
        CompileOptions {
            mode: Some(mode),
            released_apis: Some(manifest),
            run: true,
            max_steps,
            parameter_values: payload.parameter_values.unwrap_or_default(),
            preview_selection: false,
            selection_event: None,
            external_types: ext_types,
        },
    );

    let commit_re = Regex::new(r"(?i)\bCOMMIT\s+ENTITIES\b").unwrap();
    if result.stats.errors == 0 && commit_re.is_match(&source) {
        state.save_platform_state().await;
    }

    let d_text = result.diagnostics.iter().map(format_diagnostic).collect();
    let runtime = result.vm.as_ref().map(|vm| RuntimeStats {
        steps: vm.steps,
        elapsed_ms: vm.elapsed_ms,
        selected_rows: vm.selected_rows,
    });

    Ok(Json(ExecuteResponse {
        ok: result.stats.errors == 0,
        mode: mode_str.to_string(),
        saved_file: relative_path,
        diagnostics: result.diagnostics,
        diagnostic_text: d_text,
        output: result.vm.as_ref().map(|v| v.output.clone()).unwrap_or_default(),
        variables: result.vm.as_ref().map(|v| v.variables.clone()).unwrap_or_default(),
        ir: result.ir,
        stats: result.stats,
        runtime,
        manifest_counts: result.manifest_counts,
    }))
}

pub async fn selection_preview_handler(
    State(state): State<AppState>,
    Json(payload): Json<CompilePayloadRequest>,
) -> Result<Json<SelectionPreviewResponse>, AppError> {
    let manifest = read_manifest(&state.config.manifest_path).await;
    let mode = if payload.mode.as_deref() == Some("cloud") {
        LanguageMode::Cloud
    } else {
        LanguageMode::Standard
    };

    let ext_types = {
        let p = state.platform.read().await;
        p.get_compiler_types()
    };

    let result = compile_source(
        &payload.source,
        CompileOptions {
            mode: Some(mode),
            released_apis: Some(manifest),
            run: true,
            max_steps: payload.max_steps.unwrap_or(2_000_000),
            parameter_values: payload.parameter_values.unwrap_or_default(),
            preview_selection: true,
            selection_event: None,
            external_types: ext_types,
        },
    );

    Ok(Json(SelectionPreviewResponse {
        ok: result.stats.errors == 0,
        diagnostics: result.diagnostics,
        variables: result.vm.as_ref().map(|v| v.variables.clone()).unwrap_or_default(),
        output: result.vm.as_ref().map(|v| v.output.clone()).unwrap_or_default(),
        stats: result.stats,
    }))
}

pub async fn selection_event_handler(
    State(state): State<AppState>,
    Json(payload): Json<CompilePayloadRequest>,
) -> Result<Json<SelectionPreviewResponse>, AppError> {
    let event = payload.event.clone().unwrap_or_default();
    let target = payload.target.clone().unwrap_or_default();

    if !matches!(
        event.as_str(),
        "at-selection-screen-value-request" | "at-selection-screen-help-request"
    ) {
        return Err(AppError::BadRequest("Unsupported selection-screen event.".to_string()));
    }

    let target_re = Regex::new(r"(?i)^[A-Za-z_][A-Za-z0-9_]*(?:-(?:LOW|HIGH))?$").unwrap();
    if !target.is_empty() && !target_re.is_match(&target) {
        return Err(AppError::BadRequest("Invalid selection-screen target.".to_string()));
    }

    let manifest = read_manifest(&state.config.manifest_path).await;
    let mode = if payload.mode.as_deref() == Some("cloud") {
        LanguageMode::Cloud
    } else {
        LanguageMode::Standard
    };

    let ext_types = {
        let p = state.platform.read().await;
        p.get_compiler_types()
    };

    let result = compile_source(
        &payload.source,
        CompileOptions {
            mode: Some(mode),
            released_apis: Some(manifest),
            run: true,
            max_steps: payload.max_steps.unwrap_or(2_000_000),
            parameter_values: payload.parameter_values.unwrap_or_default(),
            preview_selection: false,
            selection_event: Some(RequestedEvent {
                event,
                target: if !target.is_empty() { Some(target) } else { None },
            }),
            external_types: ext_types,
        },
    );

    Ok(Json(SelectionPreviewResponse {
        ok: result.stats.errors == 0,
        diagnostics: result.diagnostics,
        variables: result.vm.as_ref().map(|v| v.variables.clone()).unwrap_or_default(),
        output: result.vm.as_ref().map(|v| v.output.clone()).unwrap_or_default(),
        stats: result.stats,
    }))
}

pub async fn gui_tables_handler(State(state): State<AppState>) -> Json<GuiTablesResponse> {
    let platform = state.platform.read().await;
    let mut tables = HashMap::new();
    for name in platform.list_tables() {
        if let Ok(rows) = platform.table_rows(&name) {
            tables.insert(name, rows);
        }
    }

    Json(GuiTablesResponse {
        tables,
        source: "ABAPC local relational fixtures".to_string(),
        read_only: true,
    })
}

pub async fn platform_catalog_handler(State(state): State<AppState>) -> Json<crate::platform::types::CatalogResponse> {
    let platform = state.platform.read().await;
    Json(platform.get_catalog())
}

pub async fn platform_create_object_handler(
    State(state): State<AppState>,
    Json(body): Json<CreateObjectRequest>,
) -> Result<(StatusCode, Json<CreateObjectResponse>), AppError> {
    let mut platform = state.platform.write().await;
    let obj = platform.create_object(
        &body.name,
        body.kind.as_deref().unwrap_or("TABL"),
        body.description.as_deref(),
        body.base_type.as_deref(),
        body.fields,
        body.initial_rows,
    )?;
    drop(platform);
    state.save_platform_state().await;

    Ok((
        StatusCode::CREATED,
        Json(CreateObjectResponse {
            ok: true,
            reported: format!("Local {} object {} created.", obj.kind, obj.name),
            object: obj,
        }),
    ))
}

pub async fn platform_get_object_handler(
    State(state): State<AppState>,
    Query(query): Query<NameQuery>,
) -> Result<Json<crate::platform::types::LocalObject>, AppError> {
    let name = query.name.unwrap_or_default();
    let platform = state.platform.read().await;
    let obj = platform
        .get_object(&name)
        .ok_or_else(|| AppError::NotFound(format!("Local repository object '{}' was not found.", name)))?;
    Ok(Json(obj))
}

pub async fn platform_delete_object_handler(
    State(state): State<AppState>,
    Query(query): Query<NameQuery>,
) -> Result<Json<DeleteObjectResponse>, AppError> {
    let name = query.name.unwrap_or_default();
    let mut platform = state.platform.write().await;
    let obj = platform.delete_object(&name)?;
    drop(platform);
    state.save_platform_state().await;

    Ok(Json(DeleteObjectResponse {
        ok: true,
        reported: format!("Local object {} deleted.", obj.name),
        object: obj,
    }))
}

pub async fn cds_views_handler(State(state): State<AppState>) -> Json<CdsViewsResponse> {
    let platform = state.platform.read().await;
    Json(CdsViewsResponse {
        views: platform.list_cds_views(),
    })
}

pub async fn cds_activate_handler(
    State(state): State<AppState>,
    Json(body): Json<ActivateCdsRequest>,
) -> Result<Json<ActivateCdsResponse>, AppError> {
    let mut platform = state.platform.write().await;
    let view = if body.replace {
        platform.replace_cds(&body.source)?
    } else {
        platform.activate_cds(&body.source)?
    };
    drop(platform);
    state.save_platform_state().await;

    Ok(Json(ActivateCdsResponse {
        ok: true,
        reported: format!("CDS view entity {} activated in the local repository.", view.name),
        view,
    }))
}

pub async fn cds_query_handler(
    State(state): State<AppState>,
    Json(body): Json<QueryCdsRequest>,
) -> Result<Json<QueryCdsResponse>, AppError> {
    let platform = state.platform.read().await;
    let rows = platform.query_cds(&body.name, &body.filter.unwrap_or_default())?;
    Ok(Json(QueryCdsResponse {
        rows,
        source: "ABAPC local relational fixture".to_string(),
    }))
}

pub async fn rap_read_handler(
    State(state): State<AppState>,
    Json(body): Json<RapReadRequest>,
) -> Result<Json<RapReadResponse>, AppError> {
    let sess_re = Regex::new(r"^[A-Za-z0-9_-]{1,80}$").unwrap();
    if !sess_re.is_match(&body.session_id) {
        return Err(AppError::BadRequest("Invalid training session ID.".to_string()));
    }
    let platform = state.platform.read().await;
    let rows = platform.rap_read(&body.entity, &body.session_id, &body.filter.unwrap_or_default())?;
    let pending = platform.rap_pending(&body.session_id);
    Ok(Json(RapReadResponse { rows, pending }))
}

pub async fn rap_modify_handler(
    State(state): State<AppState>,
    Json(body): Json<RapModifyRequest>,
) -> Result<Json<RapModifyResponse>, AppError> {
    let sess_re = Regex::new(r"^[A-Za-z0-9_-]{1,80}$").unwrap();
    if !sess_re.is_match(&body.session_id) {
        return Err(AppError::BadRequest("Invalid training session ID.".to_string()));
    }
    let op = body.operation.trim().to_ascii_uppercase();
    if !matches!(op.as_str(), "CREATE" | "UPDATE" | "DELETE") {
        return Err(AppError::BadRequest("Operation must be CREATE, UPDATE, or DELETE.".to_string()));
    }
    let mut platform = state.platform.write().await;
    let res = platform.rap_modify(&body.entity, &body.session_id, &op, &body.data.unwrap_or_default())?;
    let pending = platform.rap_pending(&body.session_id);

    Ok(Json(RapModifyResponse {
        ok: true,
        staged: res.staged,
        mapped: res.mapped,
        reported: res.reported,
        pending,
    }))
}

pub async fn rap_commit_handler(
    State(state): State<AppState>,
    Json(body): Json<RapSessionRequest>,
) -> Result<Json<RapCommitResponse>, AppError> {
    let sess_re = Regex::new(r"^[A-Za-z0-9_-]{1,80}$").unwrap();
    if !sess_re.is_match(&body.session_id) {
        return Err(AppError::BadRequest("Invalid training session ID.".to_string()));
    }
    let mut platform = state.platform.write().await;
    let res = platform.rap_commit(&body.session_id)?;
    drop(platform);
    state.save_platform_state().await;

    Ok(Json(RapCommitResponse {
        ok: true,
        committed: res.committed,
        rows: res.rows,
        reported: res.reported,
    }))
}

pub async fn rap_rollback_handler(
    State(state): State<AppState>,
    Json(body): Json<RapSessionRequest>,
) -> Result<Json<RapRollbackResponse>, AppError> {
    let sess_re = Regex::new(r"^[A-Za-z0-9_-]{1,80}$").unwrap();
    if !sess_re.is_match(&body.session_id) {
        return Err(AppError::BadRequest("Invalid training session ID.".to_string()));
    }
    let mut platform = state.platform.write().await;
    let res = platform.rap_rollback(&body.session_id);
    Ok(Json(RapRollbackResponse {
        ok: true,
        rolled_back: res.rolled_back,
        reported: res.reported,
    }))
}

pub async fn stop_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}
