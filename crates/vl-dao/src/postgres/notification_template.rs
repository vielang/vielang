use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{NotificationTemplate, NotificationType};
use crate::{DaoError, PageData, PageLink};

pub struct NotificationTemplateDao {
    pool: PgPool,
}

impl NotificationTemplateDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<NotificationTemplate>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, name, notification_type,
                   subject_template, body_template, additional_config, enabled, version
            FROM notification_template WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| NotificationTemplate {
            id:                r.id,
            created_time:      r.created_time,
            tenant_id:         r.tenant_id,
            name:              r.name,
            notification_type: NotificationType::from_str(&r.notification_type)
                .unwrap_or(NotificationType::Webhook),
            subject_template:  r.subject_template,
            body_template:     r.body_template,
            additional_config: r.additional_config,
            enabled:           r.enabled,
            version:           r.version,
        }))
    }

    #[instrument(skip(self))]
    pub async fn save(&self, t: &NotificationTemplate) -> Result<NotificationTemplate, DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO notification_template (
                id, created_time, tenant_id, name, notification_type,
                subject_template, body_template, additional_config, enabled, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            ON CONFLICT (id) DO UPDATE SET
                name              = EXCLUDED.name,
                notification_type = EXCLUDED.notification_type,
                subject_template  = EXCLUDED.subject_template,
                body_template     = EXCLUDED.body_template,
                additional_config = EXCLUDED.additional_config,
                enabled           = EXCLUDED.enabled,
                version           = notification_template.version + 1
            "#,
            t.id,
            t.created_time,
            t.tenant_id,
            t.name,
            t.notification_type.as_str(),
            t.subject_template,
            t.body_template,
            t.additional_config,
            t.enabled,
            t.version,
        )
        .execute(&self.pool)
        .await?;

        self.find_by_id(t.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!(
            "DELETE FROM notification_template WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            Err(DaoError::NotFound)
        } else {
            Ok(())
        }
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<NotificationTemplate>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM notification_template
               WHERE tenant_id = $1
               AND ($2::text IS NULL OR LOWER(name) LIKE LOWER($2))"#,
            tenant_id,
            text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, name, notification_type,
                   subject_template, body_template, additional_config, enabled, version
            FROM notification_template
            WHERE tenant_id = $1
            AND ($2::text IS NULL OR LOWER(name) LIKE LOWER($2))
            ORDER BY created_time DESC
            LIMIT $3 OFFSET $4
            "#,
            tenant_id,
            text_search,
            page_link.page_size,
            page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| NotificationTemplate {
            id:                r.id,
            created_time:      r.created_time,
            tenant_id:         r.tenant_id,
            name:              r.name,
            notification_type: NotificationType::from_str(&r.notification_type)
                .unwrap_or(NotificationType::Webhook),
            subject_template:  r.subject_template,
            body_template:     r.body_template,
            additional_config: r.additional_config,
            enabled:           r.enabled,
            version:           r.version,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }
}
