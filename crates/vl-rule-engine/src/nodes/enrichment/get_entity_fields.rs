use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Fetch specific fields from an entity and write them to message metadata.
/// More flexible than OriginatorFieldsNode: can target ORIGINATOR, CUSTOMER, or TENANT.
/// Java: TbGetEntityFieldsNode (ThingsBoard PE)
/// Relations: Success, Failure (entity not found)
/// Config:
/// ```json
/// {
///   "entitySource": "ORIGINATOR",      // ORIGINATOR | CUSTOMER | TENANT
///   "fieldsMapping": {
///     "name":        "deviceName",
///     "label":       "deviceLabel",
///     "type":        "deviceType",
///     "id":          "deviceId",
///     "createdTime": "deviceCreatedTime"
///   }
/// }
/// ```
pub struct GetEntityFieldsNode {
    entity_source: EntitySource,
    fields_mapping: Vec<(String, String)>,  // source_field → target_metadata_key
}

#[derive(Debug, Clone, Copy)]
enum EntitySource {
    Originator,
    Customer,
    Tenant,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "entitySource", default = "default_source")]
    entity_source: String,
    #[serde(rename = "fieldsMapping", default)]
    fields_mapping: std::collections::HashMap<String, String>,
}

fn default_source() -> String { "ORIGINATOR".into() }

impl GetEntityFieldsNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("GetEntityFieldsNode: {}", e)))?;
        let entity_source = match cfg.entity_source.to_uppercase().as_str() {
            "ORIGINATOR" => EntitySource::Originator,
            "CUSTOMER"   => EntitySource::Customer,
            "TENANT"     => EntitySource::Tenant,
            other        => return Err(RuleEngineError::Config(
                format!("GetEntityFieldsNode: unknown entitySource '{}'", other))),
        };
        Ok(Self {
            entity_source,
            fields_mapping: cfg.fields_mapping.into_iter().collect(),
        })
    }

    fn apply_fields(
        out: &mut TbMsg,
        mapping: &[(String, String)],
        name: &str,
        label: &str,
        entity_type: &str,
        id: &str,
        created_time: i64,
    ) {
        for (source_field, target_key) in mapping {
            let val = match source_field.as_str() {
                "name"        => name.to_string(),
                "label"       => label.to_string(),
                "type"        => entity_type.to_string(),
                "id"          => id.to_string(),
                "createdTime" => created_time.to_string(),
                _             => continue,
            };
            out.metadata.insert(target_key.clone(), val);
        }
    }
}

#[async_trait]
impl RuleNode for GetEntityFieldsNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let mut out = msg;

        match self.entity_source {
            EntitySource::Originator => {
                match out.originator_type.to_uppercase().as_str() {
                    "DEVICE" => {
                        match ctx.dao.device.find_by_id(out.originator_id).await? {
                            None => return Ok(vec![(RelationType::Failure, out)]),
                            Some(d) => Self::apply_fields(
                                &mut out, &self.fields_mapping,
                                &d.name, d.label.as_deref().unwrap_or(""),
                                "DEVICE", &d.id.to_string(), d.created_time,
                            ),
                        }
                    }
                    "ASSET" => {
                        match ctx.dao.asset.find_by_id(out.originator_id).await? {
                            None => return Ok(vec![(RelationType::Failure, out)]),
                            Some(a) => Self::apply_fields(
                                &mut out, &self.fields_mapping,
                                &a.name, a.label.as_deref().unwrap_or(""),
                                "ASSET", &a.id.to_string(), a.created_time,
                            ),
                        }
                    }
                    "CUSTOMER" => {
                        match ctx.dao.customer.find_by_id(out.originator_id).await? {
                            None => return Ok(vec![(RelationType::Failure, out)]),
                            Some(c) => Self::apply_fields(
                                &mut out, &self.fields_mapping,
                                &c.title, "",
                                "CUSTOMER", &c.id.to_string(), c.created_time,
                            ),
                        }
                    }
                    _ => {}  // Unknown originator type — pass through
                }
            }
            EntitySource::Customer => {
                let customer_id = match out.metadata.get("customerId")
                    .and_then(|s| s.parse::<uuid::Uuid>().ok())
                {
                    Some(id) => id,
                    None => return Ok(vec![(RelationType::Failure, out)]),
                };
                match ctx.dao.customer.find_by_id(customer_id).await? {
                    None => return Ok(vec![(RelationType::Failure, out)]),
                    Some(c) => Self::apply_fields(
                        &mut out, &self.fields_mapping,
                        &c.title, "",
                        "CUSTOMER", &c.id.to_string(), c.created_time,
                    ),
                }
            }
            EntitySource::Tenant => {
                match ctx.dao.tenant.find_by_id(ctx.tenant_id).await? {
                    None => return Ok(vec![(RelationType::Failure, out)]),
                    Some(t) => Self::apply_fields(
                        &mut out, &self.fields_mapping,
                        &t.title, "",
                        "TENANT", &t.id.to_string(), t.created_time,
                    ),
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
    fn parses_originator_config() {
        let node = GetEntityFieldsNode::new(&json!({
            "entitySource": "ORIGINATOR",
            "fieldsMapping": { "name": "deviceName", "type": "deviceType" }
        })).unwrap();
        assert_eq!(node.fields_mapping.len(), 2);
    }

    #[test]
    fn defaults_to_originator() {
        assert!(GetEntityFieldsNode::new(&json!({ "fieldsMapping": {} })).is_ok());
    }

    #[test]
    fn unknown_entity_source_is_error() {
        assert!(GetEntityFieldsNode::new(&json!({
            "entitySource": "RULE_CHAIN"
        })).is_err());
    }
}
