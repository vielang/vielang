use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Khớp với bảng `device_profile`.
/// Java: org.thingsboard.server.common.data.DeviceProfile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,

    pub name: String,
    pub description: Option<String>,

    /// Base64 hoặc URL — hiển thị trên mobile app
    pub image: Option<String>,

    pub is_default: bool,
    pub device_profile_type: DeviceProfileType,
    pub transport_type: DeviceTransportType,
    pub provision_type: DeviceProvisionType,

    /// JSONB — chứa transport config, alarm rules, etc.
    pub profile_data: Option<serde_json::Value>,

    pub default_rule_chain_id: Option<Uuid>,
    pub default_dashboard_id: Option<Uuid>,
    pub default_queue_name: Option<String>,
    pub default_edge_rule_chain_id: Option<Uuid>,

    /// Unique key dùng cho Device Provisioning API
    pub provision_device_key: Option<String>,

    pub firmware_id: Option<Uuid>,
    pub software_id: Option<Uuid>,

    pub external_id: Option<Uuid>,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceProfileType {
    Default,
}

/// Khớp Java: DeviceTransportType
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceTransportType {
    Default,
    Mqtt,
    Coap,
    Lwm2m,
    Snmp,
}

/// Khớp Java: DeviceProfileProvisionType
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceProvisionType {
    Disabled,
    AllowCreateNewDevices,
    CheckPreProvisionedDevices,
    X509CertificateChain,
}
