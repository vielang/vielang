/// LwM2M Bootstrap Server — OMA LwM2M spec section 5.2.
///
/// Bootstrap allows new devices to receive server credentials and configuration
/// before registering with the management server. Flow:
///
/// 1. Device → Bootstrap Server: `POST /bs?ep=<endpoint>` (Bootstrap-Request)
/// 2. Bootstrap Server → Device: `2.04 Changed` (accept)
/// 3. Bootstrap Server → Device: `PUT /0/0` Bootstrap-Write (Security object)
/// 4. Bootstrap Server → Device: `PUT /1/0` Bootstrap-Write (Server object)
/// 5. Bootstrap Server → Device: `POST /bs` Bootstrap-Finish
/// 6. Device registers with Management Server using the provisioned credentials.
///
/// This implementation handles the inbound Bootstrap-Request and queues the
/// outbound Bootstrap-Write + Bootstrap-Finish messages via the CoAP socket.

use coap_lite::{MessageType, Packet, RequestType, ResponseType};
use tracing::{debug, info};

// ── Bootstrap session state ───────────────────────────────────────────────────

/// State of a bootstrap session for one device.
#[derive(Debug, Clone, PartialEq)]
pub enum BootstrapState {
    /// Bootstrap-Request received, responses queued
    Requested,
    /// Security object written (Object 0)
    SecurityWritten,
    /// Server object written (Object 1)
    ServerWritten,
    /// Bootstrap-Finish sent
    Finished,
}

/// Active bootstrap session.
#[derive(Debug, Clone)]
pub struct BootstrapSession {
    pub endpoint: String,
    pub state:    BootstrapState,
    pub created_at: i64,
}

// ── Bootstrap packet handler ──────────────────────────────────────────────────

/// Server URI and server short ID to provision onto the device.
#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    /// LwM2M Management Server URI (e.g., "coap://192.168.1.100:5783")
    pub server_uri:      String,
    /// Short Server ID to provision (Object 1, Resource 0)
    pub short_server_id: u16,
    /// Lifetime in seconds to provision (Object 1, Resource 1)
    pub lifetime:        u32,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            server_uri:      "coap://localhost:5783".to_string(),
            short_server_id: 1,
            lifetime:        300,
        }
    }
}

/// Handle an inbound Bootstrap-Request (`POST /bs?ep=<endpoint>`).
/// Returns the list of CoAP response PDUs to send to the device:
/// 1. 2.04 Changed (accept)
/// 2. PUT /0/0 (Security Object — NoSec for simplicity)
/// 3. PUT /1/0 (Server Object — lifetime, binding)
/// 4. POST /bs (Bootstrap-Finish)
pub fn handle_bootstrap_request(
    raw:    &[u8],
    config: &BootstrapConfig,
) -> Option<Vec<Vec<u8>>> {
    let packet = Packet::from_bytes(raw).ok()?;

    // Verify it's a Bootstrap-Request: POST to /bs
    let path = packet
        .get_option(coap_lite::CoapOption::UriPath)
        .and_then(|opts| {
            opts.iter()
                .filter_map(|v| std::str::from_utf8(v).ok())
                .collect::<Vec<_>>()
                .join("/")
                .into()
        })
        .unwrap_or_default();

    if path != "bs" {
        debug!(path = %path, "Bootstrap: unexpected path, ignoring");
        return None;
    }

    // Extract endpoint name from URI query: ?ep=<endpoint>
    let endpoint = packet
        .get_option(coap_lite::CoapOption::UriQuery)
        .and_then(|opts| {
            opts.iter()
                .filter_map(|v| std::str::from_utf8(v).ok())
                .find_map(|q| {
                    q.strip_prefix("ep=").map(|ep| ep.to_string())
                })
        })
        .unwrap_or_else(|| "unknown".to_string());

    info!(endpoint = %endpoint, "LwM2M Bootstrap-Request received");

    let message_id = packet.header.message_id;
    let is_con     = packet.header.get_type() == MessageType::Confirmable;
    let token      = packet.get_token().to_vec();

    // 1. Bootstrap-Request ACK (2.04 Changed)
    let ack = build_response_ack(message_id, is_con, &token, ResponseType::Changed, &[]);

    // 2. Bootstrap-Write Security Object (Object 0, Instance 0) — NoSec mode
    let security_tlv = build_security_object_tlv(&config.server_uri, config.short_server_id);
    let write_security = build_put_request(message_id.wrapping_add(1), &[0u8; 4], "0/0", &security_tlv);

    // 3. Bootstrap-Write Server Object (Object 1, Instance 0)
    let server_tlv = build_server_object_tlv(config.short_server_id, config.lifetime);
    let write_server = build_put_request(message_id.wrapping_add(2), &[0u8; 4], "1/0", &server_tlv);

    // 4. Bootstrap-Finish (POST /bs)
    let finish = build_bootstrap_finish(message_id.wrapping_add(3), &token);

    debug!(endpoint = %endpoint, "Bootstrap: queuing Security+Server write + Finish");

    Some(vec![ack, write_security, write_server, finish])
}

