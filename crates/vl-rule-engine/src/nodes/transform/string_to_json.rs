use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Convert a JSON string field value into an embedded JSON object/array.
/// Java: TbStringToJsonNode
/// Use case: device sends `{ "payload": "{\"temp\":22}" }` → `{ "payload": {"temp": 22} }`
/// Relations: Success, Failure (field missing or not valid JSON string)
/// Config:
/// ```json
/// { "fromMetadata": false, "fieldName": "payload", "failOnError": true }
/// ```
pub struct StringToJsonNode {
    field_name:    String,
    from_metadata: bool,
    fail_on_error: bool,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "fieldName")]
    field_name: String,
    #[serde(rename = "fromMetadata", default)]
    from_metadata: bool,
    #[serde(rename = "failOnError", default = "default_true")]
    fail_on_error: bool,
}

fn default_true() -> bool { true }

impl StringToJsonNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("StringToJsonNode: {}", e)))?;
        Ok(Self {
            field_name: cfg.field_name,
            from_metadata: cfg.from_metadata,
            fail_on_error: cfg.fail_on_error,
        })
    }
}

#[async_trait]
impl RuleNode for StringToJsonNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let mut out = msg;

        if self.from_metadata {
            // Convert a metadata string value to JSON and embed it in the data payload
            let raw_str = match out.metadata.get(&self.field_name) {
                Some(v) => v.clone(),
                None => {
                    if self.fail_on_error {
                        out.metadata.insert("error".into(),
                            format!("StringToJsonNode: metadata key '{}' not found", self.field_name));
                        return Ok(vec![(RelationType::Failure, out)]);
                    }
                    return Ok(vec![(RelationType::Success, out)]);
                }
            };
            match serde_json::from_str::<serde_json::Value>(&raw_str) {
                Ok(parsed) => {
                    if let Ok(mut data) = serde_json::from_str::<serde_json::Value>(&out.data) {
                        data[&self.field_name] = parsed;
                        out.data = serde_json::to_string(&data).unwrap_or(out.data);
                    }
                }
                Err(e) => {
                    if self.fail_on_error {
                        out.metadata.insert("error".into(), format!("StringToJsonNode: {}", e));
                        return Ok(vec![(RelationType::Failure, out)]);
                    }
                }
            }
        } else {
            // Convert a string field inside the JSON data payload
            let mut data: serde_json::Value = match serde_json::from_str(&out.data) {
                Ok(v) => v,
                Err(e) => {
                    if self.fail_on_error {
                        out.metadata.insert("error".into(), format!("StringToJsonNode: {}", e));
                        return Ok(vec![(RelationType::Failure, out)]);
                    }
                    return Ok(vec![(RelationType::Success, out)]);
                }
            };

            let raw_str = match data.get(&self.field_name).and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => {
                    if self.fail_on_error {
                        out.metadata.insert("error".into(),
                            format!("StringToJsonNode: field '{}' not found or not a string", self.field_name));
                        return Ok(vec![(RelationType::Failure, out)]);
                    }
                    return Ok(vec![(RelationType::Success, out)]);
                }
            };

            match serde_json::from_str::<serde_json::Value>(&raw_str) {
                Ok(parsed) => {
                    data[&self.field_name] = parsed;
                    out.data = serde_json::to_string(&data).unwrap_or(out.data);
                }
                Err(e) => {
                    if self.fail_on_error {
                        out.metadata.insert("error".into(), format!("StringToJsonNode: {}", e));
                        return Ok(vec![(RelationType::Failure, out)]);
                    }
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
        let node = StringToJsonNode::new(&json!({ "fieldName": "payload" })).unwrap();
        assert_eq!(node.field_name, "payload");
        assert!(!node.from_metadata);
        assert!(node.fail_on_error);
    }

    #[test]
    fn missing_field_name_is_error() {
        assert!(StringToJsonNode::new(&json!({})).is_err());
    }
}
