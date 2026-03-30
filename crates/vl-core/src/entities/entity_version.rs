use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityVersion {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: String,
    pub version_number: i64,
    pub commit_msg: Option<String>,
    pub snapshot: serde_json::Value,
    pub diff: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
    pub created_time: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitRequest {
    pub entity_id: Uuid,
    pub entity_type: String,
    pub snapshot: serde_json::Value,
    pub commit_msg: Option<String>,
}

/// Request type for version creation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VersionCreateRequestType {
    SingleEntity,
    Complex,
}

/// Sync strategy for version restore
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SyncStrategy {
    Overwrite,
    Merge,
}

/// Request to create a version (matches Java VersionCreateRequest)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionCreateRequest {
    pub request_type: VersionCreateRequestType,
    pub entity_id: Option<Uuid>,
    pub entity_type: Option<String>,
    pub version_name: String,
    pub description: Option<String>,
    /// For COMPLEX type: batch entity configs
    pub entity_types: Option<Vec<EntityTypeVersionConfig>>,
    pub sync_strategy: Option<SyncStrategy>,
}

/// Per-entity-type config for COMPLEX version requests
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityTypeVersionConfig {
    pub entity_type: String,
    pub save_relations: bool,
    pub save_attributes: bool,
    pub save_credentials: bool,
}

/// Status of an async version operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionRequestStatus {
    pub request_id: Uuid,
    pub done: bool,
    pub added: i64,
    pub modified: i64,
    pub removed: i64,
    pub error: Option<String>,
}

/// AutoCommit settings per tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoCommitSettings {
    pub enabled: bool,
    pub entity_types: Vec<String>,
}
