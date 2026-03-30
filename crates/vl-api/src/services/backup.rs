use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;
use tracing::{info, instrument, warn};
use uuid::Uuid;

use vl_core::entities::{
    BackupEntities, BackupExportLog, BackupScheduleConfig, ExportOptions,
    ImportMode, ImportOptions, ImportReport, TelemetryInfo, TenantBackup,
};
use vl_dao::{
    BackupExportDao,
    postgres::{
        asset::AssetDao,
        customer::CustomerDao,
        dashboard::DashboardDao,
        device::DeviceDao,
        rule_chain::RuleChainDao,
        tenant::TenantDao,
        user::UserDao,
    },
};

use crate::services::job_scheduler::JobHandler;

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("DAO error: {0}")]
    Dao(#[from] vl_dao::DaoError),
    #[error("Tenant not found")]
    TenantNotFound,
    #[error("Serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
}

// ── ExportService ─────────────────────────────────────────────────────────────

pub struct ExportService {
    tenant_dao:     Arc<TenantDao>,
    device_dao:     Arc<DeviceDao>,
    asset_dao:      Arc<AssetDao>,
    customer_dao:   Arc<CustomerDao>,
    dashboard_dao:  Arc<DashboardDao>,
    rule_chain_dao: Arc<RuleChainDao>,
    user_dao:       Arc<UserDao>,
    export_dao:     Arc<BackupExportDao>,
}

impl ExportService {
    pub fn new(
        tenant_dao:     Arc<TenantDao>,
        device_dao:     Arc<DeviceDao>,
        asset_dao:      Arc<AssetDao>,
        customer_dao:   Arc<CustomerDao>,
        dashboard_dao:  Arc<DashboardDao>,
        rule_chain_dao: Arc<RuleChainDao>,
        user_dao:       Arc<UserDao>,
        export_dao:     Arc<BackupExportDao>,
    ) -> Self {
        Self {
            tenant_dao, device_dao, asset_dao, customer_dao,
            dashboard_dao, rule_chain_dao, user_dao, export_dao,
        }
    }

    #[instrument(skip(self))]
    pub async fn export_tenant(
        &self,
        tenant_id: Uuid,
        options: &ExportOptions,
    ) -> Result<TenantBackup, BackupError> {
        let tenant = self.tenant_dao.find_by_id(tenant_id).await?
            .ok_or(BackupError::TenantNotFound)?;

        let mut backup = TenantBackup::new(tenant_id, tenant.title);

        // Export all entity types in parallel
        let (devices, assets, customers, dashboards, rule_chains, users) = tokio::join!(
            self.device_dao.find_all_by_tenant(tenant_id),
            self.asset_dao.find_all_by_tenant(tenant_id),
            self.customer_dao.find_all_by_tenant(tenant_id),
            self.dashboard_dao.find_all_by_tenant(tenant_id),
            self.rule_chain_dao.find_all_by_tenant(tenant_id),
            self.user_dao.find_all_by_tenant(tenant_id),
        );

        backup.entities = BackupEntities {
            devices:     devices?,
            assets:      assets?,
            customers:   customers?,
            dashboards:  dashboards?,
            rule_chains: rule_chains?,
            users:       users?,
        };
        backup.telemetry = TelemetryInfo::excluded();

        // Record audit log
        let log = BackupExportLog {
            id:               Uuid::new_v4(),
            tenant_id,
            created_time:     backup.exported_at,
            device_count:     backup.entities.devices.len() as i32,
            asset_count:      backup.entities.assets.len() as i32,
            customer_count:   backup.entities.customers.len() as i32,
            dashboard_count:  backup.entities.dashboards.len() as i32,
            rule_chain_count: backup.entities.rule_chains.len() as i32,
            user_count:       backup.entities.users.len() as i32,
            include_telemetry: options.include_telemetry,
            status:           "COMPLETED".to_string(),
            error_message:    None,
        };

        if let Err(e) = self.export_dao.record(&log).await {
            warn!("Failed to record backup audit log: {e}");
        }

        info!(
            tenant_id = %tenant_id,
            devices = backup.entities.devices.len(),
            assets = backup.entities.assets.len(),
            "Backup export completed"
        );

        Ok(backup)
    }

    /// Return recent export history for a tenant.
    pub async fn export_history(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<BackupExportLog>, BackupError> {
        Ok(self.export_dao.find_by_tenant(tenant_id, 20).await?)
    }
}

// ── ImportService ─────────────────────────────────────────────────────────────

pub struct ImportService {
    device_dao:     Arc<DeviceDao>,
    asset_dao:      Arc<AssetDao>,
    customer_dao:   Arc<CustomerDao>,
    dashboard_dao:  Arc<DashboardDao>,
    rule_chain_dao: Arc<RuleChainDao>,
    user_dao:       Arc<UserDao>,
}

impl ImportService {
    pub fn new(
        device_dao:     Arc<DeviceDao>,
        asset_dao:      Arc<AssetDao>,
        customer_dao:   Arc<CustomerDao>,
        dashboard_dao:  Arc<DashboardDao>,
        rule_chain_dao: Arc<RuleChainDao>,
        user_dao:       Arc<UserDao>,
    ) -> Self {
        Self { device_dao, asset_dao, customer_dao, dashboard_dao, rule_chain_dao, user_dao }
    }

