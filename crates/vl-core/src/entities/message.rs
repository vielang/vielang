use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Message type constants — khớp Java: TbMsgType
pub mod msg_type {
    pub const POST_TELEMETRY_REQUEST:  &str = "POST_TELEMETRY_REQUEST";
    pub const POST_ATTRIBUTES_REQUEST: &str = "POST_ATTRIBUTES_REQUEST";
    pub const CONNECT_EVENT:           &str = "CONNECT_EVENT";
    pub const DISCONNECT_EVENT:        &str = "DISCONNECT_EVENT";
    pub const ENTITY_CREATED:          &str = "ENTITY_CREATED";
    pub const ENTITY_UPDATED:          &str = "ENTITY_UPDATED";
    pub const ENTITY_DELETED:          &str = "ENTITY_DELETED";
    pub const ALARM:                   &str = "ALARM";
    pub const ALARM_ACK:               &str = "ALARM_ACK";
    pub const ALARM_CLEAR:             &str = "ALARM_CLEAR";
    pub const RPC_CALL_FROM_SERVER:    &str = "RPC_CALL_FROM_SERVER_SIDE_REQUEST";
    pub const ATTRIBUTE_UPDATED:       &str = "ATTRIBUTES_UPDATED";
}

/// Đơn vị thông điệp của Rule Engine — khớp Java: TbMsg
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TbMsg {
    pub id:              Uuid,
    pub ts:              i64,
    pub msg_type:        String,
    pub originator_id:   Uuid,
    pub originator_type: String,
    pub customer_id:     Option<Uuid>,
    pub metadata:        HashMap<String, String>,
    /// JSON body of the message
    pub data:            String,
    pub rule_chain_id:   Option<Uuid>,
    pub rule_node_id:    Option<Uuid>,
    /// Tenant that owns the originator — set by transport layer when known.
    /// Used for multi-tenant rule chain routing.
    #[serde(default)]
    pub tenant_id:       Option<Uuid>,
}

impl TbMsg {
    pub fn new(
        msg_type: impl Into<String>,
        originator_id: Uuid,
        originator_type: impl Into<String>,
        data: impl Into<String>,
    ) -> Self {
        Self {
            id:              Uuid::new_v4(),
            ts:              chrono::Utc::now().timestamp_millis(),
            msg_type:        msg_type.into(),
            originator_id,
            originator_type: originator_type.into(),
            customer_id:     None,
            metadata:        HashMap::new(),
            data:            data.into(),
            rule_chain_id:   None,
            rule_node_id:    None,
            tenant_id:       None,
        }
    }

    /// Builder: attach tenant context for multi-tenant rule chain routing.
    pub fn with_tenant(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }
}
