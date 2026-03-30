use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::AssetProfile;
use crate::{DaoError, PageData, PageLink};

pub struct AssetProfileDao {
    pool: PgPool,
}

impl AssetProfileDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    fn map_row(r: &AssetProfileRow) -> AssetProfile {
        AssetProfile {
            id:                        r.id,
            created_time:              r.created_time,
            tenant_id:                 r.tenant_id,
            name:                      r.name.clone(),
            description:               r.description.clone(),
            image:                     r.image.clone(),
            is_default:                r.is_default,
            default_rule_chain_id:     r.default_rule_chain_id,
            default_dashboard_id:      r.default_dashboard_id,
            default_queue_name:        r.default_queue_name.clone(),
            default_edge_rule_chain_id: r.default_edge_rule_chain_id,
            external_id:               r.external_id,
            version:                   r.version,
        }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<AssetProfile>, DaoError> {
        let row = sqlx::query_as!(
            AssetProfileRow,
            r#"
            SELECT id, created_time, tenant_id, name, description, image,
                   is_default, default_rule_chain_id, default_dashboard_id,
                   default_queue_name, default_edge_rule_chain_id,
                   external_id, version
            FROM asset_profile WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(Self::map_row))
    }

    #[instrument(skip(self))]
    pub async fn find_default(&self, tenant_id: Uuid) -> Result<Option<AssetProfile>, DaoError> {
        let row = sqlx::query_as!(
            AssetProfileRow,
            r#"
            SELECT id, created_time, tenant_id, name, description, image,
                   is_default, default_rule_chain_id, default_dashboard_id,
                   default_queue_name, default_edge_rule_chain_id,
                   external_id, version
            FROM asset_profile WHERE tenant_id = $1 AND is_default = TRUE
            LIMIT 1
            "#,
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(Self::map_row))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<AssetProfile>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM asset_profile
               WHERE tenant_id = $1
               AND ($2::text IS NULL OR LOWER(name) LIKE LOWER($2))"#,
            tenant_id,
            text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            AssetProfileRow,
            r#"
            SELECT id, created_time, tenant_id, name, description, image,
                   is_default, default_rule_chain_id, default_dashboard_id,
                   default_queue_name, default_edge_rule_chain_id,
                   external_id, version
            FROM asset_profile
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

        let data = rows.iter().map(Self::map_row).collect();
        Ok(PageData::new(data, total, page_link))
    }

    /// Lấy danh sách tên profile (id + name) theo tenant
    #[instrument(skip(self))]
    pub async fn find_names_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<(Uuid, String)>, DaoError> {
        let rows = sqlx::query!(
            "SELECT id, name FROM asset_profile WHERE tenant_id = $1 ORDER BY name",
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| (r.id, r.name)).collect())
    }

    #[instrument(skip(self))]
    pub async fn save(&self, profile: &AssetProfile) -> Result<AssetProfile, DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO asset_profile (
                id, created_time, tenant_id, name, description, image,
                is_default, default_rule_chain_id, default_dashboard_id,
                default_queue_name, default_edge_rule_chain_id,
                external_id, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            ON CONFLICT (id) DO UPDATE SET
                name                      = EXCLUDED.name,
                description               = EXCLUDED.description,
                image                     = EXCLUDED.image,
                is_default                = EXCLUDED.is_default,
                default_rule_chain_id     = EXCLUDED.default_rule_chain_id,
                default_dashboard_id      = EXCLUDED.default_dashboard_id,
                default_queue_name        = EXCLUDED.default_queue_name,
                default_edge_rule_chain_id = EXCLUDED.default_edge_rule_chain_id,
                external_id               = EXCLUDED.external_id,
                version                   = asset_profile.version + 1
            "#,
            profile.id,
            profile.created_time,
            profile.tenant_id,
            profile.name,
            profile.description,
            profile.image,
            profile.is_default,
            profile.default_rule_chain_id,
            profile.default_dashboard_id,
            profile.default_queue_name,
            profile.default_edge_rule_chain_id,
            profile.external_id,
            profile.version,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(profile.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn set_default(&self, tenant_id: Uuid, profile_id: Uuid) -> Result<(), DaoError> {
        sqlx::query!(
            "UPDATE asset_profile SET is_default = FALSE WHERE tenant_id = $1 AND is_default = TRUE",
            tenant_id
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            "UPDATE asset_profile SET is_default = TRUE WHERE id = $1",
            profile_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM asset_profile WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }
}

// ── Internal query struct ─────────────────────────────────────────────────────

struct AssetProfileRow {
    id:                        Uuid,
    created_time:              i64,
    tenant_id:                 Uuid,
    name:                      String,
    description:               Option<String>,
    image:                     Option<String>,
    is_default:                bool,
    default_rule_chain_id:     Option<Uuid>,
    default_dashboard_id:      Option<Uuid>,
    default_queue_name:        Option<String>,
    default_edge_rule_chain_id: Option<Uuid>,
    external_id:               Option<Uuid>,
    version:                   i64,
}
