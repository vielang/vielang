use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Khớp với bảng `device`.
/// Java: org.thingsboard.server.common.data.Device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub customer_id: Option<Uuid>,

    pub name: String,
    /// Loại device — map với device_profile.name
    pub device_type: String,
    pub label: Option<String>,
    pub device_profile_id: Uuid,

    /// JSONB — DeviceData (transport config, credentials type)
    pub device_data: Option<serde_json::Value>,

    pub firmware_id: Option<Uuid>,
    pub software_id: Option<Uuid>,

    pub external_id: Option<Uuid>,
    pub additional_info: Option<serde_json::Value>,
    pub version: i64,
}

impl Device {
    /// Khớp Java: getOwnerId() — customerId nếu có, còn lại tenantId
    pub fn owner_id(&self) -> Uuid {
        self.customer_id.unwrap_or(self.tenant_id)
    }
}

/// Thông tin device gọn để trả về trong API list responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub name: String,
    pub device_type: String,
    pub label: Option<String>,
    pub device_profile_id: Uuid,
    pub active: bool,
}

/// Device info view — JOIN device + device_profile + customer
/// Java: org.thingsboard.server.common.data.DeviceInfo
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfoView {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub name: String,
    pub label: Option<String>,
    pub device_profile_id: Uuid,
    pub device_profile_name: String,
    pub customer_title: Option<String>,
    pub firmware_id: Option<Uuid>,
    pub software_id: Option<Uuid>,
}
