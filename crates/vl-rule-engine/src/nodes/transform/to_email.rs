use std::collections::HashMap;
use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Transform a TbMsg into an email-ready format by applying ${} template substitution.
/// Stores result in message metadata for use by SendEmailNode downstream.
/// Config JSON:
/// ```json
/// {
///   "fromTemplate": "noreply@example.com",
///   "toTemplate": "${customerEmail}",
///   "subjectTemplate": "Alert: ${alarmType}",
///   "bodyTemplate": "Device ${deviceName} triggered alarm at ${ts}",
///   "isHtml": false
/// }
/// ```
pub struct ToEmailNode {
    from_template:    String,
    to_template:      String,
    cc_template:      Option<String>,
    subject_template: String,
    body_template:    String,
    is_html:          bool,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "fromTemplate", default)]
    from_template: String,
    #[serde(rename = "toTemplate", default)]
    to_template: String,
    #[serde(rename = "ccTemplate")]
    cc_template: Option<String>,
    #[serde(rename = "subjectTemplate", default)]
    subject_template: String,
    #[serde(rename = "bodyTemplate", default)]
    body_template: String,
    #[serde(rename = "isHtml", default)]
    is_html: bool,
}

impl ToEmailNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("ToEmailNode: {}", e)))?;
        Ok(Self {
            from_template:    cfg.from_template,
            to_template:      cfg.to_template,
            cc_template:      cfg.cc_template,
            subject_template: cfg.subject_template,
            body_template:    cfg.body_template,
            is_html:          cfg.is_html,
        })
    }
}

#[async_trait]
impl RuleNode for ToEmailNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        // Build full variable map: metadata + data fields
        let mut vars = msg.metadata.clone();
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&msg.data) {
            if let Some(obj) = data.as_object() {
                for (k, v) in obj {
                    vars.entry(k.clone()).or_insert_with(|| v.to_string().trim_matches('"').to_string());
                }
            }
        }
        vars.insert("ts".into(), msg.ts.to_string());

        let mut out = msg;
        out.metadata.insert("email_from".into(), substitute(&self.from_template, &vars));
        out.metadata.insert("email_to".into(), substitute(&self.to_template, &vars));
        if let Some(ref cc) = self.cc_template {
            out.metadata.insert("email_cc".into(), substitute(cc, &vars));
        }
        out.metadata.insert("email_subject".into(), substitute(&self.subject_template, &vars));
        out.metadata.insert("email_body".into(), substitute(&self.body_template, &vars));
        out.metadata.insert("email_is_html".into(), self.is_html.to_string());

        Ok(vec![(RelationType::Success, out)])
    }
}

fn substitute(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (k, v) in vars {
        result = result.replace(&format!("${{{}}}", k), v);
    }
    result
}
