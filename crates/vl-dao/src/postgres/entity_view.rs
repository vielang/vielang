use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{EntityView, EntityViewInfo};
use crate::{DaoError, PageData, PageLink};

pub struct EntityViewDao {
    pool: PgPool,
}

impl EntityViewDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    fn map_row(r: &EntityViewRow) -> EntityView {
        EntityView {
            id:               r.id,
            created_time:     r.created_time,
            tenant_id:        r.tenant_id,
            customer_id:      r.customer_id,
            entity_id:        r.entity_id,
            entity_type:      r.entity_type.clone(),
            name:             r.name.clone(),
            entity_view_type: r.ev_type.clone(),
            keys:             r.keys.clone(),
            start_ts:         r.start_ts,
            end_ts:           r.end_ts,
            additional_info:  r.additional_info.clone(),
            external_id:      r.external_id,
            version:          r.version,
        }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<EntityView>, DaoError> {
        let row = sqlx::query_as!(
            EntityViewRow,
            r#"
            SELECT id, created_time, tenant_id, customer_id,
                   entity_id, entity_type, name,
                   type AS ev_type,
                   keys, start_ts, end_ts,
                   additional_info, external_id, version
            FROM entity_view WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(Self::map_row))
    }

    /// Tìm theo tên trong scope tenant
    #[instrument(skip(self))]
    pub async fn find_by_tenant_and_name(
        &self,
        tenant_id: Uuid,
        name: &str,
    ) -> Result<Option<EntityView>, DaoError> {
        let row = sqlx::query_as!(
            EntityViewRow,
            r#"
            SELECT id, created_time, tenant_id, customer_id,
                   entity_id, entity_type, name,
                   type AS ev_type,
                   keys, start_ts, end_ts,
                   additional_info, external_id, version
            FROM entity_view WHERE tenant_id = $1 AND name = $2
            "#,
            tenant_id, name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(Self::map_row))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        ev_type: Option<&str>,
        page_link: &PageLink,
    ) -> Result<PageData<EntityView>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM entity_view
               WHERE tenant_id = $1
               AND ($2::text IS NULL OR type = $2)
               AND ($3::text IS NULL OR LOWER(name) LIKE LOWER($3))"#,
            tenant_id, ev_type, text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            EntityViewRow,
            r#"
            SELECT id, created_time, tenant_id, customer_id,
                   entity_id, entity_type, name,
                   type AS ev_type,
                   keys, start_ts, end_ts,
                   additional_info, external_id, version
            FROM entity_view
            WHERE tenant_id = $1
            AND ($2::text IS NULL OR type = $2)
            AND ($3::text IS NULL OR LOWER(name) LIKE LOWER($3))
            ORDER BY created_time DESC
            LIMIT $4 OFFSET $5
            "#,
            tenant_id, ev_type, text_search,
            page_link.page_size, page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.iter().map(Self::map_row).collect();
        Ok(PageData::new(data, total, page_link))
    }

    /// find_by_tenant kèm customer info (cho EntityViewInfo)
    #[instrument(skip(self))]
    pub async fn find_infos_by_tenant(
        &self,
        tenant_id: Uuid,
        ev_type: Option<&str>,
        page_link: &PageLink,
    ) -> Result<PageData<EntityViewInfo>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM entity_view
               WHERE tenant_id = $1
               AND ($2::text IS NULL OR type = $2)
               AND ($3::text IS NULL OR LOWER(name) LIKE LOWER($3))"#,
            tenant_id, ev_type, text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT ev.id, ev.created_time, ev.tenant_id, ev.customer_id,
                   ev.entity_id, ev.entity_type, ev.name,
                   ev.type AS ev_type,
                   ev.keys, ev.start_ts, ev.end_ts,
                   ev.additional_info, ev.external_id, ev.version,
                   c.title AS "customer_title: Option<String>",
                   c.is_public AS "customer_is_public: Option<bool>"
            FROM entity_view ev
            LEFT JOIN customer c ON c.id = ev.customer_id
            WHERE ev.tenant_id = $1
            AND ($2::text IS NULL OR ev.type = $2)
            AND ($3::text IS NULL OR LOWER(ev.name) LIKE LOWER($3))
            ORDER BY ev.created_time DESC
            LIMIT $4 OFFSET $5
            "#,
            tenant_id, ev_type, text_search,
            page_link.page_size, page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| {
            let ev = EntityView {
                id:               r.id,
                created_time:     r.created_time,
                tenant_id:        r.tenant_id,
                customer_id:      r.customer_id,
                entity_id:        r.entity_id,
                entity_type:      r.entity_type,
                name:             r.name,
                entity_view_type: r.ev_type,
                keys:             r.keys,
                start_ts:         r.start_ts,
                end_ts:           r.end_ts,
                additional_info:  r.additional_info,
                external_id:      r.external_id,
                version:          r.version,
            };
            EntityViewInfo {
                entity_view:        ev,
                customer_title:     r.customer_title,
                customer_is_public: r.customer_is_public.unwrap_or(false),
            }
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    #[instrument(skip(self))]
    pub async fn find_by_customer(
        &self,
        tenant_id: Uuid,
        customer_id: Uuid,
        ev_type: Option<&str>,
        page_link: &PageLink,
    ) -> Result<PageData<EntityView>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM entity_view
               WHERE tenant_id = $1 AND customer_id = $2
               AND ($3::text IS NULL OR type = $3)
               AND ($4::text IS NULL OR LOWER(name) LIKE LOWER($4))"#,
            tenant_id, customer_id, ev_type, text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            EntityViewRow,
            r#"
            SELECT id, created_time, tenant_id, customer_id,
                   entity_id, entity_type, name,
                   type AS ev_type,
                   keys, start_ts, end_ts,
                   additional_info, external_id, version
            FROM entity_view
            WHERE tenant_id = $1 AND customer_id = $2
            AND ($3::text IS NULL OR type = $3)
            AND ($4::text IS NULL OR LOWER(name) LIKE LOWER($4))
            ORDER BY created_time DESC
            LIMIT $5 OFFSET $6
            "#,
            tenant_id, customer_id, ev_type, text_search,
            page_link.page_size, page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.iter().map(Self::map_row).collect();
        Ok(PageData::new(data, total, page_link))
    }

    #[instrument(skip(self))]
    pub async fn find_infos_by_customer(
        &self,
        tenant_id: Uuid,
        customer_id: Uuid,
        ev_type: Option<&str>,
        page_link: &PageLink,
    ) -> Result<PageData<EntityViewInfo>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM entity_view
               WHERE tenant_id = $1 AND customer_id = $2
               AND ($3::text IS NULL OR type = $3)
               AND ($4::text IS NULL OR LOWER(name) LIKE LOWER($4))"#,
            tenant_id, customer_id, ev_type, text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT ev.id, ev.created_time, ev.tenant_id, ev.customer_id,
                   ev.entity_id, ev.entity_type, ev.name,
                   ev.type AS ev_type,
                   ev.keys, ev.start_ts, ev.end_ts,
                   ev.additional_info, ev.external_id, ev.version,
                   c.title AS "customer_title: Option<String>",
                   c.is_public AS "customer_is_public: Option<bool>"
            FROM entity_view ev
            LEFT JOIN customer c ON c.id = ev.customer_id
            WHERE ev.tenant_id = $1 AND ev.customer_id = $2
            AND ($3::text IS NULL OR ev.type = $3)
            AND ($4::text IS NULL OR LOWER(ev.name) LIKE LOWER($4))
            ORDER BY ev.created_time DESC
            LIMIT $5 OFFSET $6
            "#,
            tenant_id, customer_id, ev_type, text_search,
            page_link.page_size, page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| {
            let ev = EntityView {
                id:               r.id,
                created_time:     r.created_time,
                tenant_id:        r.tenant_id,
                customer_id:      r.customer_id,
                entity_id:        r.entity_id,
                entity_type:      r.entity_type,
                name:             r.name,
                entity_view_type: r.ev_type,
                keys:             r.keys,
                start_ts:         r.start_ts,
                end_ts:           r.end_ts,
                additional_info:  r.additional_info,
                external_id:      r.external_id,
                version:          r.version,
            };
            EntityViewInfo {
                entity_view:       ev,
                customer_title:    r.customer_title,
                customer_is_public: r.customer_is_public.unwrap_or(false),
            }
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    /// Lấy distinct types trong tenant
    #[instrument(skip(self))]
    pub async fn find_types_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<String>, DaoError> {
        let rows = sqlx::query!(
            "SELECT DISTINCT type FROM entity_view WHERE tenant_id = $1 ORDER BY type",
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.r#type).collect())
    }

    #[instrument(skip(self))]
    pub async fn save(&self, ev: &EntityView) -> Result<EntityView, DaoError> {
        let additional_info = ev.additional_info.as_ref().map(|v| v.clone());

        sqlx::query!(
            r#"
            INSERT INTO entity_view (
                id, created_time, tenant_id, customer_id,
                entity_id, entity_type, name, type,
                keys, start_ts, end_ts,
                additional_info, external_id, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
            ON CONFLICT (id) DO UPDATE SET
                name           = EXCLUDED.name,
                type           = EXCLUDED.type,
                customer_id    = EXCLUDED.customer_id,
                entity_id      = EXCLUDED.entity_id,
                entity_type    = EXCLUDED.entity_type,
                keys           = EXCLUDED.keys,
                start_ts       = EXCLUDED.start_ts,
                end_ts         = EXCLUDED.end_ts,
                additional_info = EXCLUDED.additional_info,
                external_id    = EXCLUDED.external_id,
                version        = entity_view.version + 1
            "#,
            ev.id,
            ev.created_time,
            ev.tenant_id,
            ev.customer_id,
            ev.entity_id,
            ev.entity_type,
            ev.name,
            ev.entity_view_type,
            ev.keys,
            ev.start_ts,
            ev.end_ts,
            additional_info,
            ev.external_id,
            ev.version,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(ev.id).await?.ok_or(DaoError::NotFound)
    }

    /// Assign entity view tới customer
    #[instrument(skip(self))]
    pub async fn assign_to_customer(
        &self,
        ev_id: Uuid,
        customer_id: Uuid,
    ) -> Result<EntityView, DaoError> {
        sqlx::query!(
            "UPDATE entity_view SET customer_id = $1, version = version + 1 WHERE id = $2",
            customer_id, ev_id
        )
        .execute(&self.pool)
        .await?;

        self.find_by_id(ev_id).await?.ok_or(DaoError::NotFound)
    }

    /// Unassign entity view khỏi customer
    #[instrument(skip(self))]
    pub async fn unassign_from_customer(&self, ev_id: Uuid) -> Result<EntityView, DaoError> {
        sqlx::query!(
            "UPDATE entity_view SET customer_id = NULL, version = version + 1 WHERE id = $1",
            ev_id
        )
        .execute(&self.pool)
        .await?;

        self.find_by_id(ev_id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM entity_view WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }
}

// ── Internal query struct ─────────────────────────────────────────────────────

struct EntityViewRow {
    id:              Uuid,
    created_time:    i64,
    tenant_id:       Uuid,
    customer_id:     Option<Uuid>,
    entity_id:       Uuid,
    entity_type:     String,
    name:            String,
    ev_type:         String,
    keys:            Option<serde_json::Value>,
    start_ts:        i64,
    end_ts:          i64,
    additional_info: Option<serde_json::Value>,
    external_id:     Option<Uuid>,
    version:         i64,
}
