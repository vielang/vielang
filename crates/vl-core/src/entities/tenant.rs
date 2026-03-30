
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Khớp với bảng `tenant` trong ThingsBoard PostgreSQL schema.
/// Java: org.thingsboard.server.common.data.Tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    /// Milliseconds since epoch — Java: long createdTime
    pub created_time: i64,

    // ── ContactBased fields ───────────────────────────────────────────────────
    pub country: Option<String>,
    pub state: Option<String>,
    pub city: Option<String>,
    pub address: Option<String>,
    pub address2: Option<String>,
    pub zip: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,

    // ── Tenant-specific ───────────────────────────────────────────────────────
    pub title: String,
    pub region: Option<String>,
    pub tenant_profile_id: Uuid,

    /// JSON stored as varchar in DB (additionalInfo)
    pub additional_info: Option<serde_json::Value>,

    /// Optimistic locking version
    pub version: i64,
}

/// Thông tin thu gọn dùng trong API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantInfo {
    pub id: Uuid,
    pub title: String,
    pub tenant_profile_id: Uuid,
    pub created_time: i64,
}

impl From<Tenant> for TenantInfo {
    fn from(t: Tenant) -> Self {
        Self {
            id: t.id,
            title: t.title,
            tenant_profile_id: t.tenant_profile_id,
            created_time: t.created_time,
        }
    }
}