    #[instrument(skip(self, backup))]
    pub async fn import_tenant(
        &self,
        backup:    &TenantBackup,
        target_id: Uuid,
        options:   &ImportOptions,
    ) -> Result<ImportReport, BackupError> {
        let mut report = ImportReport::default();

        // Import in dependency order: customers → devices → assets → dashboards → rule_chains → users

        for mut customer in backup.entities.customers.clone() {
            customer.tenant_id = target_id;
            if options.mode == ImportMode::Append {
                customer.id = Uuid::new_v4();
            }
            if options.mode == ImportMode::Skip {
                if let Ok(Some(_)) = self.customer_dao.find_by_id(customer.id).await {
                    report.skipped += 1;
                    continue;
                }
            }
            match self.customer_dao.save(&customer).await {
                Ok(_) => report.customers_imported += 1,
                Err(e) => report.errors.push(format!("customer {}: {e}", customer.id)),
            }
        }

        for mut device in backup.entities.devices.clone() {
            device.tenant_id = target_id;
            if options.mode == ImportMode::Append {
                device.id = Uuid::new_v4();
            }
            if options.mode == ImportMode::Skip {
                if let Ok(Some(_)) = self.device_dao.find_by_id(device.id).await {
                    report.skipped += 1;
                    continue;
                }
            }
            match self.device_dao.save(&device).await {
                Ok(_) => report.devices_imported += 1,
                Err(e) => report.errors.push(format!("device {}: {e}", device.id)),
            }
        }

        for mut asset in backup.entities.assets.clone() {
            asset.tenant_id = target_id;
            if options.mode == ImportMode::Append {
                asset.id = Uuid::new_v4();
            }
            if options.mode == ImportMode::Skip {
                if let Ok(Some(_)) = self.asset_dao.find_by_id(asset.id).await {
                    report.skipped += 1;
                    continue;
                }
            }
            match self.asset_dao.save(&asset).await {
                Ok(_) => report.assets_imported += 1,
                Err(e) => report.errors.push(format!("asset {}: {e}", asset.id)),
            }
        }

        for mut dashboard in backup.entities.dashboards.clone() {
            dashboard.tenant_id = target_id;
            if options.mode == ImportMode::Append {
                dashboard.id = Uuid::new_v4();
            }
            if options.mode == ImportMode::Skip {
                if let Ok(Some(_)) = self.dashboard_dao.find_by_id(dashboard.id).await {
                    report.skipped += 1;
                    continue;
                }
            }
            match self.dashboard_dao.save(&dashboard).await {
                Ok(_) => report.dashboards_imported += 1,
                Err(e) => report.errors.push(format!("dashboard {}: {e}", dashboard.id)),
            }
        }

        for mut chain in backup.entities.rule_chains.clone() {
            chain.tenant_id = target_id;
            if options.mode == ImportMode::Append {
                chain.id = Uuid::new_v4();
            }
            if options.mode == ImportMode::Skip {
                if let Ok(Some(_)) = self.rule_chain_dao.find_by_id(chain.id).await {
                    report.skipped += 1;
                    continue;
                }
            }
            match self.rule_chain_dao.save(&chain).await {
                Ok(_) => report.rule_chains_imported += 1,
                Err(e) => report.errors.push(format!("rule_chain {}: {e}", chain.id)),
            }
        }

        for mut user in backup.entities.users.clone() {
            user.tenant_id = target_id;
            if options.mode == ImportMode::Append {
                user.id = Uuid::new_v4();
            }
            if options.mode == ImportMode::Skip {
                if let Ok(Some(_)) = self.user_dao.find_by_id(user.id).await {
                    report.skipped += 1;
                    continue;
                }
            }
            match self.user_dao.save(&user).await {
                Ok(_) => report.users_imported += 1,
                Err(e) => report.errors.push(format!("user {}: {e}", user.id)),
            }
        }

        info!(
            target_id = %target_id,
            devices = report.devices_imported,
            assets = report.assets_imported,
            skipped = report.skipped,
            errors = report.errors.len(),
            "Backup import completed"
        );

        Ok(report)
    }
}

// ── BackupJobHandler (for scheduled backups via job scheduler) ─────────────────

pub struct BackupJobHandler {
    export_svc: Arc<ExportService>,
}

impl BackupJobHandler {
    pub fn new(export_svc: Arc<ExportService>) -> Self {
        Self { export_svc }
    }
}

#[async_trait]
impl JobHandler for BackupJobHandler {
    fn job_type(&self) -> &'static str { "BACKUP" }

    async fn execute(&self, job: &vl_core::entities::ScheduledJob) -> Result<serde_json::Value, String> {
        let cfg: BackupScheduleConfig = job.configuration
            .get("backup")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        let options = ExportOptions {
            include_telemetry: cfg.include_telemetry,
            entities: vec![],
        };

        let backup = self.export_svc
            .export_tenant(job.tenant_id, &options)
            .await
            .map_err(|e| format!("export failed: {e}"))?;

        // Write to output_dir/<tenant_id>/<timestamp>.json
        let filename = format!(
            "{}/{}/{}.json",
            cfg.output_dir,
            job.tenant_id,
            backup.exported_at
        );

        let json = serde_json::to_string(&backup)
            .map_err(|e| format!("serialize: {e}"))?;

        // Best-effort file write — if output_dir doesn't exist this is a config issue
        if let Err(e) = tokio::fs::create_dir_all(format!("{}/{}", cfg.output_dir, job.tenant_id)).await {
            return Err(format!("cannot create backup dir: {e}"));
        }
        tokio::fs::write(&filename, json)
            .await
            .map_err(|e| format!("write {filename}: {e}"))?;

        Ok(json!({ "file": filename, "exportedAt": backup.exported_at }))
    }
}
