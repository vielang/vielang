use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use crate::DaoError;

#[derive(Debug, Clone)]
pub struct RuleNodeRow {
    pub id: Uuid,
    pub created_time: i64,
    pub rule_chain_id: Option<Uuid>,
    pub additional_info: Option<String>,
    pub configuration_version: i32,
    pub configuration: Option<String>,
    pub type_: Option<String>,
    pub name: Option<String>,
    pub debug_settings: Option<String>,
    pub singleton_mode: bool,
    pub queue_name: Option<String>,
    pub external_id: Option<Uuid>,
}

pub struct RuleNodeDao {
    pool: PgPool,
}

impl RuleNodeDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<RuleNodeRow>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, rule_chain_id, additional_info,
                   configuration_version, configuration,
                   type as "type_", name, debug_settings,
                   singleton_mode, queue_name, external_id
            FROM rule_node WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| RuleNodeRow {
            id:                    r.id,
            created_time:          r.created_time,
            rule_chain_id:         r.rule_chain_id,
            additional_info:       r.additional_info,
            configuration_version: r.configuration_version,
            configuration:         r.configuration,
            type_:                 r.type_,
            name:                  r.name,
            debug_settings:        r.debug_settings,
            singleton_mode:        r.singleton_mode,
            queue_name:            r.queue_name,
            external_id:           r.external_id,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_by_chain(&self, rule_chain_id: Uuid) -> Result<Vec<RuleNodeRow>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, rule_chain_id, additional_info,
                   configuration_version, configuration,
                   type as "type_", name, debug_settings,
                   singleton_mode, queue_name, external_id
            FROM rule_node
            WHERE rule_chain_id = $1
            ORDER BY created_time
            "#,
            rule_chain_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| RuleNodeRow {
            id:                    r.id,
            created_time:          r.created_time,
            rule_chain_id:         r.rule_chain_id,
            additional_info:       r.additional_info,
            configuration_version: r.configuration_version,
            configuration:         r.configuration,
            type_:                 r.type_,
            name:                  r.name,
            debug_settings:        r.debug_settings,
            singleton_mode:        r.singleton_mode,
            queue_name:            r.queue_name,
            external_id:           r.external_id,
        }).collect())
    }

    #[instrument(skip(self))]
    pub async fn save(&self, node: &RuleNodeRow) -> Result<RuleNodeRow, DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO rule_node (
                id, created_time, rule_chain_id, additional_info,
                configuration_version, configuration, type, name,
                debug_settings, singleton_mode, queue_name, external_id
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
            ON CONFLICT (id) DO UPDATE SET
                rule_chain_id         = EXCLUDED.rule_chain_id,
                additional_info       = EXCLUDED.additional_info,
                configuration_version = EXCLUDED.configuration_version,
                configuration         = EXCLUDED.configuration,
                type                  = EXCLUDED.type,
                name                  = EXCLUDED.name,
                debug_settings        = EXCLUDED.debug_settings,
                singleton_mode        = EXCLUDED.singleton_mode,
                queue_name            = EXCLUDED.queue_name,
                external_id           = EXCLUDED.external_id
            "#,
            node.id,
            node.created_time,
            node.rule_chain_id,
            node.additional_info,
            node.configuration_version,
            node.configuration,
            node.type_,
            node.name,
            node.debug_settings,
            node.singleton_mode,
            node.queue_name,
            node.external_id,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(node.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        sqlx::query!("DELETE FROM rule_node WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Delete all rule nodes belonging to a rule chain. Returns the number of rows deleted.
    #[instrument(skip(self))]
    pub async fn delete_by_chain(&self, rule_chain_id: Uuid) -> Result<i64, DaoError> {
        let result = sqlx::query!(
            "DELETE FROM rule_node WHERE rule_chain_id = $1",
            rule_chain_id
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() as i64)
    }
}
