use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKey {
    pub id:           Uuid,
    pub created_time: i64,
    pub tenant_id:    Uuid,
    pub user_id:      Uuid,
    pub name:         String,
    /// SHA-256 hex of the raw key — never returned in responses
    #[serde(skip_serializing)]
    pub key_hash:     String,
    /// First 16 chars of raw key for display
    pub key_prefix:   String,
    pub scopes:       Vec<String>,
    pub expires_at:   Option<i64>,
    pub last_used_at: Option<i64>,
    pub enabled:      bool,
}
