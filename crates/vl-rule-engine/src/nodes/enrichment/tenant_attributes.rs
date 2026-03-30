use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::{AttributeScope, TbMsg};
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Fetch attributes of the tenant and add to message metadata.
/// Config JSON:
/// ```json
/// {
///   "attrMapping": { "smtpHost": "tenantSmtpHost" }
/// }
/// ```
pub struct TenantAttributesNode {
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

impl TenantAttributesNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("TenantAttributesNode: {}", e)))?;
        Ok(Self {
            attr_mapping: cfg.attr_mapping.into_iter().collect(),
            tell_failure_if_absent: cfg.tell_failure_if_absent,
        })
    }
}

#[async_trait]
impl RuleNode for TenantAttributesNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let source_keys: Vec<String> = self.attr_mapping.iter().map(|(k, _)| k.clone()).collect();
        let key_ids_map = ctx.dao.kv.lookup_key_ids(&source_keys).await?;

        let attrs = ctx.dao.kv.find_attributes(ctx.tenant_id, AttributeScope::ServerScope, None).await?;
        let found: std::collections::HashMap<i32, String> = attrs.into_iter()
            .map(|a| (a.attribute_key, attr_to_string(&a)))
            .collect();

        let mut out = msg;
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
