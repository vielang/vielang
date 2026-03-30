use sqlx::PgPool;
use uuid::Uuid;
use vl_core::entities::NotificationInbox;
use crate::{DaoError, PageData, PageLink};

pub struct NotificationInboxDao {
    pool: PgPool,
}

impl NotificationInboxDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    pub async fn save(&self, n: &NotificationInbox) -> Result<NotificationInbox, DaoError> {
        let row = sqlx::query!(
            "INSERT INTO notification_inbox
             (id, tenant_id, recipient_user_id, subject, body, notification_type, severity,
              status, sent_time, read_time, additional_config)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
             ON CONFLICT (id) DO UPDATE SET
               status = EXCLUDED.status,
               read_time = EXCLUDED.read_time
             RETURNING id, tenant_id, recipient_user_id, subject, body,
                       notification_type, severity, status, sent_time, read_time, additional_config",
            n.id, n.tenant_id, n.recipient_user_id,
            n.subject, n.body, n.notification_type, n.severity,
            n.status, n.sent_time, n.read_time,
            n.additional_config
        ).fetch_one(&self.pool).await?;

        Ok(NotificationInbox {
            id:                row.id,
            tenant_id:         row.tenant_id,
            recipient_user_id: row.recipient_user_id,
            subject:           row.subject,
            body:              row.body,
            notification_type: row.notification_type,
            severity:          row.severity,
            status:            row.status,
            sent_time:         row.sent_time,
            read_time:         row.read_time,
            additional_config: row.additional_config,
        })
    }

    pub async fn find_by_user(&self, user_id: Uuid, page_link: &PageLink) -> Result<PageData<NotificationInbox>, DaoError> {
        let offset = page_link.page * page_link.page_size;
        let rows = sqlx::query!(
            "SELECT id, tenant_id, recipient_user_id, subject, body, notification_type,
                    severity, status, sent_time, read_time, additional_config
             FROM notification_inbox
             WHERE recipient_user_id = $1
             ORDER BY sent_time DESC
             LIMIT $2 OFFSET $3",
            user_id, page_link.page_size, offset
        ).fetch_all(&self.pool).await?;

        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM notification_inbox WHERE recipient_user_id = $1",
            user_id
        ).fetch_one(&self.pool).await?.unwrap_or(0);

        let data = rows.into_iter().map(|r| NotificationInbox {
            id:                r.id,
            tenant_id:         r.tenant_id,
            recipient_user_id: r.recipient_user_id,
            subject:           r.subject,
            body:              r.body,
            notification_type: r.notification_type,
            severity:          r.severity,
            status:            r.status,
            sent_time:         r.sent_time,
            read_time:         r.read_time,
            additional_config: r.additional_config,
        }).collect();

        let total_pages = if page_link.page_size == 0 { 0 } else {
            (total + page_link.page_size - 1) / page_link.page_size
        };
        Ok(PageData {
            data,
            total_pages,
            total_elements: total,
            has_next: (page_link.page + 1) * page_link.page_size < total,
        })
    }

    pub async fn mark_read(&self, id: Uuid, user_id: Uuid) -> Result<(), DaoError> {
        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query!(
            "UPDATE notification_inbox SET status = 'READ', read_time = $3
             WHERE id = $1 AND recipient_user_id = $2",
            id, user_id, now
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn mark_all_read(&self, user_id: Uuid) -> Result<(), DaoError> {
        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query!(
            "UPDATE notification_inbox SET status = 'READ', read_time = $2
             WHERE recipient_user_id = $1 AND status = 'SENT'",
            user_id, now
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<(), DaoError> {
        sqlx::query!(
            "DELETE FROM notification_inbox WHERE id = $1 AND recipient_user_id = $2",
            id, user_id
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn count_unread(&self, user_id: Uuid) -> Result<i64, DaoError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM notification_inbox WHERE recipient_user_id = $1 AND status = 'SENT'",
            user_id
        ).fetch_one(&self.pool).await?.unwrap_or(0);
        Ok(count)
    }
}
