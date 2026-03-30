use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Copy a value from the message payload (or metadata) and write it back
/// into the message metadata under a new key name.
/// Java: TbAssignAttributeNode
/// Use case: extract a computed field from data and make it accessible as metadata
///           for downstream nodes (e.g. for template rendering in send_email).
/// Relations: Success, Failure (source key missing)
/// Config:
/// ```json
/// {
///   "mapping": [
///     { "sourceKey": "temperature",  "targetKey": "ss_temperature", "fromData": true  },
///     { "sourceKey": "deviceName",   "targetKey": "device",         "fromData": false }
///   ],
///   "tellFailureIfAbsent": false
/// }
/// ```
pub struct AssignAttributeNode {
    mapping: Vec<KeyMapping>,
    tell_failure_if_absent: bool,
}

#[derive(Debug, Clone)]
struct KeyMapping {
    source_key: String,
    target_key: String,
    from_data: bool, // true = read from msg.data JSON, false = read from msg.metadata
}

#[derive(Deserialize)]
struct RawMapping {
    #[serde(rename = "sourceKey")]
    source_key: String,
    #[serde(rename = "targetKey")]
    target_key: String,
    #[serde(rename = "fromData", default = "default_true")]
    from_data: bool,
}

#[derive(Deserialize)]
struct Config {
    #[serde(default)]
    mapping: Vec<RawMapping>,
    #[serde(rename = "tellFailureIfAbsent", default)]
    tell_failure_if_absent: bool,
}

fn default_true() -> bool { true }

impl AssignAttributeNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("AssignAttributeNode: {}", e)))?;
        Ok(Self {
            mapping: cfg.mapping.into_iter().map(|m| KeyMapping {
                source_key: m.source_key,
                target_key: m.target_key,
                from_data: m.from_data,
            }).collect(),
            tell_failure_if_absent: cfg.tell_failure_if_absent,
        })
    }
}

#[async_trait]
impl RuleNode for AssignAttributeNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let data_json: Option<serde_json::Value> = serde_json::from_str(&msg.data).ok();
        let mut out = msg;
        let mut any_missing = false;

        for km in &self.mapping {
            let value_opt = if km.from_data {
                data_json.as_ref()
                    .and_then(|v| v.get(&km.source_key))
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
            } else {
                out.metadata.get(&km.source_key).cloned()
            };

            match value_opt {
                Some(val) => { out.metadata.insert(km.target_key.clone(), val); }
                None => { any_missing = true; }
            }
        }

        if any_missing && self.tell_failure_if_absent {
            Ok(vec![(RelationType::Failure, out)])
        } else {
            Ok(vec![(RelationType::Success, out)])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_mapping() {
        let node = AssignAttributeNode::new(&json!({
            "mapping": [{
                "sourceKey": "temperature",
                "targetKey": "ss_temperature",
                "fromData": true
            }]
        })).unwrap();
        assert_eq!(node.mapping.len(), 1);
        assert!(node.mapping[0].from_data);
    }

    #[test]
    fn empty_mapping_ok() {
        let node = AssignAttributeNode::new(&json!({})).unwrap();
        assert!(node.mapping.is_empty());
    }
}
