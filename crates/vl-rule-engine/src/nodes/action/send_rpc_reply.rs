use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Mark a message as an RPC reply to be sent to the originating server.
/// Java: TbSendRPCReplyNode
/// Config:
/// ```json
/// { "requestIdKey": "requestId" }
/// ```
pub struct SendRpcReplyNode {
    request_id_key: String,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "requestIdKey", default = "default_req_id_key")]
    request_id_key: String,
}

fn default_req_id_key() -> String { "requestId".into() }

impl SendRpcReplyNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("SendRpcReplyNode: {}", e)))?;
        Ok(Self { request_id_key: cfg.request_id_key })
    }
}

#[async_trait]
impl RuleNode for SendRpcReplyNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let request_id = msg.metadata.get(&self.request_id_key)
            .cloned()
            .unwrap_or_default();

        let mut out = msg;
        out.metadata.insert("rpcReplyRequestId".into(), request_id);
        out.metadata.insert("rpcReply".into(), "true".into());

        Ok(vec![(RelationType::Success, out)])
    }
}
