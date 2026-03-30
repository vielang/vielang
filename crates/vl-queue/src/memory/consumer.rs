use async_trait::async_trait;
use tokio::sync::broadcast;
use tracing::warn;

use crate::{QueueError, QueueMsg, TbConsumer};

/// In-memory consumer — reads from one or more broadcast receivers.
pub struct MemoryConsumer {
    receivers: Vec<broadcast::Receiver<QueueMsg>>,
    group_id:  String,
}

impl MemoryConsumer {
    pub fn new(receivers: Vec<broadcast::Receiver<QueueMsg>>, group_id: String) -> Self {
        Self { receivers, group_id }
    }
}

#[async_trait]
impl TbConsumer for MemoryConsumer {
    async fn poll(&mut self) -> Result<Vec<QueueMsg>, QueueError> {
        let mut msgs = Vec::new();

        for rx in &mut self.receivers {
            loop {
                match rx.try_recv() {
                    Ok(msg) => msgs.push(msg),
                    Err(broadcast::error::TryRecvError::Empty) => break,
                    Err(broadcast::error::TryRecvError::Lagged(n)) => {
                        warn!(group = %self.group_id, dropped = n, "Consumer lagged — messages dropped");
                        // Continue reading from the new position
                    }
                    Err(broadcast::error::TryRecvError::Closed) => {
                        return Err(QueueError::Closed);
                    }
                }
            }
        }

        if !msgs.is_empty() {
            metrics::counter!("vielang_queue_consume_total", "backend" => "memory", "group" => self.group_id.clone())
                .increment(msgs.len() as u64);
        }
        Ok(msgs)
    }

    async fn commit(&mut self) -> Result<(), QueueError> {
        // No-op: in-memory queue has no offset tracking
        Ok(())
    }

    fn group_id(&self) -> &str {
        &self.group_id
    }
}
