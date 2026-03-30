use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Check whether the originator has a specific relation.
/// True if relation exists, False otherwise.
/// Java: TbCheckRelationNode
/// Config:
/// ```json
/// { "relationType": "Contains", "direction": "FROM", "entityType": "DEVICE" }
/// ```
pub struct CheckRelationNode {
    relation_type: String,
    direction: String,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "relationType")]
    relation_type: String,
    #[serde(default = "default_direction")]
    direction: String,
}

fn default_direction() -> String { "FROM".into() }

impl CheckRelationNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("CheckRelationNode: {}", e)))?;
        Ok(Self { relation_type: cfg.relation_type, direction: cfg.direction })
    }
}

#[async_trait]
impl RuleNode for CheckRelationNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let relations = if self.direction == "FROM" {
            ctx.dao.relation
                .find_by_from_filtered(msg.originator_id, &msg.originator_type, Some(&self.relation_type), None)
                .await?
        } else {
            ctx.dao.relation
                .find_by_to(msg.originator_id, &msg.originator_type)
                .await?
        };

        let has_relation = !relations.is_empty();
        let rel = if has_relation { RelationType::True } else { RelationType::False };
        Ok(vec![(rel, msg)])
    }
}
