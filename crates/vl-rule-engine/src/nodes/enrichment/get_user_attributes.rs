use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::{AttributeScope, TbMsg};
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Fetch attributes of a User entity related to the originator and add to metadata.
/// Looks for USER entities connected via the configured relation type.
/// Java: Covers user-side of TbGetRelatedAttributeNode
/// Relations: Success, Failure (no related user or attributes missing)
/// Config:
/// ```json
/// {
///   "relationType": "ManagedBy",          // relation type connecting originator to USER
///   "attrMapping": {
///     "email": "userEmail",
///     "firstName": "userFirstName"
///   },
///   "attrScope": "SERVER_SCOPE",          // SERVER_SCOPE | CLIENT_SCOPE | SHARED_SCOPE
///   "tellFailureIfAbsent": false
/// }
/// ```
pub struct GetUserAttributesNode {
    relation_type:          String,
    attr_mapping:           Vec<(String, String)>,
    attr_scope:             AttributeScope,
    tell_failure_if_absent: bool,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "relationType", default = "default_relation")]
    relation_type: String,
    #[serde(rename = "attrMapping", default)]
    attr_mapping: std::collections::HashMap<String, String>,
    #[serde(rename = "attrScope", default = "default_scope")]
    attr_scope: String,
    #[serde(rename = "tellFailureIfAbsent", default)]
    tell_failure_if_absent: bool,
}

fn default_relation() -> String { "ManagedBy".into() }
fn default_scope() -> String { "SERVER_SCOPE".into() }

fn parse_scope(s: &str) -> AttributeScope {
    match s.to_uppercase().as_str() {
        "CLIENT_SCOPE" => AttributeScope::ClientScope,
        "SHARED_SCOPE" => AttributeScope::SharedScope,
        _              => AttributeScope::ServerScope,
    }
}

impl GetUserAttributesNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("GetUserAttributesNode: {}", e)))?;
        Ok(Self {
            relation_type:          cfg.relation_type,
            attr_mapping:           cfg.attr_mapping.into_iter().collect(),
            attr_scope:             parse_scope(&cfg.attr_scope),
            tell_failure_if_absent: cfg.tell_failure_if_absent,
        })
    }
}

#[async_trait]
impl RuleNode for GetUserAttributesNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        // Find USER entities related FROM the originator via the configured relation
        let relations = ctx.dao.relation
            .find_by_from(msg.originator_id, &msg.originator_type)
            .await?;

        let user_id = relations.into_iter()
            .find(|r| {
                r.relation_type == self.relation_type
                    && format!("{:?}", r.to_type).to_uppercase() == "USER"
            })
            .map(|r| r.to_id);

        let mut out = msg;

        let user_id = match user_id {
            Some(id) => id,
            None => {
                if self.tell_failure_if_absent {
                    return Ok(vec![(RelationType::Failure, out)]);
                }
                return Ok(vec![(RelationType::Success, out)]);
            }
        };

        let source_keys: Vec<String> = self.attr_mapping.iter().map(|(k, _)| k.clone()).collect();
        let key_ids = ctx.dao.kv.lookup_key_ids(&source_keys).await?;

        let attrs = ctx.dao.kv.find_attributes(user_id, self.attr_scope, None).await?;
        let found: std::collections::HashMap<i32, String> = attrs.into_iter()
            .map(|a| (a.attribute_key, attr_to_string(&a)))
            .collect();

        let mut any_missing = false;
        for (source_key, target_key) in &self.attr_mapping {
            if let Some(&key_id) = key_ids.get(source_key) {
                if let Some(val) = found.get(&key_id) {
                    out.metadata.insert(target_key.clone(), val.clone());
                } else {
                    any_missing = true;
                }
            } else {
                any_missing = true;
            }
        }

        if any_missing && self.tell_failure_if_absent {
            Ok(vec![(RelationType::Failure, out)])
        } else {
            Ok(vec![(RelationType::Success, out)])
        }
    }
}

fn attr_to_string(attr: &vl_core::entities::AttributeKvEntry) -> String {
    if let Some(v) = attr.bool_v          { return v.to_string(); }
    if let Some(v) = attr.long_v          { return v.to_string(); }
    if let Some(v) = attr.dbl_v           { return v.to_string(); }
    if let Some(ref v) = attr.str_v       { return v.clone(); }
    if let Some(ref v) = attr.json_v      { return v.to_string(); }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_config() {
        let node = GetUserAttributesNode::new(&json!({
            "relationType": "ManagedBy",
            "attrMapping": { "email": "userEmail" },
            "attrScope": "SERVER_SCOPE"
        })).unwrap();
        assert_eq!(node.relation_type, "ManagedBy");
        assert_eq!(node.attr_mapping.len(), 1);
    }

    #[test]
    fn default_relation_is_managed_by() {
        let node = GetUserAttributesNode::new(&json!({ "attrMapping": {} })).unwrap();
        assert_eq!(node.relation_type, "ManagedBy");
    }

    #[test]
    fn empty_config_allowed() {
        assert!(GetUserAttributesNode::new(&json!({})).is_ok());
    }
}
