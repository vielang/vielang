use chrono::Utc;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use vl_core::entities::scheduled_job::{CreateJobRequest, JobExecution, ScheduledJob};

use crate::error::DaoError;

pub struct ScheduledJobDao {
    pool: PgPool,
}

impl ScheduledJobDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn save(
        &self,
        tenant_id: Uuid,
        req: &CreateJobRequest,
    ) -> Result<ScheduledJob, DaoError> {
        let now = Utc::now().timestamp_millis();

        let row = sqlx::query!(
            r#"INSERT INTO scheduled_job
               (tenant_id, name, job_type, schedule_type, interval_ms, cron_expression,
                configuration, enabled, next_run_at, created_time)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
               RETURNING id, tenant_id, name, job_type, schedule_type, interval_ms,
                         cron_expression, configuration as "configuration: serde_json::Value",
                         enabled, last_run_at, next_run_at, created_time"#,
            tenant_id,
            req.name,
            req.job_type,
            req.schedule_type,
            req.interval_ms,
            req.cron_expression,
            req.configuration,
            req.enabled,
            0i64,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        Ok(ScheduledJob {
            id: row.id,
            tenant_id: row.tenant_id,
            name: row.name,
            job_type: row.job_type,
            schedule_type: row.schedule_type,
            interval_ms: row.interval_ms,
            cron_expression: row.cron_expression,
            configuration: row.configuration,
            enabled: row.enabled,
            last_run_at: row.last_run_at,
            next_run_at: row.next_run_at,
            created_time: row.created_time,
        })
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<ScheduledJob>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT id, tenant_id, name, job_type, schedule_type, interval_ms,
                      cron_expression, configuration as "configuration: serde_json::Value",
                      enabled, last_run_at, next_run_at, created_time
               FROM scheduled_job WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ScheduledJob {
            id: r.id,
            tenant_id: r.tenant_id,
            name: r.name,
            job_type: r.job_type,
            schedule_type: r.schedule_type,
            interval_ms: r.interval_ms,
            cron_expression: r.cron_expression,
            configuration: r.configuration,
            enabled: r.enabled,
            last_run_at: r.last_run_at,
            next_run_at: r.next_run_at,
            created_time: r.created_time,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<ScheduledJob>, i64), DaoError> {
        let offset = page * page_size;

        let total = sqlx::query_scalar!(
            "SELECT count(*) FROM scheduled_job WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"SELECT id, tenant_id, name, job_type, schedule_type, interval_ms,
                      cron_expression, configuration as "configuration: serde_json::Value",
                      enabled, last_run_at, next_run_at, created_time
               FROM scheduled_job WHERE tenant_id = $1
               ORDER BY created_time LIMIT $2 OFFSET $3"#,
            tenant_id,
            page_size,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows
            .into_iter()
            .map(|r| ScheduledJob {
                id: r.id,
                tenant_id: r.tenant_id,
                name: r.name,
                job_type: r.job_type,
                schedule_type: r.schedule_type,
                interval_ms: r.interval_ms,
                cron_expression: r.cron_expression,
                configuration: r.configuration,
                enabled: r.enabled,
                last_run_at: r.last_run_at,
                next_run_at: r.next_run_at,
                created_time: r.created_time,
            })
            .collect();

        Ok((data, total))
    }

    #[instrument(skip(self))]
    pub async fn update(
        &self,
        id: Uuid,
        req: &CreateJobRequest,
    ) -> Result<ScheduledJob, DaoError> {
        let row = sqlx::query!(
            r#"UPDATE scheduled_job
               SET name=$1, job_type=$2, schedule_type=$3, interval_ms=$4,
                   cron_expression=$5, configuration=$6, enabled=$7
               WHERE id=$8
               RETURNING id, tenant_id, name, job_type, schedule_type, interval_ms,
                         cron_expression, configuration as "configuration: serde_json::Value",
                         enabled, last_run_at, next_run_at, created_time"#,
            req.name,
            req.job_type,
            req.schedule_type,
            req.interval_ms,
            req.cron_expression,
            req.configuration,
            req.enabled,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => DaoError::NotFound,
            other => DaoError::Database(other),
        })?;

        Ok(ScheduledJob {
            id: row.id,
            tenant_id: row.tenant_id,
            name: row.name,
            job_type: row.job_type,
            schedule_type: row.schedule_type,
            interval_ms: row.interval_ms,
            cron_expression: row.cron_expression,
            configuration: row.configuration,
            enabled: row.enabled,
            last_run_at: row.last_run_at,
            next_run_at: row.next_run_at,
            created_time: row.created_time,
        })
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM scheduled_job WHERE id=$1", id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }

    /// Find all enabled jobs whose next_run_at <= now_ms
    #[instrument(skip(self))]
    pub async fn find_due_jobs(&self, now_ms: i64) -> Result<Vec<ScheduledJob>, DaoError> {
        let rows = sqlx::query!(
            r#"SELECT id, tenant_id, name, job_type, schedule_type, interval_ms,
                      cron_expression, configuration as "configuration: serde_json::Value",
                      enabled, last_run_at, next_run_at, created_time
               FROM scheduled_job
               WHERE enabled = true AND next_run_at <= $1
               ORDER BY next_run_at"#,
            now_ms
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ScheduledJob {
                id: r.id,
                tenant_id: r.tenant_id,
                name: r.name,
                job_type: r.job_type,
                schedule_type: r.schedule_type,
                interval_ms: r.interval_ms,
                cron_expression: r.cron_expression,
                configuration: r.configuration,
                enabled: r.enabled,
                last_run_at: r.last_run_at,
                next_run_at: r.next_run_at,
                created_time: r.created_time,
            })
            .collect())
    }

    /// Update last_run_at and next_run_at after a job execution
    #[instrument(skip(self))]
    pub async fn update_next_run(
        &self,
        id: Uuid,
        last_run_at: i64,
        next_run_at: i64,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            "UPDATE scheduled_job SET last_run_at=$1, next_run_at=$2 WHERE id=$3",
            last_run_at,
            next_run_at,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Record a job execution (inserts RUNNING, then updates to final status).
    /// This single function inserts a complete execution record.
    #[instrument(skip(self, error_msg, result))]
    pub async fn record_execution(
        &self,
        job_id: Uuid,
        status: &str,
        error_msg: Option<&str>,
        result: Option<serde_json::Value>,
    ) -> Result<JobExecution, DaoError> {
        let now = Utc::now().timestamp_millis();

        let row = sqlx::query!(
            r#"INSERT INTO job_execution (job_id, started_at, finished_at, status, error_message, result)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, job_id, started_at, finished_at, status,
                         error_message, result as "result: serde_json::Value""#,
            job_id,
            now,
            now,
            status,
            error_msg,
            result as Option<serde_json::Value>
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(JobExecution {
            id: row.id,
            job_id: row.job_id,
            started_at: row.started_at,
            finished_at: row.finished_at,
            status: row.status,
            error_message: row.error_message,
            result: row.result,
        })
    }

    /// Set a job to CANCELLED and disable it (prevents future scheduled runs).
    /// Uses non-macro query because `status` column was added in migration 048.
    #[instrument(skip(self))]
    pub async fn cancel(&self, id: Uuid) -> Result<(), DaoError> {
        let res = sqlx::query(
            "UPDATE scheduled_job SET enabled = false, status = 'CANCELLED' WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        if res.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }

    /// List executions for a job (newest first), up to limit.
    #[instrument(skip(self))]
    pub async fn list_executions(
        &self,
        job_id: Uuid,
        limit: i64,
    ) -> Result<Vec<JobExecution>, DaoError> {
        let rows = sqlx::query!(
            r#"SELECT id, job_id, started_at, finished_at, status,
                      error_message, result as "result: serde_json::Value"
               FROM job_execution
               WHERE job_id = $1
               ORDER BY started_at DESC
               LIMIT $2"#,
            job_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| JobExecution {
                id: r.id,
                job_id: r.job_id,
                started_at: r.started_at,
                finished_at: r.finished_at,
                status: r.status,
                error_message: r.error_message,
                result: r.result,
            })
            .collect())
    }

    /// Find the first job of a given type for a tenant (e.g. for BACKUP schedule lookup).
    #[instrument(skip(self))]
    pub async fn find_by_tenant_and_type(
        &self,
        tenant_id: Uuid,
        job_type: &str,
    ) -> Result<Option<ScheduledJob>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT id, tenant_id, name, job_type, schedule_type, interval_ms,
                      cron_expression, configuration as "configuration: serde_json::Value",
                      enabled, last_run_at, next_run_at, created_time
               FROM scheduled_job
               WHERE tenant_id = $1 AND job_type = $2
               ORDER BY created_time DESC
               LIMIT 1"#,
            tenant_id,
            job_type,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ScheduledJob {
            id: r.id,
            tenant_id: r.tenant_id,
            name: r.name,
            job_type: r.job_type,
            schedule_type: r.schedule_type,
            interval_ms: r.interval_ms,
            cron_expression: r.cron_expression,
            configuration: r.configuration,
            enabled: r.enabled,
            last_run_at: r.last_run_at,
            next_run_at: r.next_run_at,
            created_time: r.created_time,
        }))
    }

    /// Insert or update the single scheduled job of a given type for a tenant.
    #[instrument(skip(self, configuration))]
    pub async fn upsert_by_tenant_and_type(
        &self,
        tenant_id: Uuid,
        job_type: &str,
        cron: &str,
        configuration: serde_json::Value,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO scheduled_job
                (tenant_id, name, job_type, schedule_type, cron_expression,
                 configuration, enabled, next_run_at, created_time)
            VALUES ($1, $2, $3, 'CRON', $4, $5, true, 0, $6)
            ON CONFLICT DO NOTHING
            "#,
            tenant_id,
            format!("{job_type} scheduled job"),
            job_type,
            cron,
            configuration,
            Utc::now().timestamp_millis(),
        )
        .execute(&self.pool)
        .await?;

        // Update existing if it already existed
        sqlx::query!(
            r#"
            UPDATE scheduled_job
            SET cron_expression = $3,
                configuration   = $4,
                enabled         = true
            WHERE tenant_id = $1 AND job_type = $2
            "#,
            tenant_id,
            job_type,
            cron,
            configuration,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
