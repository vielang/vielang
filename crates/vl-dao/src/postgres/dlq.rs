use sqlx::{PgPool, Row};
use tracing::instrument;
use uuid::Uuid;

use crate::DaoError;

// ── Domain types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DlqMessage {
    pub id:            Uuid,
    pub topic:         String,
    pub payload:       Vec<u8>,
    pub error_message: Option<String>,
    pub retry_count:   i32,
    pub status:        String,
    pub created_at:    i64,
    pub updated_at:    i64,
}

// ── DAO ───────────────────────────────────────────────────────────────────────

pub struct DlqDao {
    pool: PgPool,
}

impl DlqDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Lưu một message thất bại vào DLQ.
    #[instrument(skip(self, payload))]
    pub async fn store(
        &self,
        topic:         &str,
        payload:       &[u8],
        error_message: Option<&str>,
    ) -> Result<Uuid, DaoError> {
        let now = chrono::Utc::now().timestamp_millis();
        let row = sqlx::query(
            r#"INSERT INTO dlq_messages (topic, payload, error_message, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING id"#,
        )
        .bind(topic)
        .bind(payload)
        .bind(error_message)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        row.try_get::<Uuid, _>("id").map_err(DaoError::Database)
    }

    /// List PENDING DLQ messages (phân trang).
    #[instrument(skip(self))]
    pub async fn list_pending(
        &self,
        limit:  i64,
        offset: i64,
    ) -> Result<(Vec<DlqMessage>, i64), DaoError> {
        let total_row = sqlx::query(
            "SELECT COUNT(*) AS cnt FROM dlq_messages WHERE status = 'PENDING'",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DaoError::Database)?;
        let total: i64 = total_row.try_get("cnt").map_err(DaoError::Database)?;

        let rows = sqlx::query(
            r#"SELECT id, topic, payload, error_message, retry_count, status, created_at, updated_at
               FROM dlq_messages
               WHERE status = 'PENDING'
               ORDER BY created_at ASC
               LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        let msgs = rows
            .into_iter()
            .map(|r| {
                Ok(DlqMessage {
                    id:            r.try_get("id").map_err(DaoError::Database)?,
                    topic:         r.try_get("topic").map_err(DaoError::Database)?,
                    payload:       r.try_get("payload").map_err(DaoError::Database)?,
                    error_message: r.try_get("error_message").map_err(DaoError::Database)?,
                    retry_count:   r.try_get("retry_count").map_err(DaoError::Database)?,
                    status:        r.try_get("status").map_err(DaoError::Database)?,
                    created_at:    r.try_get("created_at").map_err(DaoError::Database)?,
                    updated_at:    r.try_get("updated_at").map_err(DaoError::Database)?,
                })
            })
            .collect::<Result<Vec<_>, DaoError>>()?;

        Ok((msgs, total))
    }

    /// Lấy một message theo ID.
    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<DlqMessage>, DaoError> {
        let row = sqlx::query(
            r#"SELECT id, topic, payload, error_message, retry_count, status, created_at, updated_at
               FROM dlq_messages WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        row.map(|r| {
            Ok(DlqMessage {
                id:            r.try_get("id").map_err(DaoError::Database)?,
                topic:         r.try_get("topic").map_err(DaoError::Database)?,
                payload:       r.try_get("payload").map_err(DaoError::Database)?,
                error_message: r.try_get("error_message").map_err(DaoError::Database)?,
                retry_count:   r.try_get("retry_count").map_err(DaoError::Database)?,
                status:        r.try_get("status").map_err(DaoError::Database)?,
                created_at:    r.try_get("created_at").map_err(DaoError::Database)?,
                updated_at:    r.try_get("updated_at").map_err(DaoError::Database)?,
            })
        })
        .transpose()
    }

    /// Đánh dấu message đã được replay.
    #[instrument(skip(self))]
    pub async fn mark_replayed(&self, id: Uuid) -> Result<bool, DaoError> {
        let now = chrono::Utc::now().timestamp_millis();
        let result = sqlx::query(
            r#"UPDATE dlq_messages
               SET status = 'REPLAYED', updated_at = $1, retry_count = retry_count + 1
               WHERE id = $2 AND status = 'PENDING'"#,
        )
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        Ok(result.rows_affected() > 0)
    }

    /// Xóa toàn bộ PENDING messages (purge DLQ).
    #[instrument(skip(self))]
    pub async fn purge_pending(&self) -> Result<u64, DaoError> {
        let result = sqlx::query(
            "DELETE FROM dlq_messages WHERE status = 'PENDING'",
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        Ok(result.rows_affected())
    }

    /// Đếm PENDING messages theo topic (cho monitoring).
    #[instrument(skip(self))]
    pub async fn count_pending(&self, topic: &str) -> Result<i64, DaoError> {
        let row = sqlx::query(
            "SELECT COUNT(*) AS cnt FROM dlq_messages WHERE topic = $1 AND status = 'PENDING'",
        )
        .bind(topic)
        .fetch_one(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        row.try_get::<i64, _>("cnt").map_err(DaoError::Database)
    }
}
