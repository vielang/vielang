use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use coap_lite::{CoapOption, MessageType, Packet, ResponseType};
use dashmap::DashMap;
use tokio::net::UdpSocket;
use tokio::sync::{Notify, RwLock};
use tracing::{debug, warn};
use uuid::Uuid;

use crate::error::TransportError;

// ── Types ─────────────────────────────────────────────────────────────────────

/// A single active CoAP Observe subscription (RFC 7641).
#[derive(Debug, Clone)]
pub struct ObserveRelation {
    pub device_id: Uuid,
    pub addr:      SocketAddr,
    /// CoAP token copied from the original GET+Observe request.
    pub token:     Vec<u8>,
    /// Monotonically increasing observe sequence number (24-bit).
    pub seq:       u32,
}

// ── Registry ──────────────────────────────────────────────────────────────────

/// Thread-safe registry of CoAP Observe relations with notification support.
///
/// Each device can hold one active relation (matching ThingsBoard behaviour).
pub struct ObserveRegistry {
    relations:    RwLock<std::collections::HashMap<Uuid, ObserveRelation>>,
    /// Pending CON notifications waiting for ACK: message_id → Notify.
    pending_acks: DashMap<u16, Arc<Notify>>,
    socket:       Arc<UdpSocket>,
    /// Maximum CON retransmit attempts per RFC 7252 §4.2.
    max_retransmit: u32,
    /// Initial ACK timeout in milliseconds (RFC 7252 default: 2000ms).
    initial_timeout_ms: u64,
}

impl ObserveRegistry {
    pub fn new(socket: Arc<UdpSocket>) -> Arc<Self> {
        Arc::new(Self {
            relations:          RwLock::new(std::collections::HashMap::new()),
            pending_acks:       DashMap::new(),
            socket,
            max_retransmit:     4,
            initial_timeout_ms: 2_000,
        })
    }

    // ── Subscription management ───────────────────────────────────────────────

    pub async fn register(&self, device_id: Uuid, addr: SocketAddr, token: Vec<u8>) {
        let rel = ObserveRelation { device_id, addr, token, seq: 0 };
        self.relations.write().await.insert(device_id, rel);
        debug!(device_id = %device_id, peer = %addr, "CoAP Observe registered");
    }

    pub async fn deregister(&self, device_id: Uuid) {
        if self.relations.write().await.remove(&device_id).is_some() {
            debug!(device_id = %device_id, "CoAP Observe deregistered");
        }
    }

    pub async fn deregister_peer(&self, device_id: Uuid, peer: SocketAddr) {
        let mut guard = self.relations.write().await;
        if let Some(rel) = guard.get(&device_id) {
            if rel.addr == peer {
                guard.remove(&device_id);
                debug!(device_id = %device_id, peer = %peer, "CoAP Observe deregistered by peer");
            }
        }
    }

    // ── Incoming ACK handling ─────────────────────────────────────────────────

    /// Called by the packet handler when an ACK arrives for a CON notification.
    pub fn ack_received(&self, message_id: u16) {
        if let Some((_, notify)) = self.pending_acks.remove(&message_id) {
            notify.notify_one();
            debug!(message_id = message_id, "CoAP Observe ACK received");
        }
    }

    // ── Notification ─────────────────────────────────────────────────────────

