use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiModel {
    pub id:              Uuid,
    pub created_time:    i64,
    pub tenant_id:       Option<Uuid>,
    pub name:            String,
    pub configuration:   Option<serde_json::Value>,
    pub additional_info: Option<serde_json::Value>,
}
