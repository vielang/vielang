use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Khớp với bảng `tenant_profile`.
/// Java: org.thingsboard.server.common.data.TenantProfile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantProfile {
    pub id: Uuid,
    pub created_time: i64,

    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub isolated_vl_rule_engine: bool,

    /// JSONB — chứa resource limits, rate limits, API quotas
    pub profile_data: Option<serde_json::Value>,

    pub version: i64,
}

/// Lightweight variant — chỉ id + name, dùng cho list endpoints
/// Java: org.thingsboard.server.common.data.EntityInfo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInfo {
    pub id: Uuid,
    pub name: String,
}
