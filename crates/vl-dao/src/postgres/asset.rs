use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{Asset, AssetInfoView};
use crate::{DaoError, PageData, PageLink};

pub struct AssetDao {
    pool: PgPool,
}

impl AssetDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Asset>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id,
                   asset_profile_id, name, type as asset_type, label,
                   external_id, additional_info, version
            FROM asset WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Asset {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            asset_profile_id: r.asset_profile_id,
            name: r.name,
            asset_type: r.asset_type.unwrap_or_default(),
            label: r.label,
            external_id: r.external_id,
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version: r.version,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<Asset>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM asset
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
            SELECT id, created_time, tenant_id, customer_id,
                   asset_profile_id, name, type as asset_type, label,
                   external_id, additional_info, version
            FROM asset
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

        let data = rows.into_iter().map(|r| Asset {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            asset_profile_id: r.asset_profile_id,
            name: r.name,
            asset_type: r.asset_type.unwrap_or_default(),
            label: r.label,
            external_id: r.external_id,
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version: r.version,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    /// Export all assets for a tenant (used by backup service).
    #[instrument(skip(self))]
    pub async fn find_all_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<Asset>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id,
                   asset_profile_id, name, type as asset_type, label,
                   external_id, additional_info, version
            FROM asset WHERE tenant_id = $1 ORDER BY created_time
            "#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| Asset {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            asset_profile_id: r.asset_profile_id,
            name: r.name,
            asset_type: r.asset_type.unwrap_or_default(),
            label: r.label,
            external_id: r.external_id,
            additional_info: r.additional_info.and_then(|s| serde_json::from_str(&s).ok()),
            version: r.version,
        }).collect())
    }

    #[instrument(skip(self))]
    pub async fn find_by_customer(
        &self,
        customer_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<Asset>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM asset
               WHERE customer_id = $1
               AND ($2::text IS NULL OR LOWER(name) LIKE LOWER($2))"#,
            customer_id,
            text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id,
                   asset_profile_id, name, type as asset_type, label,
                   external_id, additional_info, version
            FROM asset
            WHERE customer_id = $1
            AND ($2::text IS NULL OR LOWER(name) LIKE LOWER($2))
            ORDER BY created_time DESC
            LIMIT $3 OFFSET $4
            "#,
            customer_id,
            text_search,
            page_link.page_size,
            page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| Asset {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            asset_profile_id: r.asset_profile_id,
            name: r.name,
            asset_type: r.asset_type.unwrap_or_default(),
            label: r.label,
            external_id: r.external_id,
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version: r.version,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    #[instrument(skip(self))]
    pub async fn save(&self, asset: &Asset) -> Result<Asset, DaoError> {
        let additional_info = asset.additional_info.as_ref().map(|v| v.to_string());

        sqlx::query!(
            r#"
            INSERT INTO asset (
                id, created_time, tenant_id, customer_id, asset_profile_id,
                name, type, label, external_id, additional_info, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            ON CONFLICT (id) DO UPDATE SET
                name             = EXCLUDED.name,
                type             = EXCLUDED.type,
                label            = EXCLUDED.label,
                customer_id      = EXCLUDED.customer_id,
                asset_profile_id = EXCLUDED.asset_profile_id,
                external_id      = EXCLUDED.external_id,
                additional_info  = EXCLUDED.additional_info,
                version          = asset.version + 1
            "#,
            asset.id,
            asset.created_time,
            asset.tenant_id,
            asset.customer_id,
            asset.asset_profile_id,
            asset.name,
            asset.asset_type,
            asset.label,
            asset.external_id,
            additional_info,
            asset.version,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(asset.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM asset WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }

    /// Tìm asset theo tên trong tenant — dùng cho bulk import
    #[instrument(skip(self))]
    pub async fn find_by_name(
        &self,
        tenant_id: Uuid,
        name: &str,
    ) -> Result<Option<Asset>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id,
                   asset_profile_id, name, type as asset_type, label,
                   external_id, additional_info, version
            FROM asset WHERE tenant_id = $1 AND name = $2
            "#,
            tenant_id,
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Asset {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            asset_profile_id: r.asset_profile_id,
            name: r.name,
            asset_type: r.asset_type.unwrap_or_default(),
            label: r.label,
            external_id: r.external_id,
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version: r.version,
        }))
    }

    /// GET /api/asset/info/{assetId}
    #[instrument(skip(self))]
    pub async fn find_info_by_id(&self, id: Uuid) -> Result<Option<AssetInfoView>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT a.id, a.created_time, a.tenant_id, a.customer_id,
                   a.name, a.type as asset_type, a.label,
                   a.asset_profile_id, ap.name as asset_profile_name,
                   c.title as "customer_title?"
            FROM asset a
            JOIN asset_profile ap ON a.asset_profile_id = ap.id
            LEFT JOIN customer c ON a.customer_id = c.id
            WHERE a.id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| AssetInfoView {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            name: r.name,
            asset_type: r.asset_type.unwrap_or_default(),
            label: r.label,
            asset_profile_id: r.asset_profile_id,
            asset_profile_name: r.asset_profile_name,
            customer_title: r.customer_title,
        }))
    }

    /// GET /api/tenant/assetInfos
    #[instrument(skip(self))]
    pub async fn find_infos_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<AssetInfoView>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM asset WHERE tenant_id = $1
               AND ($2::text IS NULL OR LOWER(name) LIKE LOWER($2))"#,
            tenant_id, text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT a.id, a.created_time, a.tenant_id, a.customer_id,
                   a.name, a.type as asset_type, a.label,
                   a.asset_profile_id, ap.name as asset_profile_name,
                   c.title as "customer_title?"
            FROM asset a
            JOIN asset_profile ap ON a.asset_profile_id = ap.id
            LEFT JOIN customer c ON a.customer_id = c.id
            WHERE a.tenant_id = $1
            AND ($2::text IS NULL OR LOWER(a.name) LIKE LOWER($2))
            ORDER BY a.created_time DESC
            LIMIT $3 OFFSET $4
            "#,
            tenant_id, text_search, page_link.page_size, page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| AssetInfoView {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            name: r.name,
            asset_type: r.asset_type.unwrap_or_default(),
            label: r.label,
            asset_profile_id: r.asset_profile_id,
            asset_profile_name: r.asset_profile_name,
            customer_title: r.customer_title,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    /// GET /api/customer/{customerId}/assetInfos
    #[instrument(skip(self))]
    pub async fn find_infos_by_customer(
        &self,
        customer_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<AssetInfoView>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM asset WHERE customer_id = $1
               AND ($2::text IS NULL OR LOWER(name) LIKE LOWER($2))"#,
            customer_id, text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT a.id, a.created_time, a.tenant_id, a.customer_id,
                   a.name, a.type as asset_type, a.label,
                   a.asset_profile_id, ap.name as asset_profile_name,
                   c.title as "customer_title?"
            FROM asset a
            JOIN asset_profile ap ON a.asset_profile_id = ap.id
            LEFT JOIN customer c ON a.customer_id = c.id
            WHERE a.customer_id = $1
            AND ($2::text IS NULL OR LOWER(a.name) LIKE LOWER($2))
            ORDER BY a.created_time DESC
            LIMIT $3 OFFSET $4
            "#,
            customer_id, text_search, page_link.page_size, page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| AssetInfoView {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            name: r.name,
            asset_type: r.asset_type.unwrap_or_default(),
            label: r.label,
            asset_profile_id: r.asset_profile_id,
            asset_profile_name: r.asset_profile_name,
            customer_title: r.customer_title,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }
}
