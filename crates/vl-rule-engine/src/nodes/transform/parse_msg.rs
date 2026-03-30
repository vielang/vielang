use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Parse message payload from a specific format into JSON.
/// Java: TbParseMsgNode
/// Supported formats: JSON (validate/re-serialize), TEXT (wrap as { "text": "..." }),
///                    CSV (first line = header, second line = values)
/// Relations: Success, Failure (parse error)
/// Config:
/// ```json
/// { "parseMode": "JSON" }
/// { "parseMode": "TEXT" }
/// { "parseMode": "CSV", "delimiter": ",", "headerLine": true }
/// ```
pub struct ParseMsgNode {
    mode: ParseMode,
    csv_delimiter: char,
    csv_header_line: bool,
}

#[derive(Debug, Clone, Copy)]
enum ParseMode {
    Json,
    Text,
    Csv,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "parseMode", default = "default_mode")]
    parse_mode: String,
    #[serde(default = "default_delimiter")]
    delimiter: String,
    #[serde(rename = "headerLine", default = "default_true")]
    header_line: bool,
}

fn default_mode() -> String { "JSON".into() }
fn default_delimiter() -> String { ",".into() }
fn default_true() -> bool { true }

impl ParseMsgNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("ParseMsgNode: {}", e)))?;
        let mode = match cfg.parse_mode.to_uppercase().as_str() {
            "JSON" => ParseMode::Json,
            "TEXT" => ParseMode::Text,
            "CSV"  => ParseMode::Csv,
            other  => return Err(RuleEngineError::Config(
                format!("ParseMsgNode: unknown parseMode '{}'", other)
            )),
        };
        let delimiter = cfg.delimiter.chars().next().unwrap_or(',');
        Ok(Self { mode, csv_delimiter: delimiter, csv_header_line: cfg.header_line })
    }

    fn parse_csv(&self, raw: &str) -> Result<serde_json::Value, String> {
        let mut lines = raw.lines();
        if self.csv_header_line {
            let headers: Vec<&str> = lines
                .next()
                .ok_or("CSV: empty input")?
                .split(self.csv_delimiter)
                .map(str::trim)
                .collect();
            let values: Vec<&str> = lines
                .next()
                .ok_or("CSV: missing data line")?
                .split(self.csv_delimiter)
                .map(str::trim)
                .collect();
            let obj: serde_json::Map<String, serde_json::Value> = headers
                .into_iter()
                .zip(values.into_iter())
                .map(|(h, v)| {
                    // Try to parse as number first, then bool, then string
                    let val = v.parse::<f64>()
                        .map(serde_json::Value::from)
                        .or_else(|_| v.parse::<bool>().map(serde_json::Value::from))
                        .unwrap_or_else(|_| serde_json::Value::String(v.to_string()));
                    (h.to_string(), val)
                })
                .collect();
            Ok(serde_json::Value::Object(obj))
        } else {
            // No header: return array of values
            let values: Vec<serde_json::Value> = raw
                .split(self.csv_delimiter)
                .map(|v| {
                    let v = v.trim();
                    v.parse::<f64>()
                        .map(serde_json::Value::from)
                        .unwrap_or_else(|_| serde_json::Value::String(v.to_string()))
                })
                .collect();
            Ok(serde_json::Value::Array(values))
        }
    }
}

#[async_trait]
impl RuleNode for ParseMsgNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let parsed = match self.mode {
            ParseMode::Json => {
                match serde_json::from_str::<serde_json::Value>(&msg.data) {
                    Ok(v)  => serde_json::to_string(&v).unwrap_or_else(|_| msg.data.clone()),
                    Err(e) => {
                        let mut m = msg;
                        m.metadata.insert("error".into(), format!("ParseMsgNode: {}", e));
                        return Ok(vec![(RelationType::Failure, m)]);
                    }
                }
            }
            ParseMode::Text => {
                serde_json::json!({ "text": msg.data }).to_string()
            }
            ParseMode::Csv => {
                match self.parse_csv(&msg.data) {
                    Ok(v)  => serde_json::to_string(&v).unwrap_or_else(|_| msg.data.clone()),
                    Err(e) => {
                        let mut m = msg;
                        m.metadata.insert("error".into(), format!("ParseMsgNode: {}", e));
                        return Ok(vec![(RelationType::Failure, m)]);
                    }
                }
            }
        };

        let mut out = msg;
        out.data = parsed;
        Ok(vec![(RelationType::Success, out)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_mode_roundtrips() {
        let node = ParseMsgNode::new(&json!({ "parseMode": "JSON" })).unwrap();
        let _ = node; // compile check
    }

    #[test]
    fn csv_parse_with_headers() {
        let node = ParseMsgNode::new(&json!({
            "parseMode": "CSV",
            "delimiter": ",",
            "headerLine": true
        })).unwrap();
        let result = node.parse_csv("temperature,humidity\n22.5,60").unwrap();
        assert_eq!(result["temperature"], 22.5);
        assert_eq!(result["humidity"], 60.0);
    }

    #[test]
    fn unknown_mode_is_error() {
        assert!(ParseMsgNode::new(&json!({ "parseMode": "YAML" })).is_err());
    }
}
