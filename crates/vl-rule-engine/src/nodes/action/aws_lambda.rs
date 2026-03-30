use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Invoke an AWS Lambda function via Function URL or API Gateway.
/// Java: TbAwsLambdaNode
/// Config:
/// ```json
/// {
///   "functionUrl": "https://xxx.lambda-url.us-east-1.on.aws/",
///   "functionName": "myFunction",
///   "invocationType": "Event"
/// }
/// ```
pub struct AwsLambdaNode {
    function_url: String,
    invocation_type: String,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "functionUrl", default)]
    function_url: String,
    #[serde(rename = "invocationType", default = "default_invocation")]
    invocation_type: String,
}

fn default_invocation() -> String { "Event".into() }

impl AwsLambdaNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("AwsLambdaNode: {}", e)))?;
        Ok(Self {
            function_url: cfg.function_url,
            invocation_type: cfg.invocation_type,
        })
    }
}

#[async_trait]
impl RuleNode for AwsLambdaNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        if self.function_url.is_empty() {
            tracing::info!("AwsLambdaNode (log-only): body={}", msg.data);
            return Ok(vec![(RelationType::Success, msg)]);
        }

        let client = reqwest::Client::new();
        let res = client.post(&self.function_url)
            .header("X-Amz-Invocation-Type", self.invocation_type.as_str())
            .header("Content-Type", "application/json")
            .body(msg.data.clone())
            .send()
            .await
            .map_err(|e| RuleEngineError::Processing(format!("Lambda invoke: {}", e)))?;

        if res.status().is_success() {
            Ok(vec![(RelationType::Success, msg)])
        } else {
            Ok(vec![(RelationType::Failure, msg)])
        }
    }
}
