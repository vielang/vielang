use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Delete a relation between originator and a target entity.
/// Java: TbDeleteRelationNode
/// Config same as CreateRelationNode.
pub struct DeleteRelationNode {
    relation_type: String,
    direction: String,
    entity_type: String,
    entity_id_key: String,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "relationType")]
    relation_type: String,
    #[serde(default = "default_direction")]
    direction: String,
    #[serde(rename = "entityType", default)]
    entity_type: String,
    #[serde(rename = "entityIdKey", default = "default_entity_id_key")]
    entity_id_key: String,
}

fn default_direction() -> String { "FROM".into() }
fn default_entity_id_key() -> String { "entityId".into() }

impl DeleteRelationNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("DeleteRelationNode: {}", e)))?;
        Ok(Self {
            relation_type: cfg.relation_type,
            direction: cfg.direction,
            entity_type: cfg.entity_type,
            entity_id_key: cfg.entity_id_key,
        })
    }
}

#[async_trait]
impl RuleNode for DeleteRelationNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let target_id_str = msg.metadata.get(&self.entity_id_key)
            .cloned()
            .unwrap_or_default();
        let target_id = match Uuid::parse_str(&target_id_str) {
            Ok(id) => id,
            Err(_) => return Ok(vec![(RelationType::Failure, msg)]),
        };

        let (from_id, from_type, to_id, to_type) = if self.direction == "FROM" {
            (msg.originator_id, msg.originator_type.clone(), target_id, self.entity_type.clone())
        } else {
            (target_id, self.entity_type.clone(), msg.originator_id, msg.originator_type.clone())
        };

        ctx.dao.relation.delete(from_id, &from_type, to_id, &to_type, &self.relation_type, "COMMON").await?;
        Ok(vec![(RelationType::Success, msg)])
    }
}
