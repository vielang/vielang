use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::{AttributeScope, TbMsg};
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Fetch attributes of entities related to the originator (via relation graph).
/// Config JSON:
/// ```json
/// {
///   "direction": "FROM",
///   "relationType": "Contains",
///   "entityType": "ASSET",
///   "attrMapping": { "location": "ss_location" }
/// }
/// ```
pub struct RelatedAttributesNode {
    direction:     String,    // "FROM" | "TO"
    relation_type: Option<String>,
    entity_type:   Option<String>,
    attr_mapping:  Vec<(String, String)>,
    tell_failure_if_absent: bool,
}

#[derive(Deserialize)]
struct Config {
    #[serde(default = "default_direction")]
    direction: String,
    #[serde(rename = "relationType")]
    relation_type: Option<String>,
    #[serde(rename = "entityType")]
    entity_type: Option<String>,
    #[serde(rename = "attrMapping", default)]
    attr_mapping: std::collections::HashMap<String, String>,
    #[serde(rename = "tellFailureIfAbsent", default)]
    tell_failure_if_absent: bool,
}

fn default_direction() -> String { "FROM".to_string() }

impl RelatedAttributesNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("RelatedAttributesNode: {}", e)))?;
        Ok(Self {
            direction:              cfg.direction,
            relation_type:         cfg.relation_type,
            entity_type:           cfg.entity_type,
            attr_mapping:          cfg.attr_mapping.into_iter().collect(),
            tell_failure_if_absent: cfg.tell_failure_if_absent,
        })
    }
}

#[async_trait]
impl RuleNode for RelatedAttributesNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        // Find related entities
        let relations = if self.direction.to_uppercase() == "FROM" {
            ctx.dao.relation.find_by_from(msg.originator_id, &msg.originator_type).await?
        } else {
            ctx.dao.relation.find_by_to(msg.originator_id, &msg.originator_type).await?
        };

        // Filter by relation_type and entity_type
        let related_ids: Vec<uuid::Uuid> = relations.into_iter()
            .filter(|r| {
                let type_match = self.relation_type.as_ref()
                    .map(|rt| rt == &r.relation_type)
                    .unwrap_or(true);
                let entity_match = self.entity_type.as_ref()
                    .map(|et| et.to_uppercase() == format!("{:?}", r.to_type).to_uppercase())
                    .unwrap_or(true);
                type_match && entity_match
            })
            .map(|r| if self.direction.to_uppercase() == "FROM" { r.to_id } else { r.from_id })
            .collect();

        let mut out = msg;

        if related_ids.is_empty() {
            if self.tell_failure_if_absent {
                return Ok(vec![(RelationType::Failure, out)]);
            }
            return Ok(vec![(RelationType::Success, out)]);
        }

        let source_keys: Vec<String> = self.attr_mapping.iter().map(|(k, _)| k.clone()).collect();
        let key_ids_map = ctx.dao.kv.lookup_key_ids(&source_keys).await?;

        // Take first matching related entity
        let related_id = related_ids[0];
        let attrs = ctx.dao.kv.find_attributes(related_id, AttributeScope::ServerScope, None).await?;
        let found: std::collections::HashMap<i32, String> = attrs.into_iter()
            .map(|a| (a.attribute_key, attr_to_string(&a)))
            .collect();

        let mut missing = false;
        for (source_key, target_key) in &self.attr_mapping {
            if let Some(&key_id) = key_ids_map.get(source_key) {
                if let Some(val) = found.get(&key_id) {
                    out.metadata.insert(target_key.clone(), val.clone());
                } else {
                    missing = true;
                }
            } else {
                missing = true;
            }
        }

        if missing && self.tell_failure_if_absent {
            Ok(vec![(RelationType::Failure, out)])
        } else {
            Ok(vec![(RelationType::Success, out)])
        }
    }
}

fn attr_to_string(attr: &vl_core::entities::AttributeKvEntry) -> String {
    if let Some(v) = attr.bool_v  { return v.to_string(); }
    if let Some(v) = attr.long_v  { return v.to_string(); }
    if let Some(v) = attr.dbl_v   { return v.to_string(); }
    if let Some(ref v) = attr.str_v  { return v.clone(); }
    if let Some(ref v) = attr.json_v { return v.to_string(); }
    String::new()
}
