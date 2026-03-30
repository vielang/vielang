use std::collections::HashMap;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, warn};
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// AI provider enum matching ThingsBoard TbAiProvider.java
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum AiProvider {
    #[serde(rename = "OPEN_AI", alias = "openai")]
    OpenAI,
    #[serde(rename = "ANTHROPIC", alias = "anthropic")]
    Anthropic,
    #[serde(rename = "AZURE_OPEN_AI", alias = "azure")]
    AzureOpenAI,
}

impl Default for AiProvider {
    fn default() -> Self { Self::OpenAI }
}

/// Response format — text or structured JSON
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum ResponseFormat {
    #[serde(rename = "TEXT", alias = "text")]
    Text,
    #[serde(rename = "JSON", alias = "json")]
    Json,
}

impl Default for ResponseFormat {
    fn default() -> Self { Self::Text }
}

#[derive(Deserialize)]
struct AiNodeConfig {
    #[serde(default)]
    provider: AiProvider,
    /// Optional override for the provider's API endpoint
    #[serde(rename = "apiEndpoint")]
    api_endpoint: Option<String>,
    /// API key for the provider
    #[serde(rename = "apiKey")]
    api_key: String,
    /// Model name: "gpt-4o", "claude-3-5-sonnet-20241022", etc.
    #[serde(default = "default_model")]
    model: String,
    /// System prompt — supports ${key} substitution from metadata
    #[serde(rename = "systemPrompt", default)]
    system_prompt: String,
    /// User prompt — supports ${key} and ${data} substitution
    #[serde(rename = "userPrompt")]
    user_prompt: String,
    /// Whether to expect JSON or plain text from the AI
    #[serde(rename = "responseFormat", default)]
    response_format: ResponseFormat,
    /// Request timeout in seconds (1–600)
    #[serde(rename = "timeoutSeconds", default = "default_timeout")]
    timeout_seconds: u64,
    #[serde(default)]
    temperature: Option<f64>,
    #[serde(rename = "maxTokens", default)]
    max_tokens: Option<u32>,
}

fn default_model() -> String { "gpt-4o".to_string() }
fn default_timeout() -> u64 { 30 }

/// Rule node that calls an external AI model API and injects the response
/// into the outgoing message body.
///
/// Mirrors ThingsBoard `TbAiNode.java`. Supports:
/// - OpenAI Chat Completions API
/// - Anthropic Messages API
/// - Azure OpenAI (same format as OpenAI, different endpoint + auth header)
///
/// Config JSON example:
/// ```json
/// {
///   "provider": "OPEN_AI",
///   "apiKey": "sk-...",
///   "model": "gpt-4o",
///   "systemPrompt": "You are an IoT analyst.",
///   "userPrompt": "Telemetry: ${data}. Device: ${deviceName}",
///   "responseFormat": "JSON",
///   "timeoutSeconds": 30,
///   "temperature": 0.3,
///   "maxTokens": 512
/// }
/// ```
pub struct AiNode {
    provider:        AiProvider,
    api_endpoint:    String,
    api_key:         String,
    model:           String,
    system_prompt:   String,
    user_prompt:     String,
    response_format: ResponseFormat,
    timeout_seconds: u64,
    temperature:     Option<f64>,
    max_tokens:      Option<u32>,
}

impl AiNode {
    pub fn new(config: &Value) -> Result<Self, RuleEngineError> {
        let cfg: AiNodeConfig = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("AiNode config: {}", e)))?;

        let api_endpoint = cfg.api_endpoint.unwrap_or_else(|| match cfg.provider {
            AiProvider::Anthropic => {
                "https://api.anthropic.com/v1/messages".to_string()
            }
            AiProvider::AzureOpenAI => {
                "https://YOUR_RESOURCE.openai.azure.com/openai/deployments/YOUR_DEPLOYMENT\
                 /chat/completions?api-version=2024-02-01"
                    .to_string()
            }
            AiProvider::OpenAI => {
                "https://api.openai.com/v1/chat/completions".to_string()
            }
        });

        Ok(Self {
            provider: cfg.provider,
            api_endpoint,
            api_key: cfg.api_key,
            model: cfg.model,
            system_prompt: cfg.system_prompt,
            user_prompt: cfg.user_prompt,
            response_format: cfg.response_format,
            timeout_seconds: cfg.timeout_seconds,
            temperature: cfg.temperature,
            max_tokens: cfg.max_tokens,
        })
    }

    /// Call OpenAI (or Azure OpenAI) Chat Completions API.
    async fn call_openai(&self, system: &str, user: &str) -> Result<String, String> {
        let mut messages = Vec::new();
        if !system.is_empty() {
            messages.push(json!({"role": "system", "content": system}));
        }
        messages.push(json!({"role": "user", "content": user}));

        let mut body = json!({
            "model": self.model,
            "messages": messages,
        });
        if let Some(t) = self.temperature {
            body["temperature"] = json!(t);
        }
        if let Some(m) = self.max_tokens {
            body["max_tokens"] = json!(m);
        }
        if self.response_format == ResponseFormat::Json {
            body["response_format"] = json!({"type": "json_object"});
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .build()
            .map_err(|e| e.to_string())?;

        // Azure uses "api-key" header; OpenAI uses "Authorization: Bearer"
        let auth_header = if self.provider == AiProvider::AzureOpenAI {
            ("api-key".to_string(), self.api_key.clone())
        } else {
            ("Authorization".to_string(), format!("Bearer {}", self.api_key))
        };

        let resp = client
            .post(&self.api_endpoint)
            .header(auth_header.0, auth_header.1)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("OpenAI API error {}: {}", status, text));
        }

        let json: Value = resp.json().await.map_err(|e| e.to_string())?;
        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| format!("Unexpected OpenAI response structure: {}", json))
    }

    /// Call Anthropic Messages API.
    async fn call_anthropic(&self, system: &str, user: &str) -> Result<String, String> {
        let mut body = json!({
            "model": self.model,
            "messages": [{"role": "user", "content": user}],
            "max_tokens": self.max_tokens.unwrap_or(1024),
        });
        if !system.is_empty() {
            body["system"] = json!(system);
        }
        if let Some(t) = self.temperature {
            body["temperature"] = json!(t);
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .build()
            .map_err(|e| e.to_string())?;

        let resp = client
            .post(&self.api_endpoint)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Anthropic API error {}: {}", status, text));
        }

        let json: Value = resp.json().await.map_err(|e| e.to_string())?;
        json["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| format!("Unexpected Anthropic response structure: {}", json))
    }
}

