use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::BackupExportLog;
use crate::DaoError;

pub struct BackupExportDao {
    pool: PgPool,
}

impl BackupExportDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    /// Record a completed (or failed) backup export in the audit log.
    #[instrument(skip(self))]
    pub async fn record(&self, log: &BackupExportLog) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO backup_export_log (
                id, tenant_id, created_time,
                device_count, asset_count, customer_count,
                dashboard_count, rule_chain_count, user_count,
                include_telemetry, status, error_message
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
            "#,
            log.id,
            log.tenant_id,
            log.created_time,
            log.device_count,
            log.asset_count,
            log.customer_count,
            log.dashboard_count,
            log.rule_chain_count,
            log.user_count,
            log.include_telemetry,
            log.status,
            log.error_message,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Fetch recent export history for a tenant (most recent first).
    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        limit: i64,
    ) -> Result<Vec<BackupExportLog>, DaoError> {
        let rows: Vec<_> = sqlx::query!(
            r#"
            SELECT id, tenant_id, created_time,
                   device_count, asset_count, customer_count,
                   dashboard_count, rule_chain_count, user_count,
                   include_telemetry, status, error_message
            FROM backup_export_log
            WHERE tenant_id = $1
            ORDER BY created_time DESC
            LIMIT $2
            "#,
            tenant_id,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| BackupExportLog {
            id:               r.id,
            tenant_id:        r.tenant_id,
            created_time:     r.created_time,
            device_count:     r.device_count,
            asset_count:      r.asset_count,
            customer_count:   r.customer_count,
            dashboard_count:  r.dashboard_count,
            rule_chain_count: r.rule_chain_count,
            user_count:       r.user_count,
            include_telemetry: r.include_telemetry,
            status:           r.status,
            error_message:    r.error_message,
        }).collect())
    }
}
