use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{NotificationRequest, NotificationStatus};
use crate::{DaoError, PageData, PageLink};

pub struct NotificationRequestDao {
    pool: PgPool,
}

impl NotificationRequestDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<NotificationRequest>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, rule_id, template_id,
                   info, status, error, sent_time, version
            FROM notification_request WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| NotificationRequest {
            id:           r.id,
            created_time: r.created_time,
            tenant_id:    r.tenant_id,
            rule_id:      r.rule_id,
            template_id:  r.template_id,
            info:         r.info,
            status:       NotificationStatus::from_str(&r.status)
                .unwrap_or(NotificationStatus::Scheduled),
            error:        r.error,
            sent_time:    r.sent_time,
            version:      r.version,
        }))
    }

    #[instrument(skip(self))]
    pub async fn save(&self, req: &NotificationRequest) -> Result<NotificationRequest, DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO notification_request (
                id, created_time, tenant_id, rule_id, template_id,
                info, status, error, sent_time, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            ON CONFLICT (id) DO UPDATE SET
                status    = EXCLUDED.status,
                error     = EXCLUDED.error,
                sent_time = EXCLUDED.sent_time,
                version   = notification_request.version + 1
            "#,
            req.id,
            req.created_time,
            req.tenant_id,
            req.rule_id,
            req.template_id,
            req.info,
            req.status.as_str(),
            req.error,
            req.sent_time,
            req.version,
        )
        .execute(&self.pool)
        .await?;

        self.find_by_id(req.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn update_status(
        &self,
        id: Uuid,
        status: NotificationStatus,
        error: Option<String>,
        sent_time: Option<i64>,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            UPDATE notification_request
            SET status = $2, error = $3, sent_time = $4, version = version + 1
            WHERE id = $1
            "#,
            id,
            status.as_str(),
            error,
            sent_time,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<NotificationRequest>, DaoError> {
        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM notification_request WHERE tenant_id = $1",
            tenant_id,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, rule_id, template_id,
                   info, status, error, sent_time, version
            FROM notification_request
            WHERE tenant_id = $1
            ORDER BY created_time DESC
            LIMIT $2 OFFSET $3
            "#,
            tenant_id,
            page_link.page_size,
            page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| NotificationRequest {
            id:           r.id,
            created_time: r.created_time,
            tenant_id:    r.tenant_id,
            rule_id:      r.rule_id,
            template_id:  r.template_id,
            info:         r.info,
            status:       NotificationStatus::from_str(&r.status)
                .unwrap_or(NotificationStatus::Scheduled),
            error:        r.error,
            sent_time:    r.sent_time,
            version:      r.version,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }
}
