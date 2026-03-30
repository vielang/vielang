use std::collections::VecDeque;
use std::time::Instant;

use bytes::Bytes;
use dashmap::DashMap;
use uuid::Uuid;

/// A message queued for offline delivery (QoS 1/2) to a disconnected device.
pub struct QueuedMessage {
    pub topic:      String,
    pub payload:    Bytes,
    pub qos:        u8,
    pub packet_id:  u16,
    pub created_at: Instant,
    pub ttl_secs:   u64,
}

/// In-memory store of pending messages per device.
/// Used to implement MQTT persistent sessions (clean_session = false).
pub struct PersistentSessionStore {
    queues:         DashMap<Uuid, VecDeque<QueuedMessage>>,
    max_queue_size: usize,
}

impl PersistentSessionStore {
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            queues: DashMap::new(),
            max_queue_size,
        }
    }

    /// Enqueue a message for offline delivery. Drops the oldest if the queue is full.
    pub fn enqueue(&self, device_id: Uuid, msg: QueuedMessage) {
        let mut queue = self.queues.entry(device_id).or_default();
        if queue.len() >= self.max_queue_size {
            queue.pop_front(); // drop oldest
        }
        queue.push_back(msg);
    }

    /// Remove and return all non-expired pending messages for a device.
    pub fn drain_pending(&self, device_id: Uuid) -> Vec<QueuedMessage> {
        self.queues
            .remove(&device_id)
            .map(|(_, q)| {
                q.into_iter()
                    .filter(|m| m.created_at.elapsed().as_secs() < m.ttl_secs)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Whether the device has a stored session (pending messages or just a session flag).
    pub fn has_session(&self, device_id: Uuid) -> bool {
        self.queues.contains_key(&device_id)
    }

    /// Clear all stored session data for a device.
    pub fn clear_session(&self, device_id: Uuid) {
        self.queues.remove(&device_id);
    }
}
