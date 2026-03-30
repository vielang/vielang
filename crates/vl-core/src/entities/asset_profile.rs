use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Khớp với bảng `asset_profile`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetProfile {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,

    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub is_default: bool,

    pub default_rule_chain_id: Option<Uuid>,
    pub default_dashboard_id: Option<Uuid>,
    pub default_queue_name: Option<String>,
    pub default_edge_rule_chain_id: Option<Uuid>,

    pub external_id: Option<Uuid>,
    pub version: i64,
}
