use async_trait::async_trait;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Multi-way switch routing messages by their type.
/// Routes to named relations based on msg_type — matches ThingsBoard Java TbMsgTypeSwitchNode.
/// Config: `{}`
pub struct MsgTypeSwitchNode;

impl MsgTypeSwitchNode {
    pub fn new(_config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        Ok(Self)
    }
}

#[async_trait]
impl RuleNode for MsgTypeSwitchNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let relation = match msg.msg_type.as_str() {
            "POST_TELEMETRY_REQUEST"  => "Post Telemetry",
            "POST_ATTRIBUTES_REQUEST" => "Post Attributes",
            "ALARM"                   => "Alarm",
            "ALARM_ACK"               => "Alarm Acknowledged",
            "ALARM_CLEAR"             => "Alarm Cleared",
            "RPC_CALL_FROM_SERVER_SIDE_REQUEST" => "RPC Request",
            "CONNECT_EVENT"           => "Connect",
            "DISCONNECT_EVENT"        => "Disconnect",
            "ENTITY_CREATED"          => "Entity Created",
            "ENTITY_UPDATED"          => "Entity Updated",
            "ENTITY_DELETED"          => "Entity Deleted",
            _                         => "Other",
        };

        Ok(vec![(RelationType::Other(relation.to_string()), msg)])
    }
}
