use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Khớp với bảng `customer`.
/// Java: org.thingsboard.server.common.data.Customer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,

    pub title: String,

    // ── ContactBased ──────────────────────────────────────────────────────────
    pub country: Option<String>,
    pub state: Option<String>,
    pub city: Option<String>,
    pub address: Option<String>,
    pub address2: Option<String>,
    pub zip: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,

    /// Nullable — external system ID
    pub external_id: Option<Uuid>,

    pub additional_info: Option<serde_json::Value>,

    pub is_public: bool,
    pub version: i64,
}

impl Customer {
    /// Khớp Java: isPublic()
    pub fn is_public(&self) -> bool {
        self.is_public
    }
}
