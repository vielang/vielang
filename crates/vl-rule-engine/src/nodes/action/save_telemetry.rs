use async_trait::async_trait;
use vl_core::entities::{TbMsg, TsKvEntry};
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Save telemetry to ts_kv and ts_kv_latest.
/// Config: `{}` (no extra config needed)
pub struct SaveTelemetryNode;

impl SaveTelemetryNode {
    pub fn new(_config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        Ok(Self)
    }
}

#[async_trait]
impl RuleNode for SaveTelemetryNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let data: serde_json::Value = serde_json::from_str(&msg.data)?;

        let entries = extract_kv_entries(msg.originator_id, msg.ts, &data);
        for (key_name, entry) in entries {
            let key_id = ctx.dao.kv.get_or_create_key(&key_name).await?;
            let entry = TsKvEntry { key: key_id, ..entry };
            ctx.dao.kv.save_ts(&entry).await?;
            ctx.dao.kv.save_latest(&entry).await?;
        }

        Ok(vec![(RelationType::Success, msg)])
    }
}

/// Extract key→TsKvEntry pairs from a JSON object (flat key-value).
fn extract_kv_entries(
    entity_id: uuid::Uuid,
    ts: i64,
    data: &serde_json::Value,
) -> Vec<(String, TsKvEntry)> {
    let mut result = Vec::new();
    if let Some(obj) = data.as_object() {
        for (key, value) in obj {
            let entry = value_to_entry(entity_id, ts, value);
            result.push((key.clone(), entry));
        }
    }
    result
}

fn value_to_entry(entity_id: uuid::Uuid, ts: i64, value: &serde_json::Value) -> TsKvEntry {
    TsKvEntry {
        entity_id,
        key: 0, // placeholder — will be replaced with real key_id
        ts,
        bool_v:  value.as_bool(),
        long_v:  if value.is_i64() { value.as_i64() } else { None },
        dbl_v:   if value.is_f64() && !value.is_i64() { value.as_f64() } else { None },
        str_v:   value.as_str().map(String::from),
        json_v:  if value.is_object() || value.is_array() { Some(value.clone()) } else { None },
        version: 0,
    }
}
