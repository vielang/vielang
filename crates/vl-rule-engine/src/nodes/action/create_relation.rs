use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;
use vl_core::entities::{EntityRelation, EntityType, RelationTypeGroup, TbMsg};
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Create a relation between originator and a target entity.
/// Java: TbCreateRelationNode
/// Config:
/// ```json
/// {
///   "relationType": "Contains",
///   "direction": "FROM",
///   "entityType": "ASSET",
///   "entityIdKey": "entityId"
/// }
/// ```
pub struct CreateRelationNode {
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
    /// Metadata key containing the target entity ID (UUID string)
    #[serde(rename = "entityIdKey", default = "default_entity_id_key")]
    entity_id_key: String,
}

fn default_direction() -> String { "FROM".into() }
fn default_entity_id_key() -> String { "entityId".into() }

impl CreateRelationNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("CreateRelationNode: {}", e)))?;
        Ok(Self {
            relation_type: cfg.relation_type,
            direction: cfg.direction,
            entity_type: cfg.entity_type,
            entity_id_key: cfg.entity_id_key,
        })
    }
}

#[async_trait]
impl RuleNode for CreateRelationNode {
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

        let originator_entity_type = parse_entity_type(&msg.originator_type);
        let target_entity_type = parse_entity_type(&self.entity_type);

        let relation = if self.direction == "FROM" {
            EntityRelation {
                from_id:              msg.originator_id,
                from_type:            originator_entity_type,
                to_id:                target_id,
                to_type:              target_entity_type,
                relation_type:        self.relation_type.clone(),
                relation_type_group:  RelationTypeGroup::Common,
                additional_info:      None,
            }
        } else {
            EntityRelation {
                from_id:              target_id,
                from_type:            target_entity_type,
                to_id:                msg.originator_id,
                to_type:              originator_entity_type,
                relation_type:        self.relation_type.clone(),
                relation_type_group:  RelationTypeGroup::Common,
                additional_info:      None,
            }
        };

        ctx.dao.relation.save(&relation).await?;
        Ok(vec![(RelationType::Success, msg)])
    }
}

fn parse_entity_type(s: &str) -> EntityType {
    match s.to_uppercase().as_str() {
        "TENANT"   => EntityType::Tenant,
        "CUSTOMER" => EntityType::Customer,
        "USER"     => EntityType::User,
        "ASSET"    => EntityType::Asset,
        _          => EntityType::Device,
    }
}
