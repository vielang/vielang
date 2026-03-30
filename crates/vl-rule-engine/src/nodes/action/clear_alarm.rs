use async_trait::async_trait;
use vl_core::entities::{TbMsg, msg_type};
use vl_dao::DaoError;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Clear the most recent active alarm for the originator entity.
/// Config: `{ "alarmType": "HighTemperature" }`
///
/// NOTE: This requires a find_active_by_originator method in AlarmDao.
/// For Phase 5, we use a best-effort approach via the metadata from the message.
pub struct ClearAlarmNode {
    alarm_type: String,
}

impl ClearAlarmNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let alarm_type = config["alarmType"]
            .as_str()
            .ok_or_else(|| RuleEngineError::Config("alarmType required".into()))?
            .to_string();
        Ok(Self { alarm_type })
    }
}

#[async_trait]
impl RuleNode for ClearAlarmNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let ts = chrono::Utc::now().timestamp_millis();

        // Try to get alarm_id from message metadata
        if let Some(alarm_id_str) = msg.metadata.get("alarmId") {
            if let Ok(alarm_id) = alarm_id_str.parse::<uuid::Uuid>() {
                match ctx.dao.alarm.clear(alarm_id, ts).await {
                    Ok(_) => {}
                    Err(DaoError::NotFound) => {} // already cleared
                    Err(e) => return Err(RuleEngineError::Dao(e)),
                }
            }
        }

        let mut out = msg;
        out.msg_type = msg_type::ALARM_CLEAR.to_string();
        out.metadata.insert("alarmType".into(), self.alarm_type.clone());
        out.metadata.insert("alarmStatus".into(), "CLEARED_UNACK".into());

        Ok(vec![(RelationType::Success, out)])
    }
}