/// Substitute ${key} placeholders from metadata, plus ${data} from message body.
fn substitute_placeholders(
    template: &str,
    data: &str,
    metadata: &HashMap<String, String>,
) -> String {
    let mut result = template.to_string();
    result = result.replace("${data}", data);
    for (k, v) in metadata {
        result = result.replace(&format!("${{{}}}", k), v);
    }
    result
}

#[async_trait]
impl RuleNode for AiNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let system = substitute_placeholders(&self.system_prompt, &msg.data, &msg.metadata);
        let user   = substitute_placeholders(&self.user_prompt,   &msg.data, &msg.metadata);

        debug!(
            provider = ?self.provider,
            model    = %self.model,
            "Calling AI provider"
        );

        let ai_result = match self.provider {
            AiProvider::OpenAI | AiProvider::AzureOpenAI => {
                self.call_openai(&system, &user).await
            }
            AiProvider::Anthropic => {
                self.call_anthropic(&system, &user).await
            }
        };

        match ai_result {
            Ok(response) => {
                let mut out = msg;
                // Wrap plain-text response in {"response":"..."} for consistent JSON output.
                // If format is JSON and response is already valid JSON, use it directly.
                out.data = match self.response_format {
                    ResponseFormat::Json => {
                        if serde_json::from_str::<Value>(&response).is_ok() {
                            response
                        } else {
                            json!({"response": response}).to_string()
                        }
                    }
                    ResponseFormat::Text => json!({"response": response}).to_string(),
                };
                out.metadata.insert("aiProvider".into(), format!("{:?}", self.provider));
                out.metadata.insert("aiModel".into(), self.model.clone());
                Ok(vec![(RelationType::Success, out)])
            }
            Err(e) => {
                warn!(error = %e, "AI node request failed");
                let mut out = msg;
                out.metadata.insert("error".into(), e);
                Ok(vec![(RelationType::Failure, out)])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_openai_config() {
        let config = json!({
            "provider": "OPEN_AI",
            "apiKey": "sk-test",
            "model": "gpt-4o",
            "systemPrompt": "You are an IoT analyst.",
            "userPrompt": "Analyze: ${data}",
            "responseFormat": "JSON",
            "timeoutSeconds": 30,
        });
        let node = AiNode::new(&config).unwrap();
        assert_eq!(node.provider, AiProvider::OpenAI);
        assert_eq!(node.model, "gpt-4o");
        assert_eq!(node.response_format, ResponseFormat::Json);
        assert!(node.api_endpoint.contains("openai.com"));
    }

    #[test]
    fn test_anthropic_config() {
        let config = json!({
            "provider": "ANTHROPIC",
            "apiKey": "sk-ant-test",
            "model": "claude-3-5-sonnet-20241022",
            "systemPrompt": "Analyze IoT data.",
            "userPrompt": "Device: ${deviceName}. Data: ${data}",
        });
        let node = AiNode::new(&config).unwrap();
        assert_eq!(node.provider, AiProvider::Anthropic);
        assert!(node.api_endpoint.contains("anthropic.com"));
        assert_eq!(node.response_format, ResponseFormat::Text);
    }

    #[test]
    fn test_azure_custom_endpoint() {
        let config = json!({
            "provider": "AZURE_OPEN_AI",
            "apiKey": "azure-key",
            "apiEndpoint": "https://myresource.openai.azure.com/openai/deployments/gpt-4o/chat/completions?api-version=2024-02-01",
            "model": "gpt-4o",
            "userPrompt": "Summarize: ${data}",
        });
        let node = AiNode::new(&config).unwrap();
        assert_eq!(node.provider, AiProvider::AzureOpenAI);
        assert!(node.api_endpoint.contains("myresource.openai.azure.com"));
    }

    #[test]
    fn test_substitute_placeholders() {
        let mut meta = HashMap::new();
        meta.insert("deviceName".into(), "sensor-01".into());
        meta.insert("temperature".into(), "25.3".into());
        let result = substitute_placeholders(
            "Device ${deviceName} (temp=${temperature}) sent: ${data}",
            r#"{"ts":1700000000}"#,
            &meta,
        );
        assert_eq!(
            result,
            r#"Device sensor-01 (temp=25.3) sent: {"ts":1700000000}"#
        );
    }

    #[test]
    fn test_missing_config_field() {
        let config = json!({"apiKey": "k"}); // userPrompt missing
        let result = AiNode::new(&config);
        assert!(result.is_err());
    }
}
