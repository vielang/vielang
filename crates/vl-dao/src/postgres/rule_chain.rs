use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{RuleChain, RuleChainMetaData};
use crate::{DaoError, PageData, PageLink};

pub struct RuleChainDao {
    pool: PgPool,
}

impl RuleChainDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<RuleChain>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, name,
                   type as chain_type,
                   first_rule_node_id, root, debug_mode,
                   configuration, additional_info, external_id, version
            FROM rule_chain WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| RuleChain {
            id:                 r.id,
            created_time:       r.created_time,
            tenant_id:          r.tenant_id,
            name:               r.name,
            chain_type:         r.chain_type,
            first_rule_node_id: r.first_rule_node_id,
            root:               r.root,
            debug_mode:         r.debug_mode,
            configuration:      r.configuration,
            additional_info:    r.additional_info,
            external_id:        r.external_id,
            version:            r.version,
        }))
    }

    /// Find root chain for a tenant
    #[instrument(skip(self))]
    pub async fn find_root_by_tenant(&self, tenant_id: Uuid) -> Result<Option<RuleChain>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, name,
                   type as chain_type,
                   first_rule_node_id, root, debug_mode,
                   configuration, additional_info, external_id, version
            FROM rule_chain
            WHERE tenant_id = $1 AND root = TRUE
            LIMIT 1
            "#,
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| RuleChain {
            id:                 r.id,
            created_time:       r.created_time,
            tenant_id:          r.tenant_id,
            name:               r.name,
            chain_type:         r.chain_type,
            first_rule_node_id: r.first_rule_node_id,
            root:               r.root,
            debug_mode:         r.debug_mode,
            configuration:      r.configuration,
            additional_info:    r.additional_info,
            external_id:        r.external_id,
            version:            r.version,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<RuleChain>, DaoError> {
        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM rule_chain WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, name,
                   type as chain_type,
                   first_rule_node_id, root, debug_mode,
                   configuration, additional_info, external_id, version
            FROM rule_chain
            WHERE tenant_id = $1
            ORDER BY created_time DESC
            LIMIT $2 OFFSET $3
            "#,
            tenant_id,
            page_link.page_size,
            page_link.offset()
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| RuleChain {
            id:                 r.id,
            created_time:       r.created_time,
            tenant_id:          r.tenant_id,
            name:               r.name,
            chain_type:         r.chain_type,
            first_rule_node_id: r.first_rule_node_id,
            root:               r.root,
            debug_mode:         r.debug_mode,
            configuration:      r.configuration,
            additional_info:    r.additional_info,
            external_id:        r.external_id,
            version:            r.version,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    /// Export all rule chains for a tenant (used by backup service).
    #[instrument(skip(self))]
    pub async fn find_all_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<RuleChain>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, name,
                   type as chain_type,
                   first_rule_node_id, root, debug_mode,
                   configuration, additional_info, external_id, version
            FROM rule_chain WHERE tenant_id = $1 ORDER BY created_time
            "#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| RuleChain {
            id:                 r.id,
            created_time:       r.created_time,
            tenant_id:          r.tenant_id,
            name:               r.name,
            chain_type:         r.chain_type,
            first_rule_node_id: r.first_rule_node_id,
            root:               r.root,
            debug_mode:         r.debug_mode,
            configuration:      r.configuration,
            additional_info:    r.additional_info,
            external_id:        r.external_id,
            version:            r.version,
        }).collect())
    }

    #[instrument(skip(self))]
    pub async fn save(&self, chain: &RuleChain) -> Result<RuleChain, DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO rule_chain (
                id, created_time, tenant_id, name, type,
                first_rule_node_id, root, debug_mode,
                configuration, additional_info, external_id, version
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
            ON CONFLICT (id) DO UPDATE SET
                name               = EXCLUDED.name,
                type               = EXCLUDED.type,
                first_rule_node_id = EXCLUDED.first_rule_node_id,
                root               = EXCLUDED.root,
                debug_mode         = EXCLUDED.debug_mode,
                configuration      = EXCLUDED.configuration,
                additional_info    = EXCLUDED.additional_info,
                external_id        = EXCLUDED.external_id,
                version            = EXCLUDED.version
            "#,
            chain.id,
            chain.created_time,
            chain.tenant_id,
            chain.name,
            chain.chain_type,
            chain.first_rule_node_id,
            chain.root,
            chain.debug_mode,
            chain.configuration,
            chain.additional_info,
            chain.external_id,
            chain.version,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(chain.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        sqlx::query!("DELETE FROM rule_chain WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Set a chain as the root chain for a tenant (unset all others first)
    #[instrument(skip(self))]
    pub async fn set_root(&self, tenant_id: Uuid, chain_id: Uuid) -> Result<(), DaoError> {
        sqlx::query!(
            "UPDATE rule_chain SET root = FALSE WHERE tenant_id = $1",
            tenant_id
        )
        .execute(&self.pool)
        .await?;
        sqlx::query!(
            "UPDATE rule_chain SET root = TRUE WHERE id = $1",
            chain_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// GET /api/ruleChain/{id}/metadata — deserialize from `configuration` column
    #[instrument(skip(self))]
    pub async fn find_metadata(&self, id: Uuid) -> Result<Option<RuleChainMetaData>, DaoError> {
        let row = sqlx::query!(
            "SELECT configuration FROM rule_chain WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| {
            let config = r.configuration?;
            serde_json::from_str::<RuleChainMetaData>(&config).ok().or_else(|| {
                // No metadata stored yet — return empty metadata
                Some(RuleChainMetaData {
                    rule_chain_id: id,
                    first_node_index: None,
                    nodes: vec![],
                    connections: vec![],
                    rule_chain_connections: None,
                })
            })
        }).or_else(|| {
            // Row not found — return None
            None
        }))
    }

    /// POST /api/ruleChain/{id}/metadata — serialize into `configuration` column
    #[instrument(skip(self))]
    pub async fn save_metadata(&self, metadata: &RuleChainMetaData) -> Result<RuleChainMetaData, DaoError> {
        let config_json = serde_json::to_string(metadata)
            .map_err(|e| DaoError::Database(sqlx::Error::Decode(Box::new(e))))?;
        sqlx::query!(
            "UPDATE rule_chain SET configuration = $1 WHERE id = $2",
            config_json,
            metadata.rule_chain_id
        )
        .execute(&self.pool)
        .await?;
        Ok(metadata.clone())
    }
}
