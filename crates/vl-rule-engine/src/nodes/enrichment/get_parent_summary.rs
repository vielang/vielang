use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Traverse entity relations to find the parent entity and enrich message metadata
/// with the parent's fields. Useful for IoT hierarchies: Device → Asset → Building.
/// Java: TbGetParentSummaryNode (ThingsBoard PE)
/// Relations: Success (with or without parent), Failure (DAO error)
/// Config:
/// ```json
/// {
///   "relationType": "Contains",        // relation type to traverse (default: "Contains")
///   "parentEntityType": "ASSET",       // optional: only match parents of this type
///   "tellFailureIfAbsent": false,      // route to Failure if no parent found
///   "fieldsMapping": {
///     "name":  "parentName",
///     "type":  "parentType",
///     "label": "parentLabel",
///     "id":    "parentId"
///   }
/// }
/// ```
pub struct GetParentSummaryNode {
    relation_type:          String,
    parent_entity_type:     Option<String>,
    tell_failure_if_absent: bool,
    fields_mapping:         Vec<(String, String)>,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "relationType", default = "default_relation")]
    relation_type: String,
    #[serde(rename = "parentEntityType")]
    parent_entity_type: Option<String>,
    #[serde(rename = "tellFailureIfAbsent", default)]
    tell_failure_if_absent: bool,
    #[serde(rename = "fieldsMapping", default)]
    fields_mapping: std::collections::HashMap<String, String>,
}

fn default_relation() -> String { "Contains".into() }

impl GetParentSummaryNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("GetParentSummaryNode: {}", e)))?;
        Ok(Self {
            relation_type:          cfg.relation_type,
            parent_entity_type:     cfg.parent_entity_type.map(|s| s.to_uppercase()),
            tell_failure_if_absent: cfg.tell_failure_if_absent,
            fields_mapping:         cfg.fields_mapping.into_iter().collect(),
        })
    }
}

#[async_trait]
impl RuleNode for GetParentSummaryNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        // Find entities that have a relation TO the originator (parent "Contains" child)
        let relations = ctx.dao.relation
            .find_by_to(msg.originator_id, &msg.originator_type)
            .await?;

        // Filter by relation type and optionally by parent entity type
        let parent = relations.into_iter().find(|r| {
            let rel_match = r.relation_type == self.relation_type;
            let type_match = self.parent_entity_type.as_ref()
                .map(|pt| format!("{:?}", r.from_type).to_uppercase() == *pt)
                .unwrap_or(true);
            rel_match && type_match
        });

        let mut out = msg;

        match parent {
            None => {
                if self.tell_failure_if_absent {
                    return Ok(vec![(RelationType::Failure, out)]);
                }
            }
            Some(rel) => {
                // Enrich with parent entity fields based on from_type
                let from_type = format!("{:?}", rel.from_type).to_uppercase();
                match from_type.as_str() {
                    "ASSET" => {
                        if let Some(asset) = ctx.dao.asset.find_by_id(rel.from_id).await? {
                            for (source_field, target_key) in &self.fields_mapping {
                                let val = match source_field.as_str() {
                                    "name"        => asset.name.clone(),
                                    "type"        => asset.asset_type.clone(),
                                    "label"       => asset.label.clone().unwrap_or_default(),
                                    "id"          => asset.id.to_string(),
                                    "createdTime" => asset.created_time.to_string(),
                                    _             => continue,
                                };
                                out.metadata.insert(target_key.clone(), val);
                            }
                        }
                    }
                    "DEVICE" => {
                        if let Some(device) = ctx.dao.device.find_by_id(rel.from_id).await? {
                            for (source_field, target_key) in &self.fields_mapping {
                                let val = match source_field.as_str() {
                                    "name"        => device.name.clone(),
                                    "type"        => device.device_type.clone(),
                                    "label"       => device.label.clone().unwrap_or_default(),
                                    "id"          => device.id.to_string(),
                                    "createdTime" => device.created_time.to_string(),
                                    _             => continue,
                                };
                                out.metadata.insert(target_key.clone(), val);
                            }
                        }
                    }
                    "CUSTOMER" => {
                        if let Some(customer) = ctx.dao.customer.find_by_id(rel.from_id).await? {
                            for (source_field, target_key) in &self.fields_mapping {
                                let val = match source_field.as_str() {
                                    "name"  | "title" => customer.title.clone(),
                                    "id"              => customer.id.to_string(),
                                    "createdTime"     => customer.created_time.to_string(),
                                    _                 => continue,
                                };
                                out.metadata.insert(target_key.clone(), val);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(vec![(RelationType::Success, out)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_config() {
        let node = GetParentSummaryNode::new(&json!({
            "relationType": "Contains",
            "parentEntityType": "ASSET",
            "fieldsMapping": { "name": "parentName" }
        })).unwrap();
        assert_eq!(node.relation_type, "Contains");
        assert_eq!(node.parent_entity_type, Some("ASSET".into()));
        assert_eq!(node.fields_mapping.len(), 1);
    }

    #[test]
    fn default_relation_type_is_contains() {
        let node = GetParentSummaryNode::new(&json!({ "fieldsMapping": {} })).unwrap();
        assert_eq!(node.relation_type, "Contains");
    }

    #[test]
    fn empty_config_allowed() {
        assert!(GetParentSummaryNode::new(&json!({})).is_ok());
    }
}
