use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Enrich message metadata with full entity details (name, type, label, etc.)
/// from the originator entity or a related entity.
/// Java: TbGetEntityDetailsNode
/// Relations: Success, Failure (entity not found)
/// Config:
/// ```json
/// {
///   "entityType": "ORIGINATOR",  // or "CUSTOMER", "TENANT", "RELATED"
///   "detailsList": ["name", "type", "label", "additionalInfo"],
///   "fetchTo": "METADATA"        // or "DATA"
/// }
/// ```
pub struct GetEntityDetailsNode {
    entity_source: EntitySource,
    details_list:  Vec<String>,
    fetch_to_data: bool,
}

#[derive(Debug, Clone, Copy)]
enum EntitySource {
    Originator,
    Customer,
    Tenant,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "entityType", default = "default_source")]
    entity_type: String,
    #[serde(rename = "detailsList", default = "default_details")]
    details_list: Vec<String>,
    #[serde(rename = "fetchTo", default = "default_fetch_to")]
    fetch_to: String,
}

fn default_source()   -> String { "ORIGINATOR".into() }
fn default_fetch_to() -> String { "METADATA".into() }
fn default_details()  -> Vec<String> { vec!["name".into(), "type".into()] }

impl GetEntityDetailsNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("GetEntityDetailsNode: {}", e)))?;
        let entity_source = match cfg.entity_type.to_uppercase().as_str() {
            "ORIGINATOR" => EntitySource::Originator,
            "CUSTOMER"   => EntitySource::Customer,
            "TENANT"     => EntitySource::Tenant,
            other        => return Err(RuleEngineError::Config(
                format!("GetEntityDetailsNode: unknown entityType '{}'", other))),
        };
        Ok(Self {
            entity_source,
            details_list: cfg.details_list,
            fetch_to_data: cfg.fetch_to.to_uppercase() == "DATA",
        })
    }

    fn insert(
        out: &mut TbMsg,
        key: &str,
        val: String,
        fetch_to_data: bool,
    ) {
        if fetch_to_data {
            if let Ok(mut obj) = serde_json::from_str::<serde_json::Value>(&out.data) {
                obj[key] = serde_json::Value::String(val);
                out.data = serde_json::to_string(&obj).unwrap_or_else(|_| out.data.clone());
            }
        } else {
            out.metadata.insert(key.to_string(), val);
        }
    }
}

#[async_trait]
impl RuleNode for GetEntityDetailsNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let mut out = msg;

        match self.entity_source {
            EntitySource::Originator => {
                // Fetch device details for DEVICE originator
                if out.originator_type.to_uppercase() == "DEVICE" {
                    match ctx.dao.device.find_by_id(out.originator_id).await? {
                        None => return Ok(vec![(RelationType::Failure, out)]),
                        Some(device) => {
                            for detail in &self.details_list {
                                let val = match detail.as_str() {
                                    "name"  => device.name.clone(),
                                    "type"  => device.device_type.clone(),
                                    "label" => device.label.clone().unwrap_or_default(),
                                    "id"    => device.id.to_string(),
                                    _       => continue,
                                };
                                Self::insert(&mut out, &format!("originator_{}", detail), val, self.fetch_to_data);
                            }
                        }
                    }
                } else if out.originator_type.to_uppercase() == "ASSET" {
                    match ctx.dao.asset.find_by_id(out.originator_id).await? {
                        None => return Ok(vec![(RelationType::Failure, out)]),
                        Some(asset) => {
                            for detail in &self.details_list {
                                let val = match detail.as_str() {
                                    "name"  => asset.name.clone(),
                                    "type"  => asset.asset_type.clone(),
                                    "label" => asset.label.clone().unwrap_or_default(),
                                    "id"    => asset.id.to_string(),
                                    _       => continue,
                                };
                                Self::insert(&mut out, &format!("originator_{}", detail), val, self.fetch_to_data);
                            }
                        }
                    }
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
                    Some(customer) => {
                        for detail in &self.details_list {
                            let val = match detail.as_str() {
                                "name"  | "title" => customer.title.clone(),
                                "id"              => customer.id.to_string(),
                                _                 => continue,
                            };
                            Self::insert(&mut out, &format!("customer_{}", detail), val, self.fetch_to_data);
                        }
                    }
                }
            }
            EntitySource::Tenant => {
                match ctx.dao.tenant.find_by_id(ctx.tenant_id).await? {
                    None => return Ok(vec![(RelationType::Failure, out)]),
                    Some(tenant) => {
                        for detail in &self.details_list {
                            let val = match detail.as_str() {
                                "name"  | "title" => tenant.title.clone(),
                                "id"              => tenant.id.to_string(),
                                _                 => continue,
                            };
                            Self::insert(&mut out, &format!("tenant_{}", detail), val, self.fetch_to_data);
                        }
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
    fn parses_originator_config() {
        let node = GetEntityDetailsNode::new(&json!({
            "entityType": "ORIGINATOR",
            "detailsList": ["name", "type"]
        })).unwrap();
        assert!(!node.fetch_to_data);
        assert_eq!(node.details_list.len(), 2);
    }

    #[test]
    fn unknown_entity_type_is_error() {
        assert!(GetEntityDetailsNode::new(&json!({ "entityType": "RULE_CHAIN" })).is_err());
    }
}
