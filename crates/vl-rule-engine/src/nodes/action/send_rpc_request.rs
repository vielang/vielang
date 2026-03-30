use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Enrich msg metadata with RPC request data; downstream transport sends the actual RPC.
/// Java: TbSendRPCRequestNode
/// Config:
/// ```json
/// { "method": "getTemperature", "params": "{}", "requestIdKey": "requestId", "timeoutInSeconds": 60 }
/// ```
pub struct SendRpcRequestNode {
    method: String,
    params: String,
    request_id_key: String,
    timeout_in_seconds: i64,
}

#[derive(Deserialize)]
struct Config {
    #[serde(default)]
    method: String,
    #[serde(default)]
    params: String,
    #[serde(rename = "requestIdKey", default = "default_req_id_key")]
    request_id_key: String,
    #[serde(rename = "timeoutInSeconds", default = "default_timeout")]
    timeout_in_seconds: i64,
}

fn default_req_id_key() -> String { "requestId".into() }
fn default_timeout() -> i64 { 60 }

impl SendRpcRequestNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("SendRpcRequestNode: {}", e)))?;
        Ok(Self {
            method: cfg.method,
            params: cfg.params,
            request_id_key: cfg.request_id_key,
            timeout_in_seconds: cfg.timeout_in_seconds,
        })
    }
}

#[async_trait]
impl RuleNode for SendRpcRequestNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        if msg.originator_type != "DEVICE" {
            return Ok(vec![(RelationType::Failure, msg)]);
        }

        let request_id = msg.metadata.get(&self.request_id_key)
            .cloned()
            .unwrap_or_else(|| "0".into());

        let mut out = msg;
        out.metadata.insert("rpcMethod".into(), self.method.clone());
        out.metadata.insert("rpcParams".into(), self.params.clone());
        out.metadata.insert("rpcRequestId".into(), request_id);
        out.metadata.insert("rpcTimeoutMs".into(), (self.timeout_in_seconds * 1000).to_string());

        Ok(vec![(RelationType::Success, out)])
    }
}
