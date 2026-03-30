use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use crate::DaoError;

// ── UserSettingsDao ──────────────────────────────────────────────────────────

pub struct UserSettingsDao {
    pool: PgPool,
}

impl UserSettingsDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    /// Find user settings JSON by user_id and type key.
    #[instrument(skip(self))]
    pub async fn find(
        &self,
        user_id: Uuid,
        type_: &str,
    ) -> Result<Option<serde_json::Value>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT settings FROM user_settings
               WHERE user_id = $1 AND type = $2"#,
            user_id,
            type_,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.settings))
    }

    /// UPSERT user settings JSON for a given (user_id, type) pair.
    #[instrument(skip(self, settings))]
    pub async fn save(
        &self,
        user_id: Uuid,
        type_: &str,
        settings: &serde_json::Value,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO user_settings (user_id, type, settings)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, type) DO UPDATE SET
                settings = EXCLUDED.settings
            "#,
            user_id,
            type_,
            settings,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, user_id: Uuid, type_: &str) -> Result<(), DaoError> {
        sqlx::query!(
            "DELETE FROM user_settings WHERE user_id = $1 AND type = $2",
            user_id,
            type_,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

// ── UserAuthSettingsRow ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct UserAuthSettingsRow {
    pub id: Uuid,
    pub created_time: i64,
    pub user_id: Uuid,
    pub two_fa_settings: Option<String>,
}

// ── UserAuthSettingsDao ──────────────────────────────────────────────────────

pub struct UserAuthSettingsDao {
    pool: PgPool,
}

impl UserAuthSettingsDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    #[instrument(skip(self))]
    pub async fn find_by_user(&self, user_id: Uuid) -> Result<Option<UserAuthSettingsRow>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, user_id, two_fa_settings
            FROM user_auth_settings
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| UserAuthSettingsRow {
            id:              r.id,
            created_time:    r.created_time,
            user_id:         r.user_id,
            two_fa_settings: r.two_fa_settings,
        }))
    }

    /// UPSERT on user_id — stores 2FA settings JSON string.
    #[instrument(skip(self, two_fa_settings))]
    pub async fn save(&self, user_id: Uuid, two_fa_settings: &str) -> Result<(), DaoError> {
        let now = chrono::Utc::now().timestamp_millis();
        let id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO user_auth_settings (id, created_time, user_id, two_fa_settings)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id) DO UPDATE SET
                two_fa_settings = EXCLUDED.two_fa_settings
            "#,
            id,
            now,
            user_id,
            two_fa_settings,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn delete_by_user(&self, user_id: Uuid) -> Result<(), DaoError> {
        sqlx::query!(
            "DELETE FROM user_auth_settings WHERE user_id = $1",
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
