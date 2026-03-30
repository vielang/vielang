use sqlx::PgPool;
use uuid::Uuid;
use crate::DaoError;

pub struct MobileSessionDao {
    pool: PgPool,
}

impl MobileSessionDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    /// Register or refresh a mobile session.
    /// `fcm_token` is the unique key per user per device — updates all metadata on conflict.
    pub async fn upsert(
        &self,
        user_id:      Uuid,
        fcm_token:    &str,
        platform:     &str,
        app_version:  Option<&str>,
        os:           Option<&str>,
        os_version:   Option<&str>,
        device_model: Option<&str>,
    ) -> Result<(), DaoError> {
        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query(
            r#"INSERT INTO mobile_session
                   (id, user_id, fcm_token, platform, app_version,
                    os, os_version, device_model, created_time, last_active)
               VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, $8, $8)
               ON CONFLICT (user_id, fcm_token) DO UPDATE SET
                   platform     = EXCLUDED.platform,
                   app_version  = EXCLUDED.app_version,
                   os           = EXCLUDED.os,
                   os_version   = EXCLUDED.os_version,
                   device_model = EXCLUDED.device_model,
                   last_active  = EXCLUDED.last_active"#,
        )
        .bind(user_id)
        .bind(fcm_token)
        .bind(platform)
        .bind(app_version)
        .bind(os)
        .bind(os_version)
        .bind(device_model)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?;
        Ok(())
    }

    pub async fn delete(&self, user_id: Uuid, fcm_token: &str) -> Result<(), DaoError> {
        sqlx::query!(
            "DELETE FROM mobile_session WHERE user_id = $1 AND fcm_token = $2",
            user_id, fcm_token
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_by_token(&self, fcm_token: &str) -> Result<(), DaoError> {
        sqlx::query!(
            "DELETE FROM mobile_session WHERE fcm_token = $1",
            fcm_token
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn find_tokens_for_users(&self, user_ids: &[Uuid]) -> Result<Vec<String>, DaoError> {
        if user_ids.is_empty() { return Ok(vec![]); }
        let rows = sqlx::query!(
            "SELECT fcm_token FROM mobile_session WHERE user_id = ANY($1)",
            user_ids
        ).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| r.fcm_token).collect())
    }

    /// Prune sessions that haven't been active for more than `older_than_ms` milliseconds.
    pub async fn prune_stale(&self, older_than_ms: i64) -> Result<u64, DaoError> {
        let cutoff = chrono::Utc::now().timestamp_millis() - older_than_ms;
        let res = sqlx::query(
            "DELETE FROM mobile_session WHERE last_active IS NOT NULL AND last_active < $1",
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?;
        Ok(res.rows_affected())
    }
}
