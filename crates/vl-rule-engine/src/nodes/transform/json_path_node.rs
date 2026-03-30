use async_trait::async_trait;
use jsonpath_rust::{JsonPath, JsonPathValue};
use serde::Deserialize;
use std::str::FromStr;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Transforms incoming message body using a JSONPath expression.
///
/// Config JSON:
/// ```json
/// { "jsonPath": "$.temperature" }
/// ```
///
/// - Default path `"$"` → pass-through (no transformation)
/// - Single match  → msg.data replaced with that value serialised
/// - Multiple matches → msg.data replaced with a JSON array of matches
/// - No match → routes to `Failure` with `error` in metadata
///
/// Compatible with ThingsBoard Java `TbJsonPathNode` (Jayway JsonPath).
pub struct JsonPathNode {
    /// None when jsonPath == "$" (pass-through)
    path: Option<JsonPath<serde_json::Value>>,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "jsonPath", default = "default_path")]
    json_path: String,
}

fn default_path() -> String {
    "$".to_string()
}

impl JsonPathNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("JsonPathNode: {e}")))?;

        if cfg.json_path == "$" {
            return Ok(Self { path: None });
        }

        let path = JsonPath::from_str(&cfg.json_path)
            .map_err(|e| RuleEngineError::Config(
                format!("JsonPathNode: invalid path '{}': {e}", cfg.json_path)
            ))?;

        Ok(Self { path: Some(path) })
    }
}

#[async_trait]
impl RuleNode for JsonPathNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let Some(ref path) = self.path else {
            return Ok(vec![(RelationType::Success, msg)]);
        };

        let data: serde_json::Value = serde_json::from_str(&msg.data)
            .map_err(|e| RuleEngineError::Processing(
                format!("JsonPathNode: invalid JSON: {e}")
            ))?;

        // Collect owned values from JsonPathValue enum
        let matches: Vec<serde_json::Value> = path
            .find_slice(&data)
            .into_iter()
            .filter_map(|v| match v {
                JsonPathValue::Slice(val, _) => Some(val.clone()),
                JsonPathValue::NewValue(val) => Some(val),
                JsonPathValue::NoValue => None,
            })
            .collect();

        if matches.is_empty() {
            let mut out = msg;
            out.metadata.insert(
                "error".into(),
                "PathNotFoundException: no match found".into(),
            );
            return Ok(vec![(RelationType::Failure, out)]);
        }

        let result_str = if matches.len() == 1 {
            serde_json::to_string(&matches[0])
                .map_err(|e| RuleEngineError::Processing(e.to_string()))?
        } else {
            serde_json::to_string(&matches)
                .map_err(|e| RuleEngineError::Processing(e.to_string()))?
        };

        let mut out = msg;
        out.data = result_str;
        Ok(vec![(RelationType::Success, out)])
    }
}
