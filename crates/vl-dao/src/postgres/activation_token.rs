use sqlx::PgPool;
use uuid::Uuid;
use crate::error::DaoError;

pub struct ActivationRecord {
    pub user_id: Uuid,
}

pub struct ActivationTokenDao {
    pool: PgPool,
}

impl ActivationTokenDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Store a new activation token (TTL in seconds)
    pub async fn store(&self, user_id: Uuid, token: &str, ttl_secs: i64) -> Result<(), DaoError> {
        let now = chrono::Utc::now().timestamp_millis();
        let expires_at = now + ttl_secs * 1000;
        sqlx::query!(
            "INSERT INTO user_activation_token (token, user_id, expires_at, created_at) VALUES ($1, $2, $3, $4)",
            token, user_id, expires_at, now
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Consume a token atomically — returns None if invalid/expired/used
    pub async fn consume(&self, token: &str) -> Result<Option<ActivationRecord>, DaoError> {
        let now = chrono::Utc::now().timestamp_millis();
        let row = sqlx::query!(
            "UPDATE user_activation_token SET used = TRUE WHERE token = $1 AND used = FALSE AND expires_at > $2 RETURNING user_id",
            token, now
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| ActivationRecord { user_id: r.user_id }))
    }

    /// Delete expired or used tokens (called from cleanup job)
    pub async fn cleanup_expired(&self) -> Result<u64, DaoError> {
        let now = chrono::Utc::now().timestamp_millis();
        let result = sqlx::query!(
            "DELETE FROM user_activation_token WHERE used = TRUE OR expires_at < $1",
            now
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}
