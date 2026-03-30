use sqlx::PgPool;
use uuid::Uuid;

use crate::error::DaoError;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct NotificationDelivery {
    pub id:              Uuid,
    pub notification_id: Uuid,
    pub channel_type:    String,
    pub recipient:       String,
    pub status:          String,
    pub error:           Option<String>,
    pub sent_at:         Option<i64>,
    pub created_time:    i64,
}

pub struct NotificationDeliveryDao {
    pool: PgPool,
}

impl NotificationDeliveryDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record(
        &self,
        notification_id: Uuid,
        channel_type:    &str,
        recipient:       &str,
        status:          &str,
        error:           Option<&str>,
        sent_at:         Option<i64>,
        created_time:    i64,
    ) -> Result<Uuid, DaoError> {
        let id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO notification_delivery
                (id, notification_id, channel_type, recipient, status, error, sent_at, created_time)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            id,
            notification_id,
            channel_type,
            recipient,
            status,
            error,
            sent_at,
            created_time,
        )
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn update_status(
        &self,
        id:     Uuid,
        status: &str,
        error:  Option<&str>,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            "UPDATE notification_delivery SET status = $1, error = $2 WHERE id = $3",
            status,
            error,
            id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_notification(
        &self,
        notification_id: Uuid,
    ) -> Result<Vec<NotificationDelivery>, DaoError> {
        let rows = sqlx::query_as!(
            NotificationDelivery,
            r#"
            SELECT id, notification_id, channel_type, recipient, status,
                   error, sent_at, created_time
            FROM notification_delivery
            WHERE notification_id = $1
            ORDER BY created_time ASC
            "#,
            notification_id,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
}
