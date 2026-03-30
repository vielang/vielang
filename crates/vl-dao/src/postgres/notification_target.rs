use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::NotificationTarget;
use crate::{DaoError, PageData, PageLink};

pub struct NotificationTargetDao {
    pool: PgPool,
}

impl NotificationTargetDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<NotificationTarget>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, name, target_type, target_config, version
            FROM notification_target WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| NotificationTarget {
            id:            r.id,
            created_time:  r.created_time,
            tenant_id:     r.tenant_id,
            name:          r.name,
            target_type:   r.target_type,
            target_config: r.target_config,
            version:       r.version,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_by_ids(&self, ids: &[Uuid]) -> Result<Vec<NotificationTarget>, DaoError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, name, target_type, target_config, version
            FROM notification_target WHERE id = ANY($1)
            "#,
            ids
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| NotificationTarget {
            id:            r.id,
            created_time:  r.created_time,
            tenant_id:     r.tenant_id,
            name:          r.name,
            target_type:   r.target_type,
            target_config: r.target_config,
            version:       r.version,
        }).collect())
    }

    #[instrument(skip(self))]
    pub async fn save(&self, t: &NotificationTarget) -> Result<NotificationTarget, DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO notification_target (
                id, created_time, tenant_id, name, target_type, target_config, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7)
            ON CONFLICT (id) DO UPDATE SET
                name          = EXCLUDED.name,
                target_type   = EXCLUDED.target_type,
                target_config = EXCLUDED.target_config,
                version       = notification_target.version + 1
            "#,
            t.id,
            t.created_time,
            t.tenant_id,
            t.name,
            t.target_type,
            t.target_config,
            t.version,
        )
        .execute(&self.pool)
        .await?;

        self.find_by_id(t.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!(
            "DELETE FROM notification_target WHERE id = $1",
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
    ) -> Result<PageData<NotificationTarget>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM notification_target
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
            SELECT id, created_time, tenant_id, name, target_type, target_config, version
            FROM notification_target
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

        let data = rows.into_iter().map(|r| NotificationTarget {
            id:            r.id,
            created_time:  r.created_time,
            tenant_id:     r.tenant_id,
            name:          r.name,
            target_type:   r.target_type,
            target_config: r.target_config,
            version:       r.version,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }
}
