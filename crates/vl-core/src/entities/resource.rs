use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Khớp với bảng `resource` (metadata + data).
/// Java: TbResource / TbResourceInfo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TbResource {
    pub id: Uuid,
    pub created_time: i64,
    /// NULL = system resource
    pub tenant_id: Option<Uuid>,
    pub title: String,
    /// IMAGE | JS_MODULE | JKS | PKCS_12 | LWM2M_MODEL | DASHBOARD | GENERAL
    pub resource_type: String,
    pub resource_sub_type: Option<String>,
    pub resource_key: String,
    pub file_name: String,
    pub is_public: bool,
    pub public_resource_key: Option<String>,
    pub etag: Option<String>,
    pub descriptor: Option<serde_json::Value>,
    /// Binary data (stored in DB, not always returned in list)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<u8>>,
    /// Thumbnail/preview binary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<Vec<u8>>,
    pub external_id: Option<Uuid>,
    pub version: i64,
}
