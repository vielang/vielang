use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::WidgetsBundle;
use crate::{DaoError, PageData, PageLink};

pub struct WidgetsBundleDao {
    pool: PgPool,
}

impl WidgetsBundleDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    fn map_row(r: WidgetsBundleRow) -> WidgetsBundle {
        WidgetsBundle {
            id:          r.id,
            created_time: r.created_time,
            tenant_id:   r.tenant_id,
            alias:       r.alias,
            title:       r.title,
            image:       r.image,
            scada:       r.scada,
            description: r.description,
            order_index: r.order_index,
            external_id: r.external_id,
            version:     r.version,
        }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<WidgetsBundle>, DaoError> {
        let row = sqlx::query_as!(
            WidgetsBundleRow,
            r#"
            SELECT id, created_time, tenant_id, alias, title,
                   image, scada, description, order_index, external_id, version
            FROM widgets_bundle WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Self::map_row))
    }

    /// List all bundles visible to tenant (system + own tenant bundles), paginated.
    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<WidgetsBundle>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM widgets_bundle
               WHERE (tenant_id IS NULL OR tenant_id = $1)
               AND ($2::text IS NULL OR LOWER(title) LIKE LOWER($2))"#,
            tenant_id, text_search
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            WidgetsBundleRow,
            r#"
            SELECT id, created_time, tenant_id, alias, title,
                   image, scada, description, order_index, external_id, version
            FROM widgets_bundle
            WHERE (tenant_id IS NULL OR tenant_id = $1)
            AND ($2::text IS NULL OR LOWER(title) LIKE LOWER($2))
            ORDER BY COALESCE(order_index, 2147483647), title
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

    /// List all bundles (system-level — admin only).
    #[instrument(skip(self))]
    pub async fn find_all(
        &self,
        page_link: &PageLink,
    ) -> Result<PageData<WidgetsBundle>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM widgets_bundle
               WHERE ($1::text IS NULL OR LOWER(title) LIKE LOWER($1))"#,
            text_search
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            WidgetsBundleRow,
            r#"
            SELECT id, created_time, tenant_id, alias, title,
                   image, scada, description, order_index, external_id, version
            FROM widgets_bundle
            WHERE ($1::text IS NULL OR LOWER(title) LIKE LOWER($1))
            ORDER BY COALESCE(order_index, 2147483647), title
            LIMIT $2 OFFSET $3
            "#,
            text_search,
            page_link.page_size, page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(Self::map_row).collect();
        Ok(PageData::new(data, total, page_link))
    }

    #[instrument(skip(self))]
    pub async fn save(&self, wb: &WidgetsBundle) -> Result<WidgetsBundle, DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO widgets_bundle (
                id, created_time, tenant_id, alias, title,
                image, scada, description, order_index, external_id, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            ON CONFLICT (id) DO UPDATE SET
                title        = EXCLUDED.title,
                image        = EXCLUDED.image,
                scada        = EXCLUDED.scada,
                description  = EXCLUDED.description,
                order_index  = EXCLUDED.order_index,
                external_id  = EXCLUDED.external_id,
                version      = widgets_bundle.version + 1
            "#,
            wb.id,
            wb.created_time,
            wb.tenant_id,
            wb.alias,
            wb.title,
            wb.image,
            wb.scada,
            wb.description,
            wb.order_index,
            wb.external_id,
            wb.version,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(wb.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM widgets_bundle WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }

    /// Assign widget types to bundle by IDs.
    #[instrument(skip(self))]
    pub async fn add_widget_types(
        &self,
        bundle_id: Uuid,
        widget_type_ids: &[Uuid],
    ) -> Result<(), DaoError> {
        for (i, wt_id) in widget_type_ids.iter().enumerate() {
            sqlx::query!(
                r#"
                INSERT INTO widgets_bundle_widget (widgets_bundle_id, widget_type_id, widget_type_order)
                VALUES ($1, $2, $3)
                ON CONFLICT (widgets_bundle_id, widget_type_id) DO UPDATE
                    SET widget_type_order = EXCLUDED.widget_type_order
                "#,
                bundle_id, wt_id, i as i32
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    /// Assign widget types to bundle by FQNs.
    #[instrument(skip(self))]
    pub async fn add_widget_type_fqns(
        &self,
        bundle_id: Uuid,
        fqns: &[String],
    ) -> Result<(), DaoError> {
        for (i, fqn) in fqns.iter().enumerate() {
            sqlx::query!(
                r#"
                INSERT INTO widgets_bundle_widget (widgets_bundle_id, widget_type_id, widget_type_order)
                SELECT $1, id, $3 FROM widget_type WHERE fqn = $2
                ON CONFLICT (widgets_bundle_id, widget_type_id) DO UPDATE
                    SET widget_type_order = EXCLUDED.widget_type_order
                "#,
                bundle_id, fqn, i as i32
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
}

// ── Internal query struct ─────────────────────────────────────────────────────

struct WidgetsBundleRow {
    id:           Uuid,
    created_time: i64,
    tenant_id:    Option<Uuid>,
    alias:        String,
    title:        String,
    image:        Option<String>,
    scada:        bool,
    description:  Option<String>,
    order_index:  Option<i32>,
    external_id:  Option<Uuid>,
    version:      i64,
}