// ── PDU builders ─────────────────────────────────────────────────────────────

fn build_response_ack(
    message_id: u16,
    is_con:     bool,
    token:      &[u8],
    code:       ResponseType,
    payload:    &[u8],
) -> Vec<u8> {
    let mut pkt = Packet::new();
    pkt.header.set_type(if is_con { MessageType::Acknowledgement } else { MessageType::NonConfirmable });
    pkt.header.message_id = message_id;
    pkt.set_token(token.to_vec());
    pkt.header.code = coap_lite::MessageClass::Response(code);
    pkt.payload = payload.to_vec();
    pkt.to_bytes().unwrap_or_default()
}

/// Build a CON PUT request to write a resource path (e.g., "0/0").
fn build_put_request(message_id: u16, token: &[u8; 4], path: &str, payload: &[u8]) -> Vec<u8> {
    let mut pkt = Packet::new();
    pkt.header.set_type(MessageType::Confirmable);
    pkt.header.message_id = message_id;
    pkt.set_token(token.to_vec());
    pkt.header.code = coap_lite::MessageClass::Request(RequestType::Put);
    // Add URI-Path options
    for segment in path.split('/') {
        pkt.add_option(coap_lite::CoapOption::UriPath, segment.as_bytes().to_vec());
    }
    // Content-Format: TLV = 11542
    pkt.add_option(
        coap_lite::CoapOption::ContentFormat,
        (11542u16).to_be_bytes().to_vec(),
    );
    pkt.payload = payload.to_vec();
    pkt.to_bytes().unwrap_or_default()
}

/// Build Bootstrap-Finish (POST /bs with no payload).
fn build_bootstrap_finish(message_id: u16, token: &[u8]) -> Vec<u8> {
    let mut pkt = Packet::new();
    pkt.header.set_type(MessageType::Confirmable);
    pkt.header.message_id = message_id;
    pkt.set_token(token.to_vec());
    pkt.header.code = coap_lite::MessageClass::Request(RequestType::Post);
    pkt.add_option(coap_lite::CoapOption::UriPath, b"bs".to_vec());
    pkt.to_bytes().unwrap_or_default()
}

// ── TLV builders for bootstrap objects ───────────────────────────────────────

/// Encode LwM2M Security Object (0) Instance 0 as TLV.
/// Provisions NoSec mode pointing to the management server URI.
fn build_security_object_tlv(server_uri: &str, _short_server_id: u16) -> Vec<u8> {
    use super::codec::{encode_resource_i64, encode_resource_string};

    let mut tlv = Vec::new();
    // Resource 0: LwM2M Server URI
    tlv.extend(encode_resource_string(0, server_uri));
    // Resource 1: Bootstrap Server = false (0)
    tlv.extend(encode_resource_i64(1, 0));
    // Resource 2: Security Mode = 3 (NoSec)
    tlv.extend(encode_resource_i64(2, 3));
    // Resource 3: Public Key or Identity = empty
    tlv.extend(encode_resource_string(3, ""));
    // Resource 4: Server Public Key = empty
    tlv.extend(encode_resource_string(4, ""));
    // Resource 5: Secret Key = empty
    tlv.extend(encode_resource_string(5, ""));
    // Resource 10: Short Server ID
    tlv.extend(encode_resource_i64(10, _short_server_id as i64));
    tlv
}

/// Encode LwM2M Server Object (1) Instance 0 as TLV.
fn build_server_object_tlv(short_server_id: u16, lifetime: u32) -> Vec<u8> {
    use super::codec::{encode_resource_i64, encode_resource_string};

    let mut tlv = Vec::new();
    // Resource 0: Short Server ID
    tlv.extend(encode_resource_i64(0, short_server_id as i64));
    // Resource 1: Lifetime
    tlv.extend(encode_resource_i64(1, lifetime as i64));
    // Resource 6: Notification Storing = true (1)
    tlv.extend(encode_resource_i64(6, 1));
    // Resource 7: Binding = "U" (UDP)
    tlv.extend(encode_resource_string(7, "U"));
    tlv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_config_defaults() {
        let cfg = BootstrapConfig::default();
        assert_eq!(cfg.short_server_id, 1);
        assert_eq!(cfg.lifetime, 300);
        assert!(cfg.server_uri.contains("5783"));
    }

    #[test]
    fn test_build_security_tlv_not_empty() {
        let tlv = build_security_object_tlv("coap://localhost:5783", 1);
        assert!(!tlv.is_empty());
    }

    #[test]
    fn test_build_server_tlv_not_empty() {
        let tlv = build_server_object_tlv(1, 300);
        assert!(!tlv.is_empty());
    }

    #[test]
    fn test_bootstrap_session_state() {
        let sess = BootstrapSession {
            endpoint:   "ep-test".to_string(),
            state:      BootstrapState::Requested,
            created_at: 0,
        };
        assert_eq!(sess.state, BootstrapState::Requested);
    }
}
