use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Change the originator of the message.
/// Config JSON:
/// ```json
/// {
///   "originatorSource": "CUSTOMER",
///   "entityType": "CUSTOMER"
/// }
/// ```
/// originatorSource: "CUSTOMER" | "TENANT" | "RELATED"
pub struct ChangeOriginatorNode {
    originator_source: String,
    relation_type:     Option<String>,
    entity_type:       Option<String>,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "originatorSource")]
    originator_source: String,
    #[serde(rename = "relationType")]
    relation_type: Option<String>,
    #[serde(rename = "entityType")]
    entity_type: Option<String>,
}

impl ChangeOriginatorNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("ChangeOriginatorNode: {}", e)))?;
        Ok(Self {
            originator_source: cfg.originator_source,
            relation_type:     cfg.relation_type,
            entity_type:       cfg.entity_type,
        })
    }
}

#[async_trait]
impl RuleNode for ChangeOriginatorNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let mut out = msg;

        match self.originator_source.to_uppercase().as_str() {
            "CUSTOMER" => {
                let customer_id = if out.originator_type.to_uppercase() == "DEVICE" {
                    ctx.dao.device.find_by_id(out.originator_id).await?
                        .and_then(|d| d.customer_id)
                } else if out.originator_type.to_uppercase() == "ASSET" {
                    ctx.dao.asset.find_by_id(out.originator_id).await?
                        .and_then(|a| a.customer_id)
                } else {
                    None
                };

                if let Some(cid) = customer_id {
                    out.originator_id   = cid;
                    out.originator_type = "CUSTOMER".into();
                    return Ok(vec![(RelationType::Success, out)]);
                }
                Ok(vec![(RelationType::Failure, out)])
            }
            "TENANT" => {
                out.originator_id   = ctx.tenant_id;
                out.originator_type = "TENANT".into();
                Ok(vec![(RelationType::Success, out)])
            }
            "RELATED" => {
                let relations = ctx.dao.relation
                    .find_by_from(out.originator_id, &out.originator_type).await?;

                let related = relations.into_iter().find(|r| {
                    let rtype_match = self.relation_type.as_ref()
                        .map(|rt| rt == &r.relation_type)
                        .unwrap_or(true);
                    let etype_match = self.entity_type.as_ref()
                        .map(|et| et.to_uppercase() == format!("{:?}", r.to_type).to_uppercase())
                        .unwrap_or(true);
                    rtype_match && etype_match
                });

                if let Some(rel) = related {
                    out.originator_id   = rel.to_id;
                    out.originator_type = format!("{:?}", rel.to_type).to_uppercase();
                    Ok(vec![(RelationType::Success, out)])
                } else {
                    Ok(vec![(RelationType::Failure, out)])
                }
            }
            _ => Ok(vec![(RelationType::Success, out)]),
        }
    }
}
