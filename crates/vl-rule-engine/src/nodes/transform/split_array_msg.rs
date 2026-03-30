use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Split a JSON array field into individual messages, one per element.
/// Java: TbSplitArrayMsgNode
/// Config:
/// ```json
/// { "arrayKey": "data" }
/// ```
pub struct SplitArrayMsgNode {
    array_key: String,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "arrayKey", default = "default_key")]
    array_key: String,
}

fn default_key() -> String { "data".into() }

impl SplitArrayMsgNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("SplitArrayMsgNode: {}", e)))?;
        Ok(Self { array_key: cfg.array_key })
    }
}

#[async_trait]
impl RuleNode for SplitArrayMsgNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let data: serde_json::Value = serde_json::from_str(&msg.data)
            .unwrap_or(serde_json::json!({}));

        let array = if let Some(arr) = data.get(&self.array_key).and_then(|v| v.as_array()) {
            arr.clone()
        } else if data.is_array() {
            data.as_array().cloned().unwrap_or_default()
        } else {
            return Ok(vec![(RelationType::Success, msg)]);
        };

        if array.is_empty() {
            return Ok(vec![(RelationType::Success, msg)]);
        }

        let results = array.into_iter().map(|item| {
            let item_data = serde_json::to_string(&item).unwrap_or_else(|_| "{}".into());
            let split_msg = TbMsg {
                id:              uuid::Uuid::new_v4(),
                ts:              msg.ts,
                msg_type:        msg.msg_type.clone(),
                originator_id:   msg.originator_id,
                originator_type: msg.originator_type.clone(),
                customer_id:     msg.customer_id,
                data:            item_data,
                metadata:        msg.metadata.clone(),
                rule_chain_id:   msg.rule_chain_id,
                rule_node_id:    msg.rule_node_id,
                tenant_id:       msg.tenant_id,
            };
            (RelationType::Success, split_msg)
        }).collect();

        Ok(results)
    }
}
