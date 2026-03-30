use async_trait::async_trait;
use vl_core::entities::{AttributeKvEntry, AttributeScope, TbMsg};
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Save attributes to attribute_kv.
/// Config: `{ "scope": "SERVER_SCOPE" }` (CLIENT_SCOPE | SERVER_SCOPE | SHARED_SCOPE)
pub struct SaveAttributesNode {
    scope: AttributeScope,
}

impl SaveAttributesNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let scope_str = config["scope"].as_str().unwrap_or("SERVER_SCOPE");
        let scope = parse_scope(scope_str);
        Ok(Self { scope })
    }
}

#[async_trait]
impl RuleNode for SaveAttributesNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let data: serde_json::Value = serde_json::from_str(&msg.data)?;
        let now = chrono::Utc::now().timestamp_millis();

        if let Some(obj) = data.as_object() {
            for (key_name, value) in obj {
                let key_id = ctx.dao.kv.get_or_create_key(key_name).await?;
                let attr = json_value_to_attr(msg.originator_id, self.scope, key_id, value, now);
                ctx.dao.kv.save_attribute(&attr).await?;
            }
        }

        Ok(vec![(RelationType::Success, msg)])
    }
}

fn parse_scope(s: &str) -> AttributeScope {
    match s.to_uppercase().as_str() {
        "CLIENT_SCOPE" => AttributeScope::ClientScope,
        "SHARED_SCOPE" => AttributeScope::SharedScope,
        _              => AttributeScope::ServerScope,
    }
}

fn json_value_to_attr(
    entity_id: uuid::Uuid,
    scope: AttributeScope,
    key_id: i32,
    value: &serde_json::Value,
    ts: i64,
) -> AttributeKvEntry {
    AttributeKvEntry {
        entity_id,
        attribute_type: scope,
        attribute_key: key_id,
        last_update_ts: ts,
        bool_v: value.as_bool(),
        long_v: if value.is_i64() { value.as_i64() } else { None },
        dbl_v:  if value.is_f64() && !value.is_i64() { value.as_f64() } else { None },
        str_v:  value.as_str().map(String::from),
        json_v: if value.is_object() || value.is_array() { Some(value.clone()) } else { None },
        version: 0,
    }
}
