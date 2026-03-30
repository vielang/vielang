use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Khớp với bảng `entity_view`.
/// Java: org.thingsboard.server.common.data.EntityView
///
/// EntityView là filtered read-only view của một Device hoặc Asset,
/// dùng để expose một tập con keys telemetry/attribute cho Customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityView {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub customer_id: Option<Uuid>,

    /// ID của entity gốc (Device, Asset...)
    pub entity_id: Uuid,
    /// Loại entity gốc: "DEVICE", "ASSET"
    pub entity_type: String,

    pub name: String,
    /// Phân loại view, ví dụ "Temperature Sensor"
    pub entity_view_type: String,

    /// Tập keys được phép expose:
    /// { "timeseries": [...], "attributes": { "cs": [...], "ss": [...], "sh": [...] } }
    pub keys: Option<serde_json::Value>,

    /// Giới hạn thời gian telemetry (ms epoch), 0 = không giới hạn
    pub start_ts: i64,
    pub end_ts: i64,

    pub additional_info: Option<serde_json::Value>,
    pub external_id: Option<Uuid>,
    pub version: i64,
}

/// Lightweight variant kèm thông tin customer
/// Java: org.thingsboard.server.common.data.EntityViewInfo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityViewInfo {
    #[serde(flatten)]
    pub entity_view: EntityView,
    pub customer_title: Option<String>,
    pub customer_is_public: bool,
}
