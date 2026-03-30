use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{Dashboard, DashboardInfo, HomeDashboardInfo};
use crate::{DaoError, PageData, PageLink};

pub struct DashboardDao {
    pool: PgPool,
}

impl DashboardDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Dashboard>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, title, configuration,
                   external_id, mobile_hide, mobile_order, version
            FROM dashboard WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Dashboard {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            title: r.title,
            configuration: r.configuration
                .and_then(|s| serde_json::from_str(&s).ok()),
            external_id: r.external_id,
            mobile_hide: r.mobile_hide,
            mobile_order: r.mobile_order,
            version: r.version,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<Dashboard>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM dashboard
               WHERE tenant_id = $1
               AND ($2::text IS NULL OR LOWER(title) LIKE LOWER($2))"#,
            tenant_id,
            text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, title, configuration,
                   external_id, mobile_hide, mobile_order, version
            FROM dashboard
            WHERE tenant_id = $1
            AND ($2::text IS NULL OR LOWER(title) LIKE LOWER($2))
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

        let data = rows.into_iter().map(|r| Dashboard {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            title: r.title,
            configuration: r.configuration
                .and_then(|s| serde_json::from_str(&s).ok()),
            external_id: r.external_id,
            mobile_hide: r.mobile_hide,
            mobile_order: r.mobile_order,
            version: r.version,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    /// Export all dashboards for a tenant (used by backup service).
    #[instrument(skip(self))]
    pub async fn find_all_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<Dashboard>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, title, configuration,
                   external_id, mobile_hide, mobile_order, version
            FROM dashboard WHERE tenant_id = $1 ORDER BY created_time
            "#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| Dashboard {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            title: r.title,
            configuration: r.configuration.and_then(|s| serde_json::from_str(&s).ok()),
            external_id: r.external_id,
            mobile_hide: r.mobile_hide,
            mobile_order: r.mobile_order,
            version: r.version,
        }).collect())
    }

    /// Like find_by_tenant but optionally filters out mobile_hide=true dashboards.
    /// mobile_only=Some(true) → only show dashboards visible on mobile (mobile_hide = false)
    #[instrument(skip(self))]
    pub async fn find_by_tenant_with_mobile_filter(
        &self,
        tenant_id:   Uuid,
        mobile_only: Option<bool>,
        page_link:   &PageLink,
    ) -> Result<PageData<Dashboard>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM dashboard
               WHERE tenant_id = $1
               AND ($2::text IS NULL OR LOWER(title) LIKE LOWER($2))
               AND ($3::boolean IS NULL OR NOT $3 OR mobile_hide = false)"#,
            tenant_id,
            text_search,
            mobile_only,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"SELECT id, created_time, tenant_id, title, configuration,
                      external_id, mobile_hide, mobile_order, version
               FROM dashboard
               WHERE tenant_id = $1
               AND ($2::text IS NULL OR LOWER(title) LIKE LOWER($2))
               AND ($3::boolean IS NULL OR NOT $3 OR mobile_hide = false)
               ORDER BY created_time DESC
               LIMIT $4 OFFSET $5"#,
            tenant_id,
            text_search,
            mobile_only,
            page_link.page_size,
            page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| Dashboard {
            id:            r.id,
            created_time:  r.created_time,
            tenant_id:     r.tenant_id,
            title:         r.title,
            configuration: r.configuration.and_then(|s| serde_json::from_str(&s).ok()),
            external_id:   r.external_id,
            mobile_hide:   r.mobile_hide,
            mobile_order:  r.mobile_order,
            version:       r.version,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    #[instrument(skip(self))]
    pub async fn save(&self, dashboard: &Dashboard) -> Result<Dashboard, DaoError> {
        let configuration = dashboard.configuration.as_ref().map(|v| v.to_string());

        sqlx::query!(
            r#"
            INSERT INTO dashboard (
                id, created_time, tenant_id, title, configuration,
                external_id, mobile_hide, mobile_order, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            ON CONFLICT (id) DO UPDATE SET
                title         = EXCLUDED.title,
                configuration = EXCLUDED.configuration,
                external_id   = EXCLUDED.external_id,
                mobile_hide   = EXCLUDED.mobile_hide,
                mobile_order  = EXCLUDED.mobile_order,
                version       = dashboard.version + 1
            "#,
            dashboard.id,
            dashboard.created_time,
            dashboard.tenant_id,
            dashboard.title,
            configuration,
            dashboard.external_id,
            dashboard.mobile_hide,
            dashboard.mobile_order,
            dashboard.version,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(dashboard.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM dashboard WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn find_info_by_id(&self, id: Uuid) -> Result<Option<DashboardInfo>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, title,
                   assigned_customers, mobile_hide, mobile_order
            FROM dashboard WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| DashboardInfo {
            id:                  r.id,
            created_time:        r.created_time,
            tenant_id:           r.tenant_id,
            title:               r.title,
            assigned_customers:  r.assigned_customers,
            mobile_hide:         r.mobile_hide,
            mobile_order:        r.mobile_order,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_infos_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<DashboardInfo>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM dashboard
               WHERE tenant_id = $1
               AND ($2::text IS NULL OR LOWER(title) LIKE LOWER($2))"#,
            tenant_id,
            text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, title,
                   assigned_customers, mobile_hide, mobile_order
            FROM dashboard
            WHERE tenant_id = $1
            AND ($2::text IS NULL OR LOWER(title) LIKE LOWER($2))
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

        let data = rows.into_iter().map(|r| DashboardInfo {
            id:                 r.id,
            created_time:       r.created_time,
            tenant_id:          r.tenant_id,
            title:              r.title,
            assigned_customers: r.assigned_customers,
            mobile_hide:        r.mobile_hide,
            mobile_order:       r.mobile_order,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    /// Get the home dashboard ID for a tenant (stored in admin_settings).
    #[instrument(skip(self))]
    pub async fn get_home_dashboard_info(&self, tenant_id: Uuid) -> Result<HomeDashboardInfo, DaoError> {
        let row = sqlx::query!(
            "SELECT json_value FROM admin_settings WHERE tenant_id = $1 AND key = 'home_dashboard'",
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            if let Ok(info) = serde_json::from_value::<HomeDashboardInfo>(r.json_value) {
                return Ok(info);
            }
        }
        Ok(HomeDashboardInfo { dashboard_id: None, hidden_dashboard_toolbar: false })
    }

    /// Set (or unset) the home dashboard for a tenant.
    #[instrument(skip(self))]
    pub async fn set_home_dashboard(&self, tenant_id: Uuid, info: &HomeDashboardInfo) -> Result<(), DaoError> {
        let json_value = serde_json::to_value(info)
            .map_err(|e| DaoError::Database(sqlx::Error::Decode(Box::new(e))))?;
        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query!(
            r#"
            INSERT INTO admin_settings (id, created_time, tenant_id, key, json_value)
            VALUES (gen_random_uuid(), $1, $2, 'home_dashboard', $3)
            ON CONFLICT (tenant_id, key) DO UPDATE SET json_value = EXCLUDED.json_value
            "#,
            now,
            tenant_id,
            json_value,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Replace the full list of assigned customers for a dashboard.
    #[instrument(skip(self))]
    pub async fn update_assigned_customers(&self, id: Uuid, customers_json: Option<String>) -> Result<(), DaoError> {
        sqlx::query!(
            "UPDATE dashboard SET assigned_customers = $1 WHERE id = $2",
            customers_json,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
