use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Khớp với bảng `device_credentials`.
/// Java: org.thingsboard.server.common.data.security.DeviceCredentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCredentials {
    pub id: Uuid,
    pub created_time: i64,
    pub device_id: Uuid,
    pub credentials_type: DeviceCredentialsType,
    /// MQTT: access token, X509: cert CN
    pub credentials_id: String,
    /// MQTT password, X509: full cert
    pub credentials_value: Option<String>,
}

/// Khớp Java: DeviceCredentialsType
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceCredentialsType {
    AccessToken,
    X509Certificate,
    MqttBasic,
    Lwm2mCredentials,
}
