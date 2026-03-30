use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::WidgetType;
use crate::{DaoError, PageData, PageLink};

pub struct WidgetTypeDao {
    pool: PgPool,
}

impl WidgetTypeDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    fn map_row(r: WidgetTypeRow) -> WidgetType {
        WidgetType {
            id:           r.id,
            created_time: r.created_time,
            tenant_id:    r.tenant_id,
            fqn:          r.fqn,
            name:         r.name,
            descriptor:   r.descriptor,
            deprecated:   r.deprecated,
            scada:        r.scada,
            image:        r.image,
            description:  r.description,
            tags:         r.tags,
            external_id:  r.external_id,
            version:      r.version,
        }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<WidgetType>, DaoError> {
        let row = sqlx::query_as!(
            WidgetTypeRow,
            r#"
            SELECT id, created_time, tenant_id, fqn, name, descriptor,
                   deprecated, scada, image, description, tags, external_id, version
            FROM widget_type WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Self::map_row))
    }

    #[instrument(skip(self))]
    pub async fn find_by_fqn(&self, fqn: &str) -> Result<Option<WidgetType>, DaoError> {
        let row = sqlx::query_as!(
            WidgetTypeRow,
            r#"
            SELECT id, created_time, tenant_id, fqn, name, descriptor,
                   deprecated, scada, image, description, tags, external_id, version
            FROM widget_type WHERE fqn = $1
            "#,
            fqn
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Self::map_row))
    }

    /// List widget types by bundle id, paginated.
    #[instrument(skip(self))]
    pub async fn find_by_bundle(
        &self,
        bundle_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<WidgetType>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM widget_type wt
               JOIN widgets_bundle_widget wbw ON wbw.widget_type_id = wt.id
               WHERE wbw.widgets_bundle_id = $1
               AND ($2::text IS NULL OR LOWER(wt.name) LIKE LOWER($2))"#,
            bundle_id, text_search
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            WidgetTypeRow,
            r#"
            SELECT wt.id, wt.created_time, wt.tenant_id, wt.fqn, wt.name, wt.descriptor,
                   wt.deprecated, wt.scada, wt.image, wt.description, wt.tags, wt.external_id, wt.version
            FROM widget_type wt
            JOIN widgets_bundle_widget wbw ON wbw.widget_type_id = wt.id
            WHERE wbw.widgets_bundle_id = $1
            AND ($2::text IS NULL OR LOWER(wt.name) LIKE LOWER($2))
            ORDER BY wbw.widget_type_order, wt.name
            LIMIT $3 OFFSET $4
            "#,
            bundle_id, text_search,
            page_link.page_size, page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(Self::map_row).collect();
        Ok(PageData::new(data, total, page_link))
    }

    /// List visible to tenant (system + own), paginated.
    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<WidgetType>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM widget_type
               WHERE (tenant_id IS NULL OR tenant_id = $1)
               AND ($2::text IS NULL OR LOWER(name) LIKE LOWER($2))"#,
            tenant_id, text_search
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            WidgetTypeRow,
            r#"
            SELECT id, created_time, tenant_id, fqn, name, descriptor,
                   deprecated, scada, image, description, tags, external_id, version
            FROM widget_type
            WHERE (tenant_id IS NULL OR tenant_id = $1)
            AND ($2::text IS NULL OR LOWER(name) LIKE LOWER($2))
            ORDER BY name
            LIMIT $3 OFFSET $4
            "#,
            tenant_id, text_search,
            page_link.page_size, page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(Self::map_row).collect();
        Ok(PageData::new(data, total, page_link))
    }

    /// Get FQNs for a list of widget type IDs.
    #[instrument(skip(self))]
    pub async fn find_fqns_by_ids(&self, ids: &[Uuid]) -> Result<Vec<String>, DaoError> {
        let rows = sqlx::query!(
            "SELECT fqn FROM widget_type WHERE id = ANY($1) ORDER BY fqn",
            ids
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.fqn).collect())
    }

    #[instrument(skip(self))]
    pub async fn save(&self, wt: &WidgetType) -> Result<WidgetType, DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO widget_type (
                id, created_time, tenant_id, fqn, name, descriptor,
                deprecated, scada, image, description, tags, external_id, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            ON CONFLICT (id) DO UPDATE SET
                name        = EXCLUDED.name,
                descriptor  = EXCLUDED.descriptor,
                deprecated  = EXCLUDED.deprecated,
                scada       = EXCLUDED.scada,
                image       = EXCLUDED.image,
                description = EXCLUDED.description,
                tags        = EXCLUDED.tags,
                external_id = EXCLUDED.external_id,
                version     = widget_type.version + 1
            "#,
            wt.id,
            wt.created_time,
            wt.tenant_id,
            wt.fqn,
            wt.name,
            wt.descriptor,
            wt.deprecated,
            wt.scada,
            wt.image,
            wt.description,
            wt.tags.as_deref(),
            wt.external_id,
            wt.version,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(wt.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM widget_type WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }
}

// ── Internal query struct ─────────────────────────────────────────────────────

struct WidgetTypeRow {
    id:           Uuid,
    created_time: i64,
    tenant_id:    Option<Uuid>,
    fqn:          String,
    name:         String,
    descriptor:   serde_json::Value,
    deprecated:   bool,
    scada:        bool,
    image:        Option<String>,
    description:  Option<String>,
    tags:         Option<Vec<String>>,
    external_id:  Option<Uuid>,
    version:      i64,
}
