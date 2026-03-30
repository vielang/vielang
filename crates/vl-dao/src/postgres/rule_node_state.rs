use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use crate::DaoError;

#[derive(Debug, Clone)]
pub struct RuleNodeState {
    pub id: Uuid,
    pub created_time: i64,
    pub rule_node_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub state_data: String,
}

pub struct RuleNodeStateDao {
    pool: PgPool,
}

impl RuleNodeStateDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    #[instrument(skip(self))]
    pub async fn find_by_node_and_entity(
        &self,
        rule_node_id: Uuid,
        entity_id: Uuid,
    ) -> Result<Option<RuleNodeState>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, rule_node_id, entity_type, entity_id, state_data
            FROM rule_node_state
            WHERE rule_node_id = $1 AND entity_id = $2
            "#,
            rule_node_id,
            entity_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| RuleNodeState {
            id:           r.id,
            created_time: r.created_time,
            rule_node_id: r.rule_node_id,
            entity_type:  r.entity_type,
            entity_id:    r.entity_id,
            state_data:   r.state_data,
        }))
    }

    /// UPSERT on (rule_node_id, entity_id)
    #[instrument(skip(self))]
    pub async fn save(&self, state: &RuleNodeState) -> Result<RuleNodeState, DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO rule_node_state (
                id, created_time, rule_node_id, entity_type, entity_id, state_data
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (rule_node_id, entity_id) DO UPDATE SET
                entity_type = EXCLUDED.entity_type,
                state_data  = EXCLUDED.state_data
            "#,
            state.id,
            state.created_time,
            state.rule_node_id,
            state.entity_type,
            state.entity_id,
            state.state_data,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_node_and_entity(state.rule_node_id, state.entity_id)
            .await?
            .ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn delete_by_node(&self, rule_node_id: Uuid) -> Result<(), DaoError> {
        sqlx::query!(
            "DELETE FROM rule_node_state WHERE rule_node_id = $1",
            rule_node_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
