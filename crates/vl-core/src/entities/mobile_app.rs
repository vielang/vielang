use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Enums ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlatformType {
    Android,
    Ios,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MobileAppStatus {
    Published,
    Deprecated,
    Suspended,
    Draft,
}

// ── MobileApp ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileApp {
    pub id:            Uuid,
    pub created_time:  i64,
    pub tenant_id:     Uuid,
    pub pkg_name:      String,
    pub title:         Option<String>,
    pub app_secret:    String,
    pub platform_type: PlatformType,
    pub status:        MobileAppStatus,
    pub version_info:  Option<serde_json::Value>,
    pub store_info:    Option<serde_json::Value>,
}

// ── MobileAppBundle ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileAppBundle {
    pub id:                Uuid,
    pub created_time:      i64,
    pub tenant_id:         Uuid,
    pub title:             Option<String>,
    pub android_app_id:    Option<Uuid>,
    pub ios_app_id:        Option<Uuid>,
    pub layout_config:     Option<serde_json::Value>,
    pub oauth2_client_ids: Vec<Uuid>,
}

// ── QrCodeSettings ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QrCodeSettings {
    pub id:                    Uuid,
    pub created_time:          i64,
    pub tenant_id:             Uuid,
    pub use_system_settings:   bool,
    pub use_default_app:       bool,
    pub mobile_app_bundle_id:  Option<Uuid>,
    pub qr_code_config:        serde_json::Value,
    pub android_enabled:       bool,
    pub ios_enabled:           bool,
    /// Resolved from linked bundle at query time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_play_link:      Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_store_link:        Option<String>,
}

// ── LoginMobileInfo ───────────────────────────────────────────────────────────

/// Returned by GET /api/noauth/mobile — pre-login info for mobile client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginMobileInfo {
    pub qr_enabled:        bool,
    pub android_enabled:   bool,
    pub ios_enabled:       bool,
    pub google_play_link:  Option<String>,
    pub app_store_link:    Option<String>,
}
