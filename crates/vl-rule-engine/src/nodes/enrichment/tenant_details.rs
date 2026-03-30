use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Fetch tenant details and enrich message metadata.
/// Java: TbGetTenantDetailsNode
/// Config:
/// ```json
/// { "detailsList": ["title", "email"], "addToMetadata": true }
/// ```
pub struct GetTenantDetailsNode {
    details: Vec<String>,
    add_to_metadata: bool,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "detailsList", default)]
    details_list: Vec<String>,
    #[serde(rename = "addToMetadata", default = "default_true")]
    add_to_metadata: bool,
}

fn default_true() -> bool { true }

impl GetTenantDetailsNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("GetTenantDetailsNode: {}", e)))?;
        Ok(Self { details: cfg.details_list, add_to_metadata: cfg.add_to_metadata })
    }
}

#[async_trait]
impl RuleNode for GetTenantDetailsNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let Some(tenant) = ctx.dao.tenant.find_by_id(ctx.tenant_id).await? else {
            return Ok(vec![(RelationType::Failure, msg)]);
        };

        let mut out = msg;
        for detail in &self.details {
            let val = match detail.as_str() {
                "title" | "name" => tenant.title.clone(),
                "email"          => tenant.email.clone().unwrap_or_default(),
                "phone"          => tenant.phone.clone().unwrap_or_default(),
                "country"        => tenant.country.clone().unwrap_or_default(),
                "city"           => tenant.city.clone().unwrap_or_default(),
                _                => continue,
            };
            if self.add_to_metadata {
                out.metadata.insert(format!("tenant_{}", detail), val);
            } else {
                if let Ok(mut obj) = serde_json::from_str::<serde_json::Value>(&out.data) {
                    obj[detail.as_str()] = serde_json::Value::String(val);
                    out.data = serde_json::to_string(&obj).unwrap_or(out.data);
                }
            }
        }

        Ok(vec![(RelationType::Success, out)])
    }
}
