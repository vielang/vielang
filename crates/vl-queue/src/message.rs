use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use vl_core::entities::TbMsg;
use crate::error::QueueError;

/// A message envelope transported through the queue.
/// `value` is a JSON-serialized `TbMsg`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMsg {
    /// Destination topic
    pub topic: String,
    /// Partition key — typically tenant_id string for Kafka partitioning
    pub key: String,
    /// Payload — JSON-serialized TbMsg bytes
    pub value: Vec<u8>,
    /// Optional metadata headers
    pub headers: HashMap<String, String>,
    /// Backend-specific acknowledgement handle (not serialized over wire):
    /// - RabbitMQ: delivery_tag as decimal string
    /// - Redis Streams: entry ID (e.g. "1699000000000-0")
    /// - SQS: ReceiptHandle
    #[serde(skip)]
    pub ack_id: Option<String>,
}

impl QueueMsg {
    /// Wrap a TbMsg for transport on the given topic.
    pub fn from_tb_msg(topic: impl Into<String>, msg: &TbMsg) -> Result<Self, QueueError> {
        let value = serde_json::to_vec(msg)?;
        let key = msg.originator_id.to_string();
        Ok(Self {
            topic:   topic.into(),
            key,
            value,
            headers: HashMap::new(),
            ack_id:  None,
        })
    }

    /// Deserialize the payload back into a TbMsg.
    pub fn to_tb_msg(&self) -> Result<TbMsg, QueueError> {
        Ok(serde_json::from_slice(&self.value)?)
    }

    /// Convenience: create with a plain byte payload (not TbMsg).
    pub fn raw(topic: impl Into<String>, key: impl Into<String>, value: Vec<u8>) -> Self {
        Self {
            topic:   topic.into(),
            key:     key.into(),
            value,
            headers: HashMap::new(),
            ack_id:  None,
        }
    }
}
