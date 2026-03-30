use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use crate::DaoError;

#[derive(Debug, Clone)]
pub struct NotificationChannelSettings {
    pub id:           Uuid,
    pub tenant_id:    Uuid,
    pub channel:      String,
    pub config:       serde_json::Value,
    pub enabled:      bool,
    pub created_time: i64,
}

pub struct NotificationChannelSettingsDao {
    pool: PgPool,
}

impl NotificationChannelSettingsDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Lấy tất cả channel settings cho tenant
    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<NotificationChannelSettings>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, tenant_id, channel, config::text as config_str, enabled, created_time
            FROM notification_channel_settings
            WHERE tenant_id = $1
            ORDER BY channel
            "#,
            tenant_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| NotificationChannelSettings {
            id:           r.id,
            tenant_id:    r.tenant_id,
            channel:      r.channel,
            config:       r.config_str
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or(serde_json::Value::Null),
            enabled:      r.enabled,
            created_time: r.created_time,
        }).collect())
    }

    /// Lấy settings cho một channel cụ thể của tenant
    #[instrument(skip(self))]
    pub async fn find_by_tenant_and_channel(
        &self,
        tenant_id: Uuid,
        channel:   &str,
    ) -> Result<Option<NotificationChannelSettings>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, tenant_id, channel, config::text as config_str, enabled, created_time
            FROM notification_channel_settings
            WHERE tenant_id = $1 AND channel = $2
            "#,
            tenant_id,
            channel,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| NotificationChannelSettings {
            id:           r.id,
            tenant_id:    r.tenant_id,
            channel:      r.channel,
            config:       r.config_str
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or(serde_json::Value::Null),
            enabled:      r.enabled,
            created_time: r.created_time,
        }))
    }

    /// Upsert channel settings (UNIQUE constraint trên tenant_id + channel)
    #[instrument(skip(self))]
    pub async fn upsert(
        &self,
        settings: &NotificationChannelSettings,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO notification_channel_settings
                (id, tenant_id, channel, config, enabled, created_time)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (tenant_id, channel) DO UPDATE SET
                config  = EXCLUDED.config,
                enabled = EXCLUDED.enabled
            "#,
            settings.id,
            settings.tenant_id,
            settings.channel,
            settings.config as _,
            settings.enabled,
            settings.created_time,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn delete(
        &self,
        tenant_id: Uuid,
        channel:   &str,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            "DELETE FROM notification_channel_settings WHERE tenant_id = $1 AND channel = $2",
            tenant_id,
            channel,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
