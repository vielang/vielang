use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Khớp với bảng `dashboard`.
/// Java: org.thingsboard.server.common.data.Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub title: Option<String>,
    /// JSONB — layout + widget configuration
    pub configuration: Option<serde_json::Value>,
    pub external_id: Option<Uuid>,
    pub mobile_hide: bool,
    pub mobile_order: Option<i32>,
    pub version: i64,
}

/// Lightweight dashboard without configuration.
/// Java: org.thingsboard.server.common.data.DashboardInfo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardInfo {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub title: Option<String>,
    pub assigned_customers: Option<String>, // JSON array of customer objects
    pub mobile_hide: bool,
    pub mobile_order: Option<i32>,
}

/// Home dashboard reference for a tenant.
/// Java: org.thingsboard.server.common.data.HomeDashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeDashboardInfo {
    pub dashboard_id: Option<Uuid>,
    pub hidden_dashboard_toolbar: bool,
}
