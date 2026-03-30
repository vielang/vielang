use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Khớp với bảng `asset`.
/// Java: org.thingsboard.server.common.data.asset.Asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub customer_id: Option<Uuid>,

    pub name: String,
    pub asset_type: String,
    pub label: Option<String>,
    pub asset_profile_id: Uuid,

    pub external_id: Option<Uuid>,
    pub additional_info: Option<serde_json::Value>,
    pub version: i64,
}

impl Asset {
    pub fn owner_id(&self) -> Uuid {
        self.customer_id.unwrap_or(self.tenant_id)
    }
}

/// Asset with profile name and customer title (for info endpoints).
/// Java: org.thingsboard.server.common.data.asset.AssetInfo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfoView {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub name: String,
    pub asset_type: String,
    pub label: Option<String>,
    pub asset_profile_id: Uuid,
    pub asset_profile_name: String,
    pub customer_title: Option<String>,
}
