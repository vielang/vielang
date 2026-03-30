use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::{AttributeScope, TbMsg};
use vl_dao::PageLink;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Read a value from entity attributes or latest telemetry and write it into
/// the message body or metadata.
/// Java: TbGetValueNode
/// Relations: Success, Failure (value not found and tellFailureIfAbsent=true)
/// Config:
/// ```json
/// {
///   "inputSource": "ATTRIBUTE",        // ATTRIBUTE | TELEMETRY | CONSTANT
///   "inputKey": "calibration_offset",
///   "attributeScope": "SERVER_SCOPE",  // SERVER_SCOPE | CLIENT_SCOPE | SHARED_SCOPE
///   "outputTarget": "MSG_BODY",        // MSG_BODY | METADATA
///   "outputKey": "offset",
///   "defaultValue": "0",
///   "tellFailureIfAbsent": false
/// }
/// ```
pub struct GetValueNode {
    source:                  ValueSource,
    input_key:               String,
    attr_scope:              AttributeScope,
    output_target:           OutputTarget,
    output_key:              String,
    default_value:           Option<String>,
    tell_failure_if_absent:  bool,
}

#[derive(Debug, Clone, Copy)]
enum ValueSource { Attribute, Telemetry, Constant }

#[derive(Debug, Clone, Copy)]
enum OutputTarget { MsgBody, Metadata }

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "inputSource", default = "default_attr")]
    input_source: String,
    #[serde(rename = "inputKey")]
    input_key: String,
    #[serde(rename = "attributeScope", default = "default_scope")]
    attribute_scope: String,
    #[serde(rename = "outputTarget", default = "default_msg_body")]
    output_target: String,
    #[serde(rename = "outputKey")]
    output_key: String,
    #[serde(rename = "defaultValue")]
    default_value: Option<String>,
    #[serde(rename = "tellFailureIfAbsent", default)]
    tell_failure_if_absent: bool,
}

fn default_attr()      -> String { "ATTRIBUTE".into() }
fn default_scope()     -> String { "SERVER_SCOPE".into() }
fn default_msg_body()  -> String { "MSG_BODY".into() }

impl GetValueNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("GetValueNode: {}", e)))?;
        let source = match cfg.input_source.to_uppercase().as_str() {
            "ATTRIBUTE" => ValueSource::Attribute,
            "TELEMETRY" => ValueSource::Telemetry,
            "CONSTANT"  => ValueSource::Constant,
            other       => return Err(RuleEngineError::Config(
                format!("GetValueNode: unknown inputSource '{}'", other))),
        };
        let attr_scope = match cfg.attribute_scope.to_uppercase().as_str() {
            "CLIENT_SCOPE" => AttributeScope::ClientScope,
            "SHARED_SCOPE" => AttributeScope::SharedScope,
            _              => AttributeScope::ServerScope,
        };
        let output_target = if cfg.output_target.to_uppercase() == "METADATA" {
            OutputTarget::Metadata
        } else {
            OutputTarget::MsgBody
        };
        Ok(Self {
            source,
            input_key: cfg.input_key,
            attr_scope,
            output_target,
            output_key: cfg.output_key,
            default_value: cfg.default_value,
            tell_failure_if_absent: cfg.tell_failure_if_absent,
        })
    }

    fn write_value(out: &mut TbMsg, key: &str, val: String, target: OutputTarget) {
        match target {
            OutputTarget::Metadata => { out.metadata.insert(key.to_string(), val); }
            OutputTarget::MsgBody => {
                if let Ok(mut obj) = serde_json::from_str::<serde_json::Value>(&out.data) {
                    // Try to parse as number first
                    if let Ok(n) = val.parse::<f64>() {
                        obj[key] = serde_json::json!(n);
                    } else if let Ok(b) = val.parse::<bool>() {
                        obj[key] = serde_json::json!(b);
                    } else {
                        obj[key] = serde_json::json!(val);
                    }
                    out.data = serde_json::to_string(&obj).unwrap_or_else(|_| out.data.clone());
                }
            }
        }
    }
}

#[async_trait]
impl RuleNode for GetValueNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let mut out = msg;

        let value_opt: Option<String> = match self.source {
            ValueSource::Constant => Some(self.input_key.clone()),

            ValueSource::Attribute => {
                let key_ids = ctx.dao.kv.lookup_key_ids(&[self.input_key.clone()]).await?;
                if let Some(&key_id) = key_ids.get(&self.input_key) {
                    let attrs = ctx.dao.kv
                        .find_attributes(out.originator_id, self.attr_scope, Some(&[key_id]))
                        .await?;
                    attrs.into_iter().next().map(|a| attr_val_to_string(&a))
                } else {
                    None
                }
            }

            ValueSource::Telemetry => {
                let key_ids = ctx.dao.kv.lookup_key_ids(&[self.input_key.clone()]).await?;
                if let Some(&key_id) = key_ids.get(&self.input_key) {
                    let entries = ctx.dao.kv
                        .find_latest(out.originator_id, &[key_id])
                        .await?;
                    entries.into_iter().next().map(|e| ts_val_to_string(&e))
                } else {
                    None
                }
            }
        };

        match value_opt.or_else(|| self.default_value.clone()) {
            Some(val) => {
                Self::write_value(&mut out, &self.output_key, val, self.output_target);
                Ok(vec![(RelationType::Success, out)])
            }
            None => {
                if self.tell_failure_if_absent {
                    out.metadata.insert("error".into(),
                        format!("GetValueNode: key '{}' not found", self.input_key));
                    Ok(vec![(RelationType::Failure, out)])
                } else {
                    Ok(vec![(RelationType::Success, out)])
                }
            }
        }
    }
}

fn attr_val_to_string(a: &vl_core::entities::AttributeKvEntry) -> String {
    if let Some(v) = a.bool_v  { return v.to_string(); }
    if let Some(v) = a.long_v  { return v.to_string(); }
    if let Some(v) = a.dbl_v   { return v.to_string(); }
    if let Some(ref v) = a.str_v  { return v.clone(); }
    if let Some(ref v) = a.json_v { return v.to_string(); }
    String::new()
}

fn ts_val_to_string(e: &vl_core::entities::TsKvEntry) -> String {
    if let Some(v) = e.bool_v  { return v.to_string(); }
    if let Some(v) = e.long_v  { return v.to_string(); }
    if let Some(v) = e.dbl_v   { return v.to_string(); }
    if let Some(ref v) = e.str_v  { return v.clone(); }
    if let Some(ref v) = e.json_v { return v.to_string(); }
    String::new()
}

// suppress unused import
const _: fn() = || { let _ = PageLink::new(0,1); };

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_attribute_config() {
        let node = GetValueNode::new(&json!({
            "inputSource": "ATTRIBUTE",
            "inputKey": "calibration_offset",
            "outputKey": "offset"
        })).unwrap();
        assert!(matches!(node.source, ValueSource::Attribute));
        assert_eq!(node.input_key, "calibration_offset");
    }

    #[test]
    fn unknown_source_is_error() {
        assert!(GetValueNode::new(&json!({
            "inputSource": "DATABASE",
            "inputKey": "x",
            "outputKey": "y"
        })).is_err());
    }

    #[test]
    fn constant_source() {
        let node = GetValueNode::new(&json!({
            "inputSource": "CONSTANT",
            "inputKey": "273.15",
            "outputKey": "kelvin_offset"
        })).unwrap();
        assert!(matches!(node.source, ValueSource::Constant));
    }
}
