use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── AdminSettings ─────────────────────────────────────────────────────────────

/// Mirrors ThingsBoard AdminSettings — key-value JSON store for system config
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminSettings {
    pub id:           Uuid,
    pub created_time: i64,
    pub tenant_id:    Uuid,
    pub key:          String,
    pub json_value:   serde_json::Value,
}

// ── UsageInfo ─────────────────────────────────────────────────────────────────

/// Mirrors ThingsBoard UsageInfo — aggregate resource usage for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageInfo {
    pub devices:                i64,
    pub max_devices:            i64,
    pub assets:                 i64,
    pub max_assets:             i64,
    pub customers:              i64,
    pub max_customers:          i64,
    pub users:                  i64,
    pub max_users:              i64,
    pub dashboards:             i64,
    pub max_dashboards:         i64,
    pub edges:                  i64,
    pub max_edges:              i64,
    pub transport_messages:     i64,
    pub max_transport_messages: i64,
    pub js_executions:          i64,
    pub tbel_executions:        i64,
    pub max_js_executions:      i64,
    pub max_tbel_executions:    i64,
    pub emails:                 i64,
    pub max_emails:             i64,
    pub sms:                    i64,
    pub max_sms:                i64,
    pub sms_enabled:            Option<bool>,
    pub alarms:                 i64,
    pub max_alarms:             i64,
}

// ── SecuritySettings ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecuritySettings {
    pub password_policy:               PasswordPolicy,
    pub max_failed_login_attempts:     Option<i32>,
    pub user_lockout_notification_email: Option<String>,
    pub user_activation_token_ttl:     i32,
    pub password_reset_token_ttl:      i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordPolicy {
    pub minimum_length:               i32,
    pub minimum_uppercase_letters:    Option<i32>,
    pub minimum_lowercase_letters:    Option<i32>,
    pub minimum_digits:               Option<i32>,
    pub minimum_special_characters:   Option<i32>,
    pub password_expiration_period_days: Option<i32>,
    pub allow_whitespaces:            Option<bool>,
    pub force_user_to_reset_password_if_not_valid: Option<bool>,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            password_policy: PasswordPolicy {
                minimum_length:                    6,
                minimum_uppercase_letters:         None,
                minimum_lowercase_letters:         None,
                minimum_digits:                    None,
                minimum_special_characters:        None,
                password_expiration_period_days:   None,
                allow_whitespaces:                 None,
                force_user_to_reset_password_if_not_valid: None,
            },
            max_failed_login_attempts:          None,
            user_lockout_notification_email:    None,
            user_activation_token_ttl:          24,
            password_reset_token_ttl:           24,
        }
    }
}

// ── JwtSettings ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtSettingsInfo {
    pub token_expiration_time:    i64,
    pub refresh_token_exp_time:   i64,
    pub token_issuer:             String,
    /// Base64-encoded signing key (masked on read)
    pub token_signing_key:        String,
}

// ── SystemInfo ────────────────────────────────────────────────────────────────

/// Khớp Java: SystemInfoController — "monolith" not "isMonolith"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    #[serde(rename = "monolith")]
    pub is_monolith:  bool,
    #[serde(rename = "systemData")]
    pub system_data:  Vec<SystemInfoData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfoData {
    pub service_id:    String,
    pub service_type:  String,
    pub cpu_usage:     f64,
    pub cpu_count:     u64,
    pub memory_usage:  f64,
    pub total_memory:  u64,
    pub disk_usage:    f64,
    pub total_disk_space: u64,
}

// ── SystemParams ──────────────────────────────────────────────────────────────

/// Platform configuration params — returned to all authenticated users
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemParams {
    pub user_token_access_enabled:          bool,
    pub allowed_dashboard_ids:              Vec<uuid::Uuid>,
    pub edges_support_enabled:              bool,
    pub has_repository:                     bool,
    pub tbel_enabled:                       bool,
    pub persist_device_state_to_telemetry:  bool,
    pub user_settings:                      Option<serde_json::Value>,
    pub max_datapoints_limit:               i64,
    pub max_resource_size:                  i64,
    pub mobile_qr_enabled:                  bool,
    pub max_debug_mode_duration_minutes:    i32,
}

// ── FeaturesInfo ─────────────────────────────────────────────────────────────

/// Khớp Java: FeaturesInfoController — exact field names from Java TB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesInfo {
    #[serde(rename = "emailEnabled")]
    pub email_enabled: bool,
    #[serde(rename = "oauthEnabled")]
    pub oauth_enabled: bool,
    #[serde(rename = "smsEnabled")]
    pub sms_enabled: bool,
    #[serde(rename = "notificationEnabled")]
    pub notification_enabled: bool,
    #[serde(rename = "twoFaEnabled")]
    pub two_fa_enabled: bool,
}
