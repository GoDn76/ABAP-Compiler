use crate::compiler::diagnostics::Diagnostic;
use crate::compiler::ir::ProgramIR;
use crate::compiler::policy::ReleasedApiManifest;
use crate::compiler::CompileStats;
use crate::platform::types::{CdsView, LocalField, LocalObject, RapOperation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct LimitsConfig {
    #[serde(rename = "maxBodyBytes")]
    pub max_body_bytes: usize,
    #[serde(rename = "maxSteps")]
    pub max_steps: usize,
}

#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub version: String,
    pub manifest: ReleasedApiManifest,
    pub limits: LimitsConfig,
    #[serde(rename = "localOnly")]
    pub local_only: bool,
}

#[derive(Debug, Serialize)]
pub struct ExecutedFile {
    pub name: String,
    pub size: u64,
    pub modified: String,
}

#[derive(Debug, Serialize)]
pub struct ExecutedListResponse {
    pub files: Vec<ExecutedFile>,
}

#[derive(Debug, Serialize)]
pub struct CodeFileResponse {
    pub name: String,
    pub source: String,
}

#[derive(Debug, Serialize)]
pub struct ExamplesListResponse {
    pub files: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CompilePayloadRequest {
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(rename = "maxSteps", default)]
    pub max_steps: Option<usize>,
    #[serde(rename = "parameterValues", default)]
    pub parameter_values: Option<HashMap<String, Value>>,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub event: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RuntimeStats {
    pub steps: usize,
    #[serde(rename = "elapsedMs")]
    pub elapsed_ms: u64,
    #[serde(rename = "selectedRows")]
    pub selected_rows: usize,
}

#[derive(Debug, Serialize)]
pub struct ExecuteResponse {
    pub ok: bool,
    pub mode: String,
    #[serde(rename = "savedFile")]
    pub saved_file: String,
    pub diagnostics: Vec<Diagnostic>,
    #[serde(rename = "diagnosticText")]
    pub diagnostic_text: Vec<String>,
    pub output: Vec<String>,
    pub variables: HashMap<String, Value>,
    pub ir: Option<ProgramIR>,
    pub stats: CompileStats,
    pub runtime: Option<RuntimeStats>,
    #[serde(rename = "manifestCounts")]
    pub manifest_counts: HashMap<String, usize>,
}

#[derive(Debug, Serialize)]
pub struct SelectionPreviewResponse {
    pub ok: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub variables: HashMap<String, Value>,
    pub output: Vec<String>,
    pub stats: CompileStats,
}

#[derive(Debug, Serialize)]
pub struct GuiTablesResponse {
    pub tables: HashMap<String, Vec<Value>>,
    pub source: String,
    #[serde(rename = "readOnly")]
    pub read_only: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateObjectRequest {
    pub name: String,
    pub kind: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "baseType")]
    pub base_type: Option<String>,
    pub fields: Option<Vec<LocalField>>,
    #[serde(rename = "initialRows")]
    pub initial_rows: Option<Vec<Value>>,
}

#[derive(Debug, Serialize)]
pub struct CreateObjectResponse {
    pub ok: bool,
    pub object: LocalObject,
    pub reported: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteObjectResponse {
    pub ok: bool,
    pub object: LocalObject,
    pub reported: String,
}

#[derive(Debug, Serialize)]
pub struct CdsViewsResponse {
    pub views: Vec<CdsView>,
}

#[derive(Debug, Deserialize)]
pub struct ActivateCdsRequest {
    pub source: String,
    #[serde(default)]
    pub replace: bool,
}

#[derive(Debug, Serialize)]
pub struct ActivateCdsResponse {
    pub ok: bool,
    pub view: CdsView,
    pub reported: String,
}

#[derive(Debug, Deserialize)]
pub struct QueryCdsRequest {
    pub name: String,
    #[serde(default)]
    pub filter: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize)]
pub struct QueryCdsResponse {
    pub rows: Vec<Value>,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct RunRequest {
    #[serde(alias = "source", default)]
    pub code: String,
    #[serde(rename = "db_url", default)]
    pub db_url: Option<String>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(rename = "maxSteps", default)]
    pub max_steps: Option<usize>,
    #[serde(rename = "parameterValues", default)]
    pub parameter_values: Option<HashMap<String, Value>>,
    #[serde(default)]
    pub filename: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RunResponse {
    pub ok: bool,
    pub mode: String,
    pub output: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
    #[serde(rename = "diagnosticText")]
    pub diagnostic_text: Vec<String>,
    pub variables: HashMap<String, Value>,
    pub ir: Option<ProgramIR>,
    pub stats: CompileStats,
    pub runtime: Option<RuntimeStats>,
    #[serde(rename = "databaseConnected")]
    pub database_connected: bool,
    #[serde(rename = "databaseType")]
    pub database_type: String,
    pub ephemeral: bool,
}

#[derive(Debug, Deserialize)]
pub struct RapReadRequest {
    #[serde(default = "default_entity")]
    pub entity: String,
    #[serde(rename = "sessionId", default = "default_session")]
    pub session_id: String,
    #[serde(default)]
    pub filter: Option<HashMap<String, Value>>,
}

fn default_entity() -> String {
    "ZI_PRODUCT".to_string()
}
fn default_session() -> String {
    "default".to_string()
}

#[derive(Debug, Serialize)]
pub struct RapReadResponse {
    pub rows: Vec<Value>,
    pub pending: Vec<RapOperation>,
}

#[derive(Debug, Deserialize)]
pub struct RapModifyRequest {
    #[serde(default = "default_entity")]
    pub entity: String,
    #[serde(rename = "sessionId", default = "default_session")]
    pub session_id: String,
    pub operation: String,
    #[serde(default)]
    pub data: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize)]
pub struct RapModifyResponse {
    pub ok: bool,
    pub staged: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapped: Option<Value>,
    pub reported: String,
    pub pending: Vec<RapOperation>,
}

#[derive(Debug, Deserialize)]
pub struct RapSessionRequest {
    #[serde(rename = "sessionId", default = "default_session")]
    pub session_id: String,
}

#[derive(Debug, Serialize)]
pub struct RapCommitResponse {
    pub ok: bool,
    pub committed: usize,
    pub rows: usize,
    pub reported: String,
}

#[derive(Debug, Serialize)]
pub struct RapRollbackResponse {
    pub ok: bool,
    #[serde(rename = "rolledBack")]
    pub rolled_back: usize,
    pub reported: String,
}
