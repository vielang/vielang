use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{TwoFactorAuthSettings, TwoFactorProvider};

use crate::DaoError;

pub struct TwoFactorAuthDao {
    pool: PgPool,
}

impl TwoFactorAuthDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    pub async fn find_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Option<TwoFactorAuthSettings>, DaoError> {
        let r = sqlx::query!(
            "SELECT user_id, provider, enabled, secret, backup_codes, verified
             FROM two_factor_auth_settings WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        Ok(r.map(|r| {
            let backup_codes: Vec<String> =
                serde_json::from_value(r.backup_codes).unwrap_or_default();
            TwoFactorAuthSettings {
                user_id:      r.user_id,
                provider:     TwoFactorProvider::from_str(&r.provider),
                enabled:      r.enabled,
                secret:       r.secret,
                backup_codes,
                verified:     r.verified,
            }
        }))
    }

    pub async fn save(&self, settings: &TwoFactorAuthSettings) -> Result<(), DaoError> {
        let backup_codes = serde_json::to_value(&settings.backup_codes).unwrap_or_default();

        sqlx::query!(
            "INSERT INTO two_factor_auth_settings (user_id, provider, enabled, secret, backup_codes, verified)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (user_id) DO UPDATE SET
                provider     = EXCLUDED.provider,
                enabled      = EXCLUDED.enabled,
                secret       = EXCLUDED.secret,
                backup_codes = EXCLUDED.backup_codes,
                verified     = EXCLUDED.verified",
            settings.user_id,
            settings.provider.as_str(),
            settings.enabled,
            settings.secret,
            backup_codes,
            settings.verified,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?;
        Ok(())
    }

    pub async fn delete(&self, user_id: Uuid) -> Result<(), DaoError> {
        sqlx::query!(
            "DELETE FROM two_factor_auth_settings WHERE user_id = $1",
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?;
        Ok(())
    }
}
