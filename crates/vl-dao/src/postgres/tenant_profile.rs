use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{TenantProfile, EntityInfo};
use crate::{DaoError, PageData, PageLink};

pub struct TenantProfileDao {
    pool: PgPool,
}

impl TenantProfileDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    fn map_row(r: &TenantProfileRow) -> TenantProfile {
        TenantProfile {
            id:                       r.id,
            created_time:             r.created_time,
            name:                     r.name.clone(),
            description:              r.description.clone(),
            is_default:               r.is_default,
            isolated_vl_rule_engine:  r.isolated_vl_rule_engine,
            profile_data:             r.profile_data.clone(),
            version:                  r.version,
        }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<TenantProfile>, DaoError> {
        let row = sqlx::query_as!(
            TenantProfileRow,
            r#"
            SELECT id, created_time, name, description,
                   is_default, isolated_vl_rule_engine, profile_data, version
            FROM tenant_profile WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(Self::map_row))
    }

    #[instrument(skip(self))]
    pub async fn find_default(&self) -> Result<Option<TenantProfile>, DaoError> {
        let row = sqlx::query_as!(
            TenantProfileRow,
            r#"
            SELECT id, created_time, name, description,
                   is_default, isolated_vl_rule_engine, profile_data, version
            FROM tenant_profile WHERE is_default = TRUE
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(Self::map_row))
    }

    #[instrument(skip(self))]
    pub async fn find_by_page(
        &self,
        page_link: &PageLink,
    ) -> Result<PageData<TenantProfile>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM tenant_profile
               WHERE ($1::text IS NULL OR LOWER(name) LIKE LOWER($1))"#,
            text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            TenantProfileRow,
            r#"
            SELECT id, created_time, name, description,
                   is_default, isolated_vl_rule_engine, profile_data, version
            FROM tenant_profile
            WHERE ($1::text IS NULL OR LOWER(name) LIKE LOWER($1))
            ORDER BY created_time DESC
            LIMIT $2 OFFSET $3
            "#,
            text_search,
            page_link.page_size,
            page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.iter().map(Self::map_row).collect();
        Ok(PageData::new(data, total, page_link))
    }

    #[instrument(skip(self))]
    pub async fn find_infos_by_page(
        &self,
        page_link: &PageLink,
    ) -> Result<PageData<EntityInfo>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM tenant_profile
               WHERE ($1::text IS NULL OR LOWER(name) LIKE LOWER($1))"#,
            text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, name FROM tenant_profile
            WHERE ($1::text IS NULL OR LOWER(name) LIKE LOWER($1))
            ORDER BY name ASC
            LIMIT $2 OFFSET $3
            "#,
            text_search,
            page_link.page_size,
            page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| EntityInfo { id: r.id, name: r.name }).collect();
        Ok(PageData::new(data, total, page_link))
    }

    /// Load the configured rate limit for a tenant.
    /// Returns `None` if the tenant doesn't exist or has no custom limit set
    /// (meaning the server-wide default should be used).
    #[instrument(skip(self))]
    pub async fn find_rate_limit(&self, tenant_id: Uuid) -> Result<Option<i32>, DaoError> {
        let limit = sqlx::query_scalar!(
            r#"
            SELECT tp.rate_limit_per_second
            FROM tenant t
            JOIN tenant_profile tp ON tp.id = t.tenant_profile_id
            WHERE t.id = $1
            "#,
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await?;
        // fetch_optional returns Option<Option<i32>> — flatten to Option<i32>
        Ok(limit.flatten())
    }

    #[instrument(skip(self))]
    pub async fn find_info_by_id(&self, id: Uuid) -> Result<Option<EntityInfo>, DaoError> {
        let row = sqlx::query!(
            "SELECT id, name FROM tenant_profile WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| EntityInfo { id: r.id, name: r.name }))
    }

    #[instrument(skip(self))]
    pub async fn save(&self, profile: &TenantProfile) -> Result<TenantProfile, DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO tenant_profile (
                id, created_time, name, description,
                is_default, isolated_vl_rule_engine, profile_data, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            ON CONFLICT (id) DO UPDATE SET
                name                    = EXCLUDED.name,
                description             = EXCLUDED.description,
                is_default              = EXCLUDED.is_default,
                isolated_vl_rule_engine = EXCLUDED.isolated_vl_rule_engine,
                profile_data            = EXCLUDED.profile_data,
                version                 = tenant_profile.version + 1
            "#,
            profile.id,
            profile.created_time,
            profile.name,
            profile.description,
            profile.is_default,
            profile.isolated_vl_rule_engine,
            profile.profile_data,
            profile.version,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(profile.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn set_default(&self, profile_id: Uuid) -> Result<(), DaoError> {
        sqlx::query!(
            "UPDATE tenant_profile SET is_default = FALSE WHERE is_default = TRUE"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            "UPDATE tenant_profile SET is_default = TRUE WHERE id = $1",
            profile_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM tenant_profile WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }
}

// ── Internal query struct ─────────────────────────────────────────────────────

struct TenantProfileRow {
    id:                      Uuid,
    created_time:            i64,
    name:                    String,
    description:             Option<String>,
    is_default:              bool,
    isolated_vl_rule_engine: bool,
    profile_data:            Option<serde_json::Value>,
    version:                 i64,
}
