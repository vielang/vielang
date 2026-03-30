use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::TbResource;
use crate::{DaoError, PageData, PageLink};

pub struct ResourceDao {
    pool: PgPool,
}

impl ResourceDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    fn map_row(r: ResourceRow) -> TbResource {
        TbResource {
            id:                 r.id,
            created_time:       r.created_time,
            tenant_id:          r.tenant_id,
            title:              r.title,
            resource_type:      r.resource_type,
            resource_sub_type:  r.resource_sub_type,
            resource_key:       r.resource_key,
            file_name:          r.file_name,
            is_public:          r.is_public,
            public_resource_key: r.public_resource_key,
            etag:               r.etag,
            descriptor:         r.descriptor,
            data:               r.data,
            preview:            r.preview,
            external_id:        r.external_id,
            version:            r.version,
        }
    }

    fn map_info_row(r: ResourceInfoRow) -> TbResource {
        TbResource {
            id:                 r.id,
            created_time:       r.created_time,
            tenant_id:          r.tenant_id,
            title:              r.title,
            resource_type:      r.resource_type,
            resource_sub_type:  r.resource_sub_type,
            resource_key:       r.resource_key,
            file_name:          r.file_name,
            is_public:          r.is_public,
            public_resource_key: r.public_resource_key,
            etag:               r.etag,
            descriptor:         r.descriptor,
            data:               None,
            preview:            None,
            external_id:        r.external_id,
            version:            r.version,
        }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<TbResource>, DaoError> {
        let row = sqlx::query_as!(
            ResourceRow,
            r#"
            SELECT id, created_time, tenant_id, title, resource_type,
                   resource_sub_type, resource_key, file_name, is_public,
                   public_resource_key, etag, descriptor, data, preview,
                   external_id, version
            FROM resource WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Self::map_row))
    }

    /// Find by id — metadata only (no data/preview blobs).
    #[instrument(skip(self))]
    pub async fn find_info_by_id(&self, id: Uuid) -> Result<Option<TbResource>, DaoError> {
        let row = sqlx::query_as!(
            ResourceInfoRow,
            r#"
            SELECT id, created_time, tenant_id, title, resource_type,
                   resource_sub_type, resource_key, file_name, is_public,
                   public_resource_key, etag, descriptor, external_id, version
            FROM resource WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Self::map_info_row))
    }

    /// Find by (tenant_id, resource_type, resource_key) — for image download.
    #[instrument(skip(self))]
    pub async fn find_by_key(
        &self,
        tenant_id: Uuid,
        resource_type: &str,
        resource_key: &str,
    ) -> Result<Option<TbResource>, DaoError> {
        let row = sqlx::query_as!(
            ResourceRow,
            r#"
            SELECT id, created_time, tenant_id, title, resource_type,
                   resource_sub_type, resource_key, file_name, is_public,
                   public_resource_key, etag, descriptor, data, preview,
                   external_id, version
            FROM resource
            WHERE tenant_id = $1 AND resource_type = $2 AND resource_key = $3
            "#,
            tenant_id, resource_type, resource_key
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Self::map_row))
    }

    /// Find public resource by public_resource_key (for unauthenticated image download).
    #[instrument(skip(self))]
    pub async fn find_public_by_key(
        &self,
        public_resource_key: &str,
    ) -> Result<Option<TbResource>, DaoError> {
        let row = sqlx::query_as!(
            ResourceRow,
            r#"
            SELECT id, created_time, tenant_id, title, resource_type,
                   resource_sub_type, resource_key, file_name, is_public,
                   public_resource_key, etag, descriptor, data, preview,
                   external_id, version
            FROM resource
            WHERE is_public = TRUE AND public_resource_key = $1
            "#,
            public_resource_key
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Self::map_row))
    }

    /// List resources for tenant, paginated (no blobs).
    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        resource_type: Option<&str>,
        page_link: &PageLink,
    ) -> Result<PageData<TbResource>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM resource
               WHERE (tenant_id IS NULL OR tenant_id = $1)
               AND ($2::text IS NULL OR resource_type = $2)
               AND ($3::text IS NULL OR LOWER(title) LIKE LOWER($3))"#,
            tenant_id, resource_type, text_search
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            ResourceInfoRow,
            r#"
            SELECT id, created_time, tenant_id, title, resource_type,
                   resource_sub_type, resource_key, file_name, is_public,
                   public_resource_key, etag, descriptor, external_id, version
            FROM resource
            WHERE (tenant_id IS NULL OR tenant_id = $1)
            AND ($2::text IS NULL OR resource_type = $2)
            AND ($3::text IS NULL OR LOWER(title) LIKE LOWER($3))
            ORDER BY created_time DESC
            LIMIT $4 OFFSET $5
            "#,
            tenant_id, resource_type, text_search,
            page_link.page_size, page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(Self::map_info_row).collect();
        Ok(PageData::new(data, total, page_link))
    }

    #[instrument(skip(self, res))]
    pub async fn save(&self, res: &TbResource) -> Result<TbResource, DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO resource (
                id, created_time, tenant_id, title, resource_type,
                resource_sub_type, resource_key, file_name, is_public,
                public_resource_key, etag, descriptor, data, preview,
                external_id, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
            ON CONFLICT (id) DO UPDATE SET
                title               = EXCLUDED.title,
                resource_sub_type   = EXCLUDED.resource_sub_type,
                file_name           = EXCLUDED.file_name,
                is_public           = EXCLUDED.is_public,
                public_resource_key = EXCLUDED.public_resource_key,
                etag                = EXCLUDED.etag,
                descriptor          = EXCLUDED.descriptor,
                data                = EXCLUDED.data,
                preview             = EXCLUDED.preview,
                external_id         = EXCLUDED.external_id,
                version             = resource.version + 1
            "#,
            res.id,
            res.created_time,
            res.tenant_id,
            res.title,
            res.resource_type,
            res.resource_sub_type,
            res.resource_key,
            res.file_name,
            res.is_public,
            res.public_resource_key,
            res.etag,
            res.descriptor,
            res.data.as_deref(),
            res.preview.as_deref(),
            res.external_id,
            res.version,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_info_by_id(res.id).await?.ok_or(DaoError::NotFound)
    }

    /// Update metadata only (title, is_public, descriptor) without touching data blobs.
    #[instrument(skip(self))]
    pub async fn update_info(&self, res: &TbResource) -> Result<TbResource, DaoError> {
        sqlx::query!(
            r#"
            UPDATE resource SET
                title               = $2,
                is_public           = $3,
                public_resource_key = $4,
                descriptor          = $5,
                version             = version + 1
            WHERE id = $1
            "#,
            res.id,
            res.title,
            res.is_public,
            res.public_resource_key,
            res.descriptor,
        )
        .execute(&self.pool)
        .await?;

        self.find_info_by_id(res.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM resource WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }
}

// ── Internal query structs ────────────────────────────────────────────────────

struct ResourceRow {
    id:                  Uuid,
    created_time:        i64,
    tenant_id:           Option<Uuid>,
    title:               String,
    resource_type:       String,
    resource_sub_type:   Option<String>,
    resource_key:        String,
    file_name:           String,
    is_public:           bool,
    public_resource_key: Option<String>,
    etag:                Option<String>,
    descriptor:          Option<serde_json::Value>,
    data:                Option<Vec<u8>>,
    preview:             Option<Vec<u8>>,
    external_id:         Option<Uuid>,
    version:             i64,
}

/// Info only — no blobs
struct ResourceInfoRow {
    id:                  Uuid,
    created_time:        i64,
    tenant_id:           Option<Uuid>,
    title:               String,
    resource_type:       String,
    resource_sub_type:   Option<String>,
    resource_key:        String,
    file_name:           String,
    is_public:           bool,
    public_resource_key: Option<String>,
    etag:                Option<String>,
    descriptor:          Option<serde_json::Value>,
    external_id:         Option<Uuid>,
    version:             i64,
}
