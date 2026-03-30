use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Assign originator entity (DEVICE or ASSET) to a customer.
/// Java: TbAssignToCustomerNode
/// Config:
/// ```json
/// { "customerIdKey": "customerId" }
/// ```
pub struct AssignToCustomerNode {
    customer_id_key: String,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "customerIdKey", default = "default_cid_key")]
    customer_id_key: String,
}

fn default_cid_key() -> String { "customerId".into() }

impl AssignToCustomerNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("AssignToCustomerNode: {}", e)))?;
        Ok(Self { customer_id_key: cfg.customer_id_key })
    }
}

#[async_trait]
impl RuleNode for AssignToCustomerNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let cid_str = msg.metadata.get(&self.customer_id_key)
            .cloned()
            .unwrap_or_default();
        let customer_id = match Uuid::parse_str(&cid_str) {
            Ok(id) => id,
            Err(_) => return Ok(vec![(RelationType::Failure, msg)]),
        };

        match msg.originator_type.as_str() {
            "DEVICE" => {
                if let Some(mut device) = ctx.dao.device.find_by_id(msg.originator_id).await? {
                    device.customer_id = Some(customer_id);
                    ctx.dao.device.save(&device).await?;
                    Ok(vec![(RelationType::Success, msg)])
                } else {
                    Ok(vec![(RelationType::Failure, msg)])
                }
            }
            "ASSET" => {
                if let Some(mut asset) = ctx.dao.asset.find_by_id(msg.originator_id).await? {
                    asset.customer_id = Some(customer_id);
                    ctx.dao.asset.save(&asset).await?;
                    Ok(vec![(RelationType::Success, msg)])
                } else {
                    Ok(vec![(RelationType::Failure, msg)])
                }
            }
            _ => Ok(vec![(RelationType::Failure, msg)]),
        }
    }
}
