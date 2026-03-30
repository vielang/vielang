use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculatedField {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: String,
    pub name: String,
    pub expression: String,
    pub output_key: String,
    pub input_keys: Vec<String>,
    pub trigger_mode: String,
    pub output_ttl_ms: Option<i64>,
    pub enabled: bool,
    pub created_time: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCalculatedFieldRequest {
    pub entity_id: Uuid,
    pub entity_type: String,
    pub name: String,
    pub expression: String,
    pub output_key: String,
    pub input_keys: Vec<String>,
    pub trigger_mode: Option<String>,
    pub output_ttl_ms: Option<i64>,
    pub enabled: Option<bool>,
}
