use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Asset, Customer, Dashboard, Device, RuleChain, User};

// ── Export bundle ─────────────────────────────────────────────────────────────

/// The full JSON export bundle for a tenant.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantBackup {
    pub version:      String,
    pub exported_at:  i64,
    pub tenant_id:    Uuid,
    pub tenant_name:  String,
    pub entities:     BackupEntities,
    pub telemetry:    TelemetryInfo,
}

impl TenantBackup {
    pub fn new(tenant_id: Uuid, tenant_name: String) -> Self {
        Self {
            version:     "1.0".to_string(),
            exported_at: chrono::Utc::now().timestamp_millis(),
            tenant_id,
            tenant_name,
            entities:  BackupEntities::default(),
            telemetry: TelemetryInfo::default(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupEntities {
    pub devices:     Vec<Device>,
    pub assets:      Vec<Asset>,
    pub customers:   Vec<Customer>,
    pub dashboards:  Vec<Dashboard>,
    pub rule_chains: Vec<RuleChain>,
    pub users:       Vec<User>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryInfo {
    pub included: bool,
    pub note:     String,
}

impl TelemetryInfo {
    pub fn excluded() -> Self {
        Self {
            included: false,
            note: "Telemetry excluded by default due to size".to_string(),
        }
    }
}

// ── Options ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOptions {
    pub include_telemetry: bool,
    /// Entity types to include; empty = all.
    pub entities: Vec<String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_telemetry: false,
            entities: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ImportMode {
    /// Skip entities that already exist (default).
    Skip,
    /// Overwrite existing entities.
    Overwrite,
    /// Re-assign new UUIDs so entities are always appended.
    Append,
}

impl Default for ImportMode {
    fn default() -> Self { Self::Skip }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportOptions {
    pub mode:      ImportMode,
    /// Override the target tenant (defaults to the backup's tenantId).
    pub target_tenant_id: Option<Uuid>,
}

// ── Import report ─────────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub devices_imported:     usize,
    pub assets_imported:      usize,
    pub customers_imported:   usize,
    pub dashboards_imported:  usize,
    pub rule_chains_imported: usize,
    pub users_imported:       usize,
    pub skipped:              usize,
    pub errors:               Vec<String>,
}

// ── Audit log entry (mirrors backup_export_log table) ─────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupExportLog {
    pub id:               Uuid,
    pub tenant_id:        Uuid,
    pub created_time:     i64,
    pub device_count:     i32,
    pub asset_count:      i32,
    pub customer_count:   i32,
    pub dashboard_count:  i32,
    pub rule_chain_count: i32,
    pub user_count:       i32,
    pub include_telemetry: bool,
    pub status:           String,
    pub error_message:    Option<String>,
}

// ── Scheduled backup config (stored in job configuration JSON) ─────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupScheduleConfig {
    /// Cron expression, e.g. "0 0 2 * * *" (daily at 2am).
    pub cron:              String,
    pub include_telemetry: bool,
    pub output_dir:        String,
}

impl Default for BackupScheduleConfig {
    fn default() -> Self {
        Self {
            cron:              "0 0 2 * * *".to_string(),
            include_telemetry: false,
            output_dir:        "/var/vielang/backups".to_string(),
        }
    }
}
