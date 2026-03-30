use sqlx::{PgPool, Row};
use tracing::instrument;
use uuid::Uuid;

use crate::DaoError;

// ── Domain types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct QueueMessage {
    pub id:           Uuid,
    pub topic:        String,
    pub partition_id: i32,
    pub offset_value: i64,
    pub payload:      Vec<u8>,
    pub headers:      Option<serde_json::Value>,
    pub created_time: i64,
}

/// Input for a single message to persist.
pub struct NewQueueMessage<'a> {
    pub topic:        &'a str,
    pub partition_id: i32,
    pub payload:      &'a [u8],
    pub headers:      Option<serde_json::Value>,
    pub created_time: i64,
}

// ── DAO ───────────────────────────────────────────────────────────────────────

pub struct QueueMessageDao {
    pool: PgPool,
}

impl QueueMessageDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Persist a batch of messages. Returns the assigned offset_value for each message.
    /// Uses non-macro queries because queue_message is created in migration 049.
    #[instrument(skip(self, messages))]
    pub async fn save_batch(
        &self,
        messages: &[NewQueueMessage<'_>],
    ) -> Result<Vec<i64>, DaoError> {
        let mut offsets = Vec::with_capacity(messages.len());
        for msg in messages {
            let row = sqlx::query(
                r#"INSERT INTO queue_message
                   (topic, partition_id, payload, headers, created_time)
                   VALUES ($1, $2, $3, $4, $5)
                   RETURNING offset_value"#,
            )
            .bind(msg.topic)
            .bind(msg.partition_id)
            .bind(msg.payload)
            .bind(&msg.headers)
            .bind(msg.created_time)
            .fetch_one(&self.pool)
            .await
            .map_err(DaoError::Database)?;

            offsets.push(row.try_get::<i64, _>("offset_value").map_err(DaoError::Database)?);
        }
        Ok(offsets)
    }

    /// Poll unacked messages from a topic after a known offset.
    /// Stamps consumer_id on the rows so they are attributed to this consumer.
    #[instrument(skip(self))]
    pub async fn poll(
        &self,
        topic:        &str,
        consumer_id:  &str,
        after_offset: i64,
        limit:        i32,
    ) -> Result<Vec<QueueMessage>, DaoError> {
        // First stamp consumer_id on the rows we will return (claim them)
        // Then fetch them — two statements as per CLAUDE.md (no semicolons in query!)
        sqlx::query(
            r#"UPDATE queue_message
               SET consumer_id = $1
               WHERE id IN (
                   SELECT id FROM queue_message
                   WHERE topic = $2
                     AND acked_time IS NULL
                     AND offset_value > $3
                   ORDER BY offset_value ASC
                   LIMIT $4
               )
               AND acked_time IS NULL"#,
        )
        .bind(consumer_id)
        .bind(topic)
        .bind(after_offset)
        .bind(limit as i64)
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        let rows = sqlx::query(
            r#"SELECT id, topic, partition_id, offset_value,
                      payload, headers, created_time
               FROM queue_message
               WHERE topic = $1
                 AND consumer_id = $2
                 AND acked_time IS NULL
                 AND offset_value > $3
               ORDER BY offset_value ASC
               LIMIT $4"#,
        )
        .bind(topic)
        .bind(consumer_id)
        .bind(after_offset)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        rows.into_iter()
            .map(|r| {
                Ok(QueueMessage {
                    id:           r.try_get("id").map_err(DaoError::Database)?,
                    topic:        r.try_get("topic").map_err(DaoError::Database)?,
                    partition_id: r.try_get("partition_id").map_err(DaoError::Database)?,
                    offset_value: r.try_get("offset_value").map_err(DaoError::Database)?,
                    payload:      r.try_get("payload").map_err(DaoError::Database)?,
                    headers:      r.try_get("headers").map_err(DaoError::Database)?,
                    created_time: r.try_get("created_time").map_err(DaoError::Database)?,
                })
            })
            .collect()
    }

    /// Mark a batch of messages as acknowledged (processed).
    /// Only acks messages owned by the given consumer_id.
    #[instrument(skip(self, ids))]
    pub async fn ack_batch(
        &self,
        ids:         &[Uuid],
        consumer_id: &str,
    ) -> Result<u64, DaoError> {
        let now = chrono::Utc::now().timestamp_millis();
        let result = sqlx::query(
            r#"UPDATE queue_message
               SET acked_time = $1
               WHERE id = ANY($2)
                 AND consumer_id = $3
                 AND acked_time IS NULL"#,
        )
        .bind(now)
        .bind(ids)
        .bind(consumer_id)
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        Ok(result.rows_affected())
    }

    /// Delete messages that have been acked and are older than older_than_ms.
    #[instrument(skip(self))]
    pub async fn cleanup_acked(&self, older_than_ms: i64) -> Result<u64, DaoError> {
        let result = sqlx::query(
            "DELETE FROM queue_message WHERE acked_time IS NOT NULL AND created_time < $1",
        )
        .bind(older_than_ms)
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        Ok(result.rows_affected())
    }

    /// Count unacked messages for a topic.
    #[instrument(skip(self))]
    pub async fn count_pending(&self, topic: &str) -> Result<i64, DaoError> {
        let row = sqlx::query(
            "SELECT COUNT(*) as cnt FROM queue_message WHERE topic = $1 AND acked_time IS NULL",
        )
        .bind(topic)
        .fetch_one(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        row.try_get::<i64, _>("cnt").map_err(DaoError::Database)
    }

    /// Xóa các messages đã ack và cũ hơn `retention_hours` giờ.
    /// Dùng cho background cleanup job trong main.rs.
    #[instrument(skip(self))]
    pub async fn cleanup_old_acked(&self, retention_hours: u64) -> Result<u64, DaoError> {
        let cutoff_ms = chrono::Utc::now().timestamp_millis()
            - (retention_hours as i64) * 3_600_000;
        let result = sqlx::query(
            "DELETE FROM queue_message WHERE acked_time IS NOT NULL AND created_time < $1",
        )
        .bind(cutoff_ms)
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        Ok(result.rows_affected())
    }

    /// Count total bytes of unacked payload for a topic.
    #[instrument(skip(self))]
    pub async fn bytes_pending(&self, topic: &str) -> Result<i64, DaoError> {
        let row = sqlx::query(
            r#"SELECT COALESCE(SUM(length(payload)), 0) AS bytes
               FROM queue_message WHERE topic = $1 AND acked_time IS NULL"#,
        )
        .bind(topic)
        .fetch_one(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        row.try_get::<i64, _>("bytes").map_err(DaoError::Database)
    }
}
