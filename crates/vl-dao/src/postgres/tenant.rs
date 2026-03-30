use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::Tenant;
use crate::{DaoError, PageData, PageLink};

pub struct TenantDao {
    pool: PgPool,
}

impl TenantDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Tenant>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_profile_id, title, region,
                   country, state, city, address, address2, zip, phone, email,
                   additional_info, version
            FROM tenant WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Tenant {
            id: r.id,
            created_time: r.created_time,
            tenant_profile_id: r.tenant_profile_id,
            title: r.title,
            region: r.region,
            country: r.country,
            state: r.state,
            city: r.city,
            address: r.address,
            address2: r.address2,
            zip: r.zip,
            phone: r.phone,
            email: r.email,
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version: r.version,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_all(&self, page_link: &PageLink) -> Result<PageData<Tenant>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM tenant WHERE ($1::text IS NULL OR LOWER(title) LIKE LOWER($1))",
            text_search
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_profile_id, title, region,
                   country, state, city, address, address2, zip, phone, email,
                   additional_info, version
            FROM tenant
            WHERE ($1::text IS NULL OR LOWER(title) LIKE LOWER($1))
            ORDER BY created_time DESC
            LIMIT $2 OFFSET $3
            "#,
            text_search,
            page_link.page_size,
            page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| Tenant {
            id: r.id,
            created_time: r.created_time,
            tenant_profile_id: r.tenant_profile_id,
            title: r.title,
            region: r.region,
            country: r.country,
            state: r.state,
            city: r.city,
            address: r.address,
            address2: r.address2,
            zip: r.zip,
            phone: r.phone,
            email: r.email,
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version: r.version,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    #[instrument(skip(self))]
    pub async fn save(&self, tenant: &Tenant) -> Result<Tenant, DaoError> {
        let additional_info = tenant.additional_info.as_ref().map(|v| v.to_string());

        sqlx::query!(
            r#"
            INSERT INTO tenant (
                id, created_time, tenant_profile_id, title, region,
                country, state, city, address, address2, zip, phone, email,
                additional_info, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
            ON CONFLICT (id) DO UPDATE SET
                tenant_profile_id = EXCLUDED.tenant_profile_id,
                title             = EXCLUDED.title,
                region            = EXCLUDED.region,
                country           = EXCLUDED.country,
                state             = EXCLUDED.state,
                city              = EXCLUDED.city,
                address           = EXCLUDED.address,
                address2          = EXCLUDED.address2,
                zip               = EXCLUDED.zip,
                phone             = EXCLUDED.phone,
                email             = EXCLUDED.email,
                additional_info   = EXCLUDED.additional_info,
                version           = tenant.version + 1
            "#,
            tenant.id,
            tenant.created_time,
            tenant.tenant_profile_id,
            tenant.title,
            tenant.region,
            tenant.country,
            tenant.state,
            tenant.city,
            tenant.address,
            tenant.address2,
            tenant.zip,
            tenant.phone,
            tenant.email,
            additional_info,
            tenant.version,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(tenant.id).await?.ok_or(DaoError::NotFound)
    }

    /// Return all tenant IDs (for actor system init).
    #[instrument(skip(self))]
    pub async fn find_all_ids(&self) -> Result<Vec<Uuid>, DaoError> {
        let rows: Vec<(Uuid,)> =
            sqlx::query_as("SELECT id FROM tenant ORDER BY created_time")
                .fetch_all(&self.pool)
                .await?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM tenant WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }
}
