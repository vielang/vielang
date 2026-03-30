use std::net::SocketAddr;
use std::sync::Arc;

use coap_lite::{CoapRequest, CoapOption, MessageType, Packet, RequestType, ResponseType};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, warn};

use vl_cache::TbCache;
use vl_core::entities::{TbMsg, msg_type};
use vl_dao::{postgres::{device::DeviceDao, kv::KvDao}, DbPool, TimeseriesDao};
use vl_queue::{TbProducer, topics};

use crate::auth::authenticate_by_token;
use crate::mqtt::telemetry::{save_client_attributes, save_telemetry};
use super::observe::ObserveRegistry;

// Observe option values (RFC 7641)
const OBSERVE_REGISTER:   u32 = 0;
const OBSERVE_DEREGISTER: u32 = 1;

pub struct CoapContext {
    pub pool:              DbPool,
    pub ts_dao:            Arc<dyn TimeseriesDao>,
    pub rule_engine_tx:    Arc<Option<mpsc::Sender<TbMsg>>>,
    pub queue_producer:    Arc<dyn TbProducer>,
    pub cache:             Arc<dyn TbCache>,
    pub ws_tx:             broadcast::Sender<TbMsg>,
    pub observe_registry:  Arc<ObserveRegistry>,
}

/// Process a single CoAP datagram; returns response bytes or None for silence.
pub async fn handle_packet(raw: &[u8], peer: SocketAddr, ctx: &CoapContext) -> Option<Vec<u8>> {
    let packet = match Packet::from_bytes(raw) {
        Ok(p)  => p,
        Err(e) => {
            debug!("CoAP parse error: {}", e);
            return None;
        }
    };

    let msg_type_hdr  = packet.header.get_type();
    let message_id    = packet.header.message_id;
    let token         = packet.get_token().to_vec();

    // ── Handle ACK (RFC 7641 §4.4): signal pending notifications ─────────────
    if msg_type_hdr == MessageType::Acknowledgement || msg_type_hdr == MessageType::Reset {
        ctx.observe_registry.ack_received(message_id);
        return None; // ACKs don't receive a response
    }

    let is_confirmable = msg_type_hdr == MessageType::Confirmable;

    let request = CoapRequest::from_packet(packet, peer);
    let method  = *request.get_method();
    // path: "api/v1/<access_token>/<resource>" and optionally "/<sub>"
    let path    = request.get_path();
    let parts: Vec<&str> = path.splitn(5, '/').collect();

    // Expected: ["api", "v1", "<access_token>", "<resource>"]
    if parts.len() < 4 || parts[0] != "api" || parts[1] != "v1" {
        let mut resp = request.response?;
        resp.set_status(ResponseType::BadRequest);
        resp.message.payload = b"Invalid path".to_vec();
        if is_confirmable {
            resp.message.header.set_type(MessageType::Acknowledgement);
            resp.message.header.message_id = message_id;
        }
        return resp.message.to_bytes().ok();
    }

    let access_token = parts[2];
    let resource     = parts[3];
    let sub_resource = parts.get(4).copied();

    let device_dao = DeviceDao::new(ctx.pool.clone());
    let auth = match authenticate_by_token(access_token, &device_dao, &ctx.cache).await {
        Some(a) => a,
        None => {
            let mut resp = request.response?;
            resp.set_status(ResponseType::Unauthorized);
            resp.message.payload = b"Unauthorized".to_vec();
            if is_confirmable {
                resp.message.header.set_type(MessageType::Acknowledgement);
                resp.message.header.message_id = message_id;
            }
            return resp.message.to_bytes().ok();
        }
    };

    let payload = request.message.payload.clone();

    // Check Observe option (RFC 7641)
    let observe_value = request.message.get_option(CoapOption::Observe)
        .and_then(|vals| vals.front())
        .map(|v| {
            let mut n = 0u32;
            for &b in v.iter() {
                n = (n << 8) | b as u32;
            }
            n
        });

    let response_bytes = match (method, resource, sub_resource) {
        // ── Telemetry ─────────────────────────────────────────────────────────
        (RequestType::Post, "telemetry", None) => {
            let mut resp = request.response?;
            match save_telemetry(&ctx.ts_dao, "DEVICE", auth.device_id, &payload).await {
                Ok(n) => {
                    debug!(device_id = %auth.device_id, entries = n, "CoAP telemetry saved");
                    let data = String::from_utf8_lossy(&payload).to_string();
                    let msg = TbMsg::new(msg_type::POST_TELEMETRY_REQUEST, auth.device_id, "DEVICE", &data);
                    send_to_rule_engine(&ctx.rule_engine_tx, msg.clone());
                    publish_to_queue(&ctx.queue_producer, topics::VL_TRANSPORT_API_REQUESTS, &msg).await;
                    let _ = ctx.ws_tx.send(msg);
                    resp.set_status(ResponseType::Changed);
                }
                Err(e) => {
                    warn!(device_id = %auth.device_id, error = %e, "CoAP telemetry save failed");
                    resp.set_status(ResponseType::BadRequest);
                    resp.message.payload = e.to_string().into_bytes();
                }
            }
            finalize_response_msg(&mut resp.message, is_confirmable, message_id);
            resp.message.to_bytes().ok()
        }
        // ── Attributes POST ───────────────────────────────────────────────────
        (RequestType::Post, "attributes", None) => {
            let kv_dao = KvDao::new(ctx.pool.clone());
            let mut resp = request.response?;
            match save_client_attributes(&kv_dao, auth.device_id, &payload).await {
                Ok(n) => {
                    debug!(device_id = %auth.device_id, entries = n, "CoAP attributes saved");
                    let data = String::from_utf8_lossy(&payload).to_string();
                    let msg = TbMsg::new(msg_type::POST_ATTRIBUTES_REQUEST, auth.device_id, "DEVICE", &data);
                    send_to_rule_engine(&ctx.rule_engine_tx, msg.clone());
                    publish_to_queue(&ctx.queue_producer, topics::VL_TRANSPORT_API_REQUESTS, &msg).await;
                    let _ = ctx.ws_tx.send(msg);
                    resp.set_status(ResponseType::Changed);
                }
                Err(e) => {
                    warn!(device_id = %auth.device_id, error = %e, "CoAP attributes save failed");
                    resp.set_status(ResponseType::BadRequest);
                    resp.message.payload = e.to_string().into_bytes();
                }
            }
            finalize_response_msg(&mut resp.message, is_confirmable, message_id);
            resp.message.to_bytes().ok()
        }
        // ── Attributes GET (with optional Observe) ────────────────────────────
        (RequestType::Get, "attributes", None) => {
            if let Some(obs) = observe_value {
                match obs {
                    OBSERVE_REGISTER => {
                        ctx.observe_registry.register(auth.device_id, peer, token.clone()).await;
                    }
                    OBSERVE_DEREGISTER => {
                        ctx.observe_registry.deregister_peer(auth.device_id, peer).await;
                    }
                    _ => {
                        debug!(device_id = %auth.device_id, value = obs, "Unknown CoAP Observe value");
                    }
                }
            }
            let mut resp = request.response?;
            resp.set_status(ResponseType::Content);
            resp.message.payload = b"{}".to_vec();
            finalize_response_msg(&mut resp.message, is_confirmable, message_id);
            resp.message.to_bytes().ok()
        }
        // ── RPC: device polls for server-initiated RPC (GET /rpc) ─────────────
        (RequestType::Get, "rpc", None) => {
            let mut rx = ctx.ws_tx.subscribe();
            let device_id = auth.device_id;

            let rpc_data = tokio::time::timeout(
                std::time::Duration::from_millis(100),
                async move {
                    loop {
                        match rx.recv().await {
                            Ok(msg)
                                if msg.originator_id == device_id
                                    && msg.msg_type == msg_type::RPC_CALL_FROM_SERVER =>
                            {
                                return serde_json::from_str::<serde_json::Value>(&msg.data)
                                    .unwrap_or(serde_json::json!({}));
                            }
                            Ok(_) => continue,
                            Err(_) => return serde_json::json!({}),
                        }
                    }
                },
            )
            .await
            .unwrap_or(serde_json::json!({}));

            let mut resp = request.response?;
            resp.set_status(ResponseType::Content);
            resp.message.payload = serde_json::to_vec(&rpc_data).unwrap_or_default();
            finalize_response_msg(&mut resp.message, is_confirmable, message_id);
            resp.message.to_bytes().ok()
        }
        // ── RPC: device responds to server-initiated RPC (POST /rpc/{id}) ─────
        (RequestType::Post, "rpc", Some(request_id)) => {
            let mut resp = request.response?;
            let rpc_data = String::from_utf8_lossy(&payload).to_string();
            info!(device_id = %auth.device_id, request_id = %request_id, "CoAP RPC response: {}", rpc_data);
            let msg = TbMsg::new(msg_type::RPC_CALL_FROM_SERVER, auth.device_id, "DEVICE", &rpc_data);
            send_to_rule_engine(&ctx.rule_engine_tx, msg.clone());
            publish_to_queue(&ctx.queue_producer, topics::VL_TRANSPORT_API_REQUESTS, &msg).await;
            resp.set_status(ResponseType::Changed);
            finalize_response_msg(&mut resp.message, is_confirmable, message_id);
            resp.message.to_bytes().ok()
        }
        // ── Fallback ──────────────────────────────────────────────────────────
        _ => {
            let mut resp = request.response?;
            resp.set_status(ResponseType::MethodNotAllowed);
            resp.message.payload = b"Method not allowed".to_vec();
            finalize_response_msg(&mut resp.message, is_confirmable, message_id);
            resp.message.to_bytes().ok()
        }
    };

    response_bytes
}

fn finalize_response_msg(msg: &mut coap_lite::Packet, is_confirmable: bool, message_id: u16) {
    if is_confirmable {
        msg.header.set_type(MessageType::Acknowledgement);
        msg.header.message_id = message_id;
    }
}

fn send_to_rule_engine(tx: &Arc<Option<mpsc::Sender<TbMsg>>>, msg: TbMsg) {
    if let Some(sender) = tx.as_ref() {
        if let Err(e) = sender.try_send(msg) {
            debug!("Rule engine channel full: {}", e);
        }
    }
}

async fn publish_to_queue(producer: &Arc<dyn TbProducer>, topic: &str, msg: &TbMsg) {
    if let Err(e) = producer.send_tb_msg(topic, msg).await {
        debug!("Queue publish error on {}: {}", topic, e);
    }
}
