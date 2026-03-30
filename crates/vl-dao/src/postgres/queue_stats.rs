use sqlx::PgPool;
use tracing::instrument;

use vl_core::entities::QueueStats;
use crate::DaoError;

pub struct QueueStatsDao {
    pool: PgPool,
}

impl QueueStatsDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Record a stats snapshot for a queue. Returns the inserted row.
    #[instrument(skip(self))]
    pub async fn record_stats(
        &self,
        queue_name: &str,
        messages_total: i64,
        messages_per_second: f64,
        consumers_total: i32,
        lag: i64,
    ) -> Result<QueueStats, DaoError> {
        let collected_at = chrono::Utc::now().timestamp_millis();
        let row = sqlx::query!(
            r#"
            INSERT INTO queue_stats
                (queue_name, messages_total, messages_per_second, consumers_total, lag, collected_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (queue_name, collected_at) DO UPDATE
                SET messages_total       = EXCLUDED.messages_total,
                    messages_per_second  = EXCLUDED.messages_per_second,
                    consumers_total      = EXCLUDED.consumers_total,
                    lag                  = EXCLUDED.lag
            RETURNING id, queue_name, messages_total, messages_per_second, consumers_total, lag, collected_at
            "#,
            queue_name,
            messages_total,
            messages_per_second,
            consumers_total,
            lag,
            collected_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(QueueStats {
            id: row.id,
            queue_name: row.queue_name,
            messages_total: row.messages_total,
            messages_per_second: row.messages_per_second,
            consumers_total: row.consumers_total,
            lag: row.lag,
            collected_at: row.collected_at,
        })
    }

    /// Get the most recent stats snapshot for a queue.
    #[instrument(skip(self))]
    pub async fn get_latest(&self, queue_name: &str) -> Result<Option<QueueStats>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, queue_name, messages_total, messages_per_second, consumers_total, lag, collected_at
            FROM queue_stats
            WHERE queue_name = $1
            ORDER BY collected_at DESC
            LIMIT 1
            "#,
            queue_name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| QueueStats {
            id: r.id,
            queue_name: r.queue_name,
            messages_total: r.messages_total,
            messages_per_second: r.messages_per_second,
            consumers_total: r.consumers_total,
            lag: r.lag,
            collected_at: r.collected_at,
        }))
    }

    /// Get historical stats for a queue within a time range.
    #[instrument(skip(self))]
    pub async fn get_history(
        &self,
        queue_name: &str,
        from_ts: i64,
        to_ts: i64,
    ) -> Result<Vec<QueueStats>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, queue_name, messages_total, messages_per_second, consumers_total, lag, collected_at
            FROM queue_stats
            WHERE queue_name = $1
              AND collected_at >= $2
              AND collected_at <= $3
            ORDER BY collected_at ASC
            "#,
            queue_name,
            from_ts,
            to_ts
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| QueueStats {
            id: r.id,
            queue_name: r.queue_name,
            messages_total: r.messages_total,
            messages_per_second: r.messages_per_second,
            consumers_total: r.consumers_total,
            lag: r.lag,
            collected_at: r.collected_at,
        }).collect())
    }

    /// Delete stats older than `before_ts`. Returns the number of deleted rows.
    #[instrument(skip(self))]
    pub async fn cleanup_old_stats(&self, before_ts: i64) -> Result<i64, DaoError> {
        let count = sqlx::query_scalar!(
            r#"
            WITH deleted AS (
                DELETE FROM queue_stats
                WHERE collected_at < $1
                RETURNING id
            )
            SELECT count(*) FROM deleted
            "#,
            before_ts
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.unwrap_or(0))
    }

    /// Get the latest stats for all distinct queue names.
    #[instrument(skip(self))]
    pub async fn get_all_latest(&self) -> Result<Vec<QueueStats>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT ON (queue_name)
                id, queue_name, messages_total, messages_per_second, consumers_total, lag, collected_at
            FROM queue_stats
            ORDER BY queue_name, collected_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| QueueStats {
            id: r.id,
            queue_name: r.queue_name,
            messages_total: r.messages_total,
            messages_per_second: r.messages_per_second,
            consumers_total: r.consumers_total,
            lag: r.lag,
            collected_at: r.collected_at,
        }).collect())
    }
}
