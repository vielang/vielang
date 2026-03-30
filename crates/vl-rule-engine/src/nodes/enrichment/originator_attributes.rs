use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::{AttributeScope, TbMsg};
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Fetch attributes of the message originator and enrich message metadata or data.
/// Config JSON:
/// ```json
/// {
///   "attrMapping": { "temperature": "ss_temperature" },
///   "fetchTo": "METADATA",
///   "tellFailureIfAbsent": true
/// }
/// ```
pub struct OriginatorAttributesNode {
    attr_mapping:           Vec<(String, String)>, // source_attr → target_key
    fetch_to:               FetchTo,
    tell_failure_if_absent: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum FetchTo {
    Metadata,
    Data,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "attrMapping", default)]
    attr_mapping: std::collections::HashMap<String, String>,
    #[serde(rename = "fetchTo", default = "default_fetch_to")]
    fetch_to: String,
    #[serde(rename = "tellFailureIfAbsent", default)]
    tell_failure_if_absent: bool,
}

fn default_fetch_to() -> String { "METADATA".to_string() }

impl OriginatorAttributesNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("OriginatorAttributesNode: {}", e)))?;
        let fetch_to = if cfg.fetch_to == "DATA" { FetchTo::Data } else { FetchTo::Metadata };
        Ok(Self {
            attr_mapping: cfg.attr_mapping.into_iter().collect(),
            fetch_to,
            tell_failure_if_absent: cfg.tell_failure_if_absent,
        })
    }
}

#[async_trait]
impl RuleNode for OriginatorAttributesNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let source_keys: Vec<String> = self.attr_mapping.iter().map(|(k, _)| k.clone()).collect();

        // Look up key IDs from dictionary
        let key_ids_map = ctx.dao.kv.lookup_key_ids(&source_keys).await?;

        // Fetch from all scopes (SERVER, CLIENT, SHARED)
        let mut found: std::collections::HashMap<i32, String> = std::collections::HashMap::new();
        for scope in [AttributeScope::ServerScope, AttributeScope::ClientScope, AttributeScope::SharedScope] {
            let attrs = ctx.dao.kv.find_attributes(msg.originator_id, scope, None).await?;
            for attr in attrs {
                let val = attr_to_string(&attr);
                found.entry(attr.attribute_key).or_insert(val);
            }
        }

        let mut out = msg;
        let mut missing = false;

        for (source_key, target_key) in &self.attr_mapping {
            if let Some(&key_id) = key_ids_map.get(source_key) {
                if let Some(val) = found.get(&key_id) {
                    match self.fetch_to {
                        FetchTo::Metadata => { out.metadata.insert(target_key.clone(), val.clone()); }
                        FetchTo::Data => {
                            // Merge into JSON data
                            if let Ok(mut obj) = serde_json::from_str::<serde_json::Value>(&out.data) {
                                obj[target_key] = serde_json::Value::String(val.clone());
                                out.data = serde_json::to_string(&obj).unwrap_or(out.data);
                            }
                        }
                    }
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
