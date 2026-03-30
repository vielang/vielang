use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Fetch entity fields (name, label, type) of the originator and add to metadata.
/// Config JSON:
/// ```json
/// {
///   "fieldsMapping": { "name": "deviceName", "type": "deviceType" }
/// }
/// ```
pub struct OriginatorFieldsNode {
    fields_mapping: Vec<(String, String)>,  // source_field → target_key
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "fieldsMapping", default)]
    fields_mapping: std::collections::HashMap<String, String>,
}

impl OriginatorFieldsNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("OriginatorFieldsNode: {}", e)))?;
        Ok(Self {
            fields_mapping: cfg.fields_mapping.into_iter().collect(),
        })
    }
}

#[async_trait]
impl RuleNode for OriginatorFieldsNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        // Resolve originator fields based on originator_type
        let fields = match msg.originator_type.to_uppercase().as_str() {
            "DEVICE" => {
                ctx.dao.device.find_by_id(msg.originator_id).await?
                    .map(|d| EntityFields {
                        name:         d.name,
                        label:        d.label.unwrap_or_default(),
                        entity_type:  "DEVICE".into(),
                        created_time: d.created_time,
                    })
            }
            "ASSET" => {
                ctx.dao.asset.find_by_id(msg.originator_id).await?
                    .map(|a| EntityFields {
                        name:         a.name,
                        label:        a.label.unwrap_or_default(),
                        entity_type:  "ASSET".into(),
                        created_time: a.created_time,
                    })
            }
            _ => None,
        };

        let mut out = msg;
        if let Some(fields) = fields {
            for (source_field, target_key) in &self.fields_mapping {
                let val = match source_field.as_str() {
                    "name"        => fields.name.clone(),
                    "label"       => fields.label.clone(),
                    "type"        => fields.entity_type.clone(),
                    "createdTime" => fields.created_time.to_string(),
                    _             => continue,
                };
                out.metadata.insert(target_key.clone(), val);
            }
        }

        Ok(vec![(RelationType::Success, out)])
    }
}

struct EntityFields {
    name:         String,
    label:        String,
    entity_type:  String,
    created_time: i64,
}
