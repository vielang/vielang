use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Fetch customer details and enrich message metadata.
/// Java: TbGetCustomerDetailsNode
/// Config:
/// ```json
/// { "detailsList": ["title", "email", "phone"], "addToMetadata": true }
/// ```
pub struct GetCustomerDetailsNode {
    details: Vec<String>,
    add_to_metadata: bool,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "detailsList", default)]
    details_list: Vec<String>,
    #[serde(rename = "addToMetadata", default = "default_true")]
    add_to_metadata: bool,
}

fn default_true() -> bool { true }

impl GetCustomerDetailsNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("GetCustomerDetailsNode: {}", e)))?;
        Ok(Self { details: cfg.details_list, add_to_metadata: cfg.add_to_metadata })
    }
}

#[async_trait]
impl RuleNode for GetCustomerDetailsNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        // Determine customer ID — from originator if CUSTOMER, else from device
        let customer_id = if msg.originator_type == "CUSTOMER" {
            Some(msg.originator_id)
        } else if msg.originator_type == "DEVICE" {
            ctx.dao.device.find_by_id(msg.originator_id).await?
                .and_then(|d| d.customer_id)
        } else {
            None
        };

        let Some(cid) = customer_id else {
            return Ok(vec![(RelationType::Failure, msg)]);
        };

        let Some(customer) = ctx.dao.customer.find_by_id(cid).await? else {
            return Ok(vec![(RelationType::Failure, msg)]);
        };

        let mut out = msg;
        for detail in &self.details {
            let val = match detail.as_str() {
                "title"   => customer.title.clone(),
                "email"   => customer.email.clone().unwrap_or_default(),
                "phone"   => customer.phone.clone().unwrap_or_default(),
                "city"    => customer.city.clone().unwrap_or_default(),
                "country" => customer.country.clone().unwrap_or_default(),
                "zip"     => customer.zip.clone().unwrap_or_default(),
                _         => continue,
            };
            if self.add_to_metadata {
                out.metadata.insert(format!("customer_{}", detail), val);
            } else {
                if let Ok(mut obj) = serde_json::from_str::<serde_json::Value>(&out.data) {
                    obj[detail.as_str()] = serde_json::Value::String(val);
                    out.data = serde_json::to_string(&obj).unwrap_or(out.data);
                }
            }
        }

        Ok(vec![(RelationType::Success, out)])
    }
}
