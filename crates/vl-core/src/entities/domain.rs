use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DomainEntry {
    pub id:                Uuid,
    pub created_time:      i64,
    pub tenant_id:         Uuid,
    pub name:              String,
    pub oauth2_enabled:    bool,
    pub propagate_to_edge: bool,
}
