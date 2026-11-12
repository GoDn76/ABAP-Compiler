use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<bool>,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalObject {
    pub name: String,
    pub kind: String, // 'TABL'|'DTEL'|'DOMA'|'STRU'|'VIEW'|'BDEF'
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<LocalField>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(rename = "baseType", skip_serializing_if = "Option::is_none")]
    pub base_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CdsField {
    pub source: String,
    pub alias: String,
    pub key: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CdsView {
    pub name: String,
    pub source: String,
    pub fields: Vec<CdsField>,
    pub annotations: Vec<String>,
    #[serde(rename = "sourceText")]
    pub source_text: String,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RapOperation {
    pub kind: String, // 'CREATE'|'UPDATE'|'DELETE'
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogResponse {
    pub objects: Vec<LocalObject>,
    pub tables: Vec<String>,
    #[serde(rename = "cdsViews")]
    pub cds_views: Vec<String>,
    #[serde(rename = "rapEntities")]
    pub rap_entities: Vec<String>,
    pub database: String,
    #[serde(rename = "isSapBackend")]
    pub is_sap_backend: bool,
}
