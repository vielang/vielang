use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::{AttributeScope, TbMsg};
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Fetch attributes of the device/asset customer and add to message metadata.
/// Config JSON:
/// ```json
/// {
///   "attrMapping": { "email": "customerEmail" }
/// }
/// ```
pub struct CustomerAttributesNode {
    attr_mapping:           Vec<(String, String)>,
    tell_failure_if_absent: bool,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "attrMapping", default)]
    attr_mapping: std::collections::HashMap<String, String>,
    #[serde(rename = "tellFailureIfAbsent", default)]
    tell_failure_if_absent: bool,
}

impl CustomerAttributesNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("CustomerAttributesNode: {}", e)))?;
        Ok(Self {
            attr_mapping: cfg.attr_mapping.into_iter().collect(),
            tell_failure_if_absent: cfg.tell_failure_if_absent,
        })
    }
}

#[async_trait]
impl RuleNode for CustomerAttributesNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        // Resolve customer_id from originator
        let customer_id = match msg.originator_type.to_uppercase().as_str() {
            "DEVICE" => ctx.dao.device.find_by_id(msg.originator_id).await?
                .and_then(|d| d.customer_id),
            "ASSET" => ctx.dao.asset.find_by_id(msg.originator_id).await?
                .and_then(|a| a.customer_id),
            _ => None,
        };

        let mut out = msg;

        let Some(customer_id) = customer_id else {
            if self.tell_failure_if_absent {
                return Ok(vec![(RelationType::Failure, out)]);
            }
            return Ok(vec![(RelationType::Success, out)]);
        };

        let source_keys: Vec<String> = self.attr_mapping.iter().map(|(k, _)| k.clone()).collect();
        let key_ids_map = ctx.dao.kv.lookup_key_ids(&source_keys).await?;

        let attrs = ctx.dao.kv.find_attributes(customer_id, AttributeScope::ServerScope, None).await?;
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
