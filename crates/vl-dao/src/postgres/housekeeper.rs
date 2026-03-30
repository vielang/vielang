use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::HousekeeperExecution;
use crate::DaoError;

/// Statistics returned after a cleanup cycle.
#[derive(Debug, Clone)]
pub struct CleanupStats {
    pub cleaned_telemetry: i64,
    pub cleaned_events: i64,
    pub cleaned_alarms: i64,
    pub cleaned_rpc: i64,
}

pub struct HousekeeperDao {
    pool: PgPool,
}

impl HousekeeperDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Delete old timeseries records (ts < cutoff_ms), up to batch_size rows.
    /// ts_kv has no standalone id — use composite PK (entity_id, key, ts).
    #[instrument(skip(self))]
    pub async fn delete_old_telemetry(&self, cutoff_ms: i64, batch_size: i64) -> Result<i64, DaoError> {
        let count = sqlx::query_scalar!(
            r#"
            WITH deleted AS (
                DELETE FROM ts_kv
                WHERE (entity_id, key, ts) IN (
                    SELECT entity_id, key, ts
                    FROM ts_kv
                    WHERE ts < $1
                    LIMIT $2
                )
                RETURNING entity_id
            )
            SELECT count(*) FROM deleted
            "#,
            cutoff_ms,
            batch_size
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.unwrap_or(0))
    }

    /// Delete old event records (created_time < cutoff_ms), up to batch_size rows.
    #[instrument(skip(self))]
    pub async fn delete_old_events(&self, cutoff_ms: i64, batch_size: i64) -> Result<i64, DaoError> {
        let count = sqlx::query_scalar!(
            r#"
            WITH deleted AS (
                DELETE FROM event
                WHERE id IN (
                    SELECT id FROM event WHERE created_time < $1 LIMIT $2
                )
                RETURNING id
            )
            SELECT count(*) FROM deleted
            "#,
            cutoff_ms,
            batch_size
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.unwrap_or(0))
    }

    /// Delete old alarm records (start_ts < cutoff_ms), up to batch_size rows.
    #[instrument(skip(self))]
    pub async fn delete_old_alarms(&self, cutoff_ms: i64, batch_size: i64) -> Result<i64, DaoError> {
        let count = sqlx::query_scalar!(
            r#"
            WITH deleted AS (
                DELETE FROM alarm
                WHERE id IN (
                    SELECT id FROM alarm WHERE start_ts < $1 LIMIT $2
                )
                RETURNING id
            )
            SELECT count(*) FROM deleted
            "#,
            cutoff_ms,
            batch_size
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.unwrap_or(0))
    }

    /// Delete expired RPC records (expiration_time < cutoff_ms), up to batch_size rows.
    #[instrument(skip(self))]
    pub async fn delete_old_rpc(&self, cutoff_ms: i64, batch_size: i64) -> Result<i64, DaoError> {
        let count = sqlx::query_scalar!(
            r#"
            WITH deleted AS (
                DELETE FROM rpc
                WHERE id IN (
                    SELECT id FROM rpc WHERE expiration_time < $1 LIMIT $2
                )
                RETURNING id
            )
            SELECT count(*) FROM deleted
            "#,
            cutoff_ms,
            batch_size
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.unwrap_or(0))
    }

    /// Insert a new execution record with status RUNNING, return the new id.
    #[instrument(skip(self))]
    pub async fn start_execution(&self) -> Result<Uuid, DaoError> {
        let now_ms = chrono::Utc::now().timestamp_millis();
        let row = sqlx::query!(
            r#"
            INSERT INTO housekeeper_execution (started_at, status)
            VALUES ($1, 'RUNNING')
            RETURNING id
            "#,
            now_ms
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    /// Mark execution as finished with stats and final status.
    #[instrument(skip(self))]
    pub async fn finish_execution(
        &self,
        id: Uuid,
        stats: CleanupStats,
        status: &str,
    ) -> Result<(), DaoError> {
        let finished_at = chrono::Utc::now().timestamp_millis();
        sqlx::query!(
            r#"
            UPDATE housekeeper_execution
            SET finished_at        = $1,
                cleaned_telemetry  = $2,
                cleaned_events     = $3,
                cleaned_alarms     = $4,
                cleaned_rpc        = $5,
                status             = $6
            WHERE id = $7
            "#,
            finished_at,
            stats.cleaned_telemetry,
            stats.cleaned_events,
            stats.cleaned_alarms,
            stats.cleaned_rpc,
            status,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// List recent executions (newest first), up to limit.
    #[instrument(skip(self))]
    pub async fn list_executions(&self, limit: i64) -> Result<Vec<HousekeeperExecution>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, started_at, finished_at,
                   cleaned_telemetry, cleaned_events, cleaned_alarms, cleaned_rpc,
                   status
            FROM housekeeper_execution
            ORDER BY started_at DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| HousekeeperExecution {
            id: r.id,
            started_at: r.started_at,
            finished_at: r.finished_at,
            cleaned_telemetry: r.cleaned_telemetry,
            cleaned_events: r.cleaned_events,
            cleaned_alarms: r.cleaned_alarms,
            cleaned_rpc: r.cleaned_rpc,
            status: r.status,
        }).collect())
    }
}