    /// Send a CON observe notification to the registered observer for `device_id`.
    /// Retries up to `max_retransmit` times with exponential backoff.
    /// Deregisters the relation if no ACK arrives within the retry window.
    pub async fn notify_device(
        &self,
        device_id: Uuid,
        payload:   &[u8],
    ) -> Result<(), TransportError> {
        // Clone the relation so we can release the read lock before the async send.
        let rel = match self.relations.read().await.get(&device_id).cloned() {
            Some(r) => r,
            None    => return Ok(()), // no observer → nothing to do
        };

        let acked = self.send_con_with_retry(&rel, payload).await?;

        if !acked {
            warn!(device_id = %device_id, addr = %rel.addr,
                  "CoAP Observe: no ACK after retries, deregistering");
            self.deregister(device_id).await;
        } else {
            // Bump the sequence number for the next notification.
            let mut guard = self.relations.write().await;
            if let Some(r) = guard.get_mut(&device_id) {
                r.seq = r.seq.wrapping_add(1) & 0x00FF_FFFF; // 24-bit counter
            }
        }

        Ok(())
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    async fn send_con_with_retry(
        &self,
        rel:     &ObserveRelation,
        payload: &[u8],
    ) -> Result<bool, TransportError> {
        let message_id = random_message_id();
        let packet_bytes = build_notification_packet(&rel.token, message_id, rel.seq, payload)?;

        // Register ACK listener before the first send.
        let ack_notify = Arc::new(Notify::new());
        self.pending_acks.insert(message_id, ack_notify.clone());

        let mut timeout_ms = self.initial_timeout_ms;
        let mut acked = false;

        for attempt in 0..=self.max_retransmit {
            if attempt > 0 {
                debug!(device_id = %rel.device_id, attempt = attempt,
                       "CoAP Observe CON retransmit");
            }
            self.socket.send_to(&packet_bytes, rel.addr).await?;

            // Wait for ACK or timeout.
            match tokio::time::timeout(
                Duration::from_millis(timeout_ms),
                ack_notify.notified(),
            )
            .await
            {
                Ok(()) => {
                    acked = true;
                    break;
                }
                Err(_timeout) => {
                    // RFC 7252 §4.2: multiply timeout by a random factor in [1, 1.5].
                    // We use the deterministic 1.5× factor for simplicity.
                    timeout_ms = timeout_ms * 3 / 2;
                }
            }
        }

        // Remove ACK entry regardless of outcome.
        self.pending_acks.remove(&message_id);

        Ok(acked)
    }
}

// ── Packet construction ───────────────────────────────────────────────────────

/// Build a CON 2.05 Content packet carrying the observe sequence number.
fn build_notification_packet(
    token:      &[u8],
    message_id: u16,
    seq:        u32,
    payload:    &[u8],
) -> Result<Vec<u8>, TransportError> {
    let mut pkt = Packet::new();
    pkt.header.set_type(MessageType::Confirmable);
    pkt.header.message_id = message_id;
    pkt.header.code = coap_lite::MessageClass::Response(ResponseType::Content);
    pkt.set_token(token.to_vec());

    // Observe option: 3-byte big-endian sequence number (RFC 7641 §3).
    let seq_bytes = encode_observe_seq(seq);
    pkt.add_option(CoapOption::Observe, seq_bytes.to_vec());

    pkt.payload = payload.to_vec();

    pkt.to_bytes()
        .map_err(|_| TransportError::Protocol("CoAP packet encode error".into()))
}

/// Encode observe sequence number as a minimal-length big-endian byte sequence.
fn encode_observe_seq(seq: u32) -> Vec<u8> {
    if seq == 0 {
        vec![0]
    } else if seq <= 0xFF {
        vec![seq as u8]
    } else if seq <= 0xFFFF {
        vec![(seq >> 8) as u8, (seq & 0xFF) as u8]
    } else {
        vec![
            ((seq >> 16) & 0xFF) as u8,
            ((seq >> 8) & 0xFF) as u8,
            (seq & 0xFF) as u8,
        ]
    }
}

/// Generate a random 16-bit CoAP message ID.
fn random_message_id() -> u16 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;

    let mut h = DefaultHasher::new();
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos()
        .hash(&mut h);
    std::thread::current().id().hash(&mut h);
    h.finish() as u16
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_observe_seq_zero() {
        assert_eq!(encode_observe_seq(0), vec![0]);
    }

    #[test]
    fn test_encode_observe_seq_one_byte() {
        assert_eq!(encode_observe_seq(0xAB), vec![0xAB]);
    }

    #[test]
    fn test_encode_observe_seq_two_bytes() {
        assert_eq!(encode_observe_seq(0x0102), vec![0x01, 0x02]);
    }

    #[test]
    fn test_encode_observe_seq_three_bytes() {
        assert_eq!(encode_observe_seq(0x010203), vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_random_message_id_is_u16() {
        let id = random_message_id();
        // Just verify it doesn't panic and returns something.
        let _ = id;
    }

    #[test]
    fn test_build_notification_packet_round_trip() {
        let token   = vec![0xDE, 0xAD];
        let payload = b"{\"temperature\":25}";
        let bytes   = build_notification_packet(&token, 1234, 5, payload).unwrap();
        let pkt     = Packet::from_bytes(&bytes).unwrap();
        assert_eq!(pkt.header.message_id, 1234);
        assert_eq!(pkt.get_token(), &token);
        assert_eq!(&pkt.payload, payload);
    }

    #[tokio::test]
    async fn test_register_deregister() {
        // Use a real UDP socket so ObserveRegistry can be constructed.
        let socket = Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap());
        let reg    = ObserveRegistry::new(socket);
        let id     = Uuid::new_v4();
        let addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();

        reg.register(id, addr, vec![0x01]).await;
        {
            let guard = reg.relations.read().await;
            assert!(guard.contains_key(&id));
        }

        reg.deregister(id).await;
        {
            let guard = reg.relations.read().await;
            assert!(!guard.contains_key(&id));
        }
    }

    #[tokio::test]
    async fn test_ack_signals_notify() {
        let socket = Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap());
        let reg    = ObserveRegistry::new(socket);
        let notify = Arc::new(Notify::new());
        reg.pending_acks.insert(42u16, notify.clone());

        reg.ack_received(42);

        // The Notify should have been triggered — notified() should resolve immediately.
        tokio::time::timeout(Duration::from_millis(10), notify.notified())
            .await
            .expect("Notify should resolve instantly after ack_received");
    }
}
