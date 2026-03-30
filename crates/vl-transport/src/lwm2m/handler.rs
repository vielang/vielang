use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use coap_lite::{CoapRequest, MessageType, Packet, RequestType, ResponseType};
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, info, warn};
use uuid::Uuid;

use vl_cache::TbCache;
use vl_core::entities::{ActivityEvent, TbMsg, msg_type};
use vl_dao::{postgres::device::DeviceDao, DbPool, TimeseriesDao};
use vl_queue::{TbProducer, topics};

use crate::auth::authenticate_by_token;
use crate::mqtt::telemetry::save_telemetry;
use crate::lwm2m::senml::{parse_senml, senml_to_kv};

// ── Session types ─────────────────────────────────────────────────────────────

/// A parsed `</objectId/instanceId>` link from the LwM2M registration payload.
#[derive(Debug, Clone)]
pub struct Lwm2mObjectLink {
    pub object_id:   u16,
    pub instance_id: Option<u16>,
}

/// Per-device session state stored in the in-memory registry.
#[derive(Debug, Clone)]
pub struct Lwm2mSession {
    pub device_id:     Uuid,
    pub endpoint:      String,
    pub lifetime:      u32,
    pub objects:       Vec<Lwm2mObjectLink>,
    pub registered_at: i64,
}

// ── Context shared across all packet handlers ─────────────────────────────────

pub struct Lwm2mContext {
    pub pool:           DbPool,
    pub ts_dao:         Arc<dyn TimeseriesDao>,
    pub rule_engine_tx: Arc<Option<mpsc::Sender<TbMsg>>>,
    pub queue_producer: Arc<dyn TbProducer>,
    pub cache:          Arc<dyn TbCache>,
    pub ws_tx:          broadcast::Sender<TbMsg>,
    pub activity_tx:    mpsc::Sender<ActivityEvent>,
    /// Maps registration token → active LwM2M session.
    pub registry:       Arc<RwLock<HashMap<String, Lwm2mSession>>>,
}

// ── Public packet handler ─────────────────────────────────────────────────────

/// Process a single LwM2M datagram (CoAP over UDP).
/// Returns response bytes to send back to `peer`, or `None` for silence.
pub async fn handle_packet(raw: &[u8], peer: SocketAddr, ctx: &Lwm2mContext) -> Option<Vec<u8>> {
    let packet = match Packet::from_bytes(raw) {
        Ok(p) => p,
        Err(e) => {
            debug!("LwM2M CoAP parse error from {}: {}", peer, e);
            return None;
        }
    };

    let is_confirmable = packet.header.get_type() == MessageType::Confirmable;
    let message_id     = packet.header.message_id;

    let request = CoapRequest::from_packet(packet, peer);
    let method  = *request.get_method();
    // coap_lite returns path without leading slash
    let path    = request.get_path();

    // Parse path segments: ["rd"] or ["rd", "<token>"]
    let parts: Vec<&str> = path.splitn(3, '/').collect();

    match (method, parts.as_slice()) {
        // POST /rd?ep=<endpoint>&lt=<lifetime>&b=U  →  Registration
        (RequestType::Post, ["rd"]) | (RequestType::Post, []) => {
            handle_register(request, peer, is_confirmable, message_id, ctx).await
        }

        // PUT /rd/<token>  →  Registration Update
        (RequestType::Put, ["rd", token]) => {
            handle_update(request, token, peer, is_confirmable, message_id, ctx).await
        }

        // DELETE /rd/<token>  →  Deregistration
        (RequestType::Delete, ["rd", token]) => {
            handle_deregister(request, token, peer, is_confirmable, message_id, ctx).await
        }

        // POST /rd/<token>  →  Notify (device-initiated telemetry)
        (RequestType::Post, ["rd", token]) => {
            handle_notify(request, token, peer, is_confirmable, message_id, ctx).await
        }

        _ => {
            debug!("LwM2M: unhandled path '{}' method {:?} from {}", path, method, peer);
            let mut resp = request.response?;
            resp.set_status(ResponseType::NotFound);
            resp.message.payload = b"Not found".to_vec();
            finalize_response(&mut resp.message, is_confirmable, message_id);
            resp.message.to_bytes().ok()
        }
    }
}

// ── Registration (POST /rd) ───────────────────────────────────────────────────

async fn handle_register(
    request:        CoapRequest<SocketAddr>,
    peer:           SocketAddr,
    is_confirmable: bool,
    message_id:     u16,
    ctx:            &Lwm2mContext,
) -> Option<Vec<u8>> {
    // Extract query parameters: ep, lt, b
    let query_string = get_query_string(&request);
    let params       = parse_query(&query_string);

    let endpoint = match params.get("ep") {
        Some(ep) => ep.clone(),
        None => {
            warn!("LwM2M: Registration missing 'ep' param from {}", peer);
            let mut resp = request.response?;
            resp.set_status(ResponseType::BadRequest);
            resp.message.payload = b"Missing endpoint name".to_vec();
            finalize_response(&mut resp.message, is_confirmable, message_id);
            return resp.message.to_bytes().ok();
        }
    };

    let lifetime: u32 = params.get("lt")
        .and_then(|v| v.parse().ok())
        .unwrap_or(86400); // default 24h per LwM2M spec

    // Authenticate: endpoint name is used as access token
    let device_dao = DeviceDao::new(ctx.pool.clone());
    let auth = match authenticate_by_token(&endpoint, &device_dao, &ctx.cache).await {
        Some(a) => a,
        None => {
            warn!("LwM2M: Auth failed for endpoint '{}' from {}", endpoint, peer);
            let mut resp = request.response?;
            resp.set_status(ResponseType::Unauthorized);
            resp.message.payload = b"Unauthorized".to_vec();
            finalize_response(&mut resp.message, is_confirmable, message_id);
            return resp.message.to_bytes().ok();
        }
    };

    // Parse object links from payload: </1/0>,</3/0>,</3303/0>
    let objects = parse_object_links(&request.message.payload);

    // Use a short unique token derived from device_id (first 8 hex chars)
    let reg_token = format!("{}", &auth.device_id.to_string()[..8]);

    let ts = chrono::Utc::now().timestamp_millis();
    let session = Lwm2mSession {
        device_id:     auth.device_id,
        endpoint:       endpoint.clone(),
        lifetime,
        objects,
        registered_at: ts,
    };

    ctx.registry.write().await.insert(reg_token.clone(), session);

    // Notify activity service
    let _ = ctx.activity_tx.try_send(ActivityEvent::Connected {
        device_id: auth.device_id,
        ts,
    });

    info!(
        device_id = %auth.device_id,
        endpoint  = %endpoint,
        lifetime  = lifetime,
        reg_token = %reg_token,
        "LwM2M device registered"
    );

    // Respond 2.01 Created with Location-Path: /rd/<reg_token>
    let mut resp = request.response?;
    resp.set_status(ResponseType::Created);
    // Add Location-Path options: "rd" and the token
    resp.message.add_option(coap_lite::CoapOption::LocationPath, b"rd".to_vec());
    resp.message.add_option(coap_lite::CoapOption::LocationPath, reg_token.into_bytes());
    finalize_response(&mut resp.message, is_confirmable, message_id);
    resp.message.to_bytes().ok()
}

// ── Registration Update (PUT /rd/<token>) ─────────────────────────────────────

async fn handle_update(
    request:        CoapRequest<SocketAddr>,
    token:          &str,
    peer:           SocketAddr,
    is_confirmable: bool,
    message_id:     u16,
    ctx:            &Lwm2mContext,
) -> Option<Vec<u8>> {
    let mut registry = ctx.registry.write().await;
    if let Some(session) = registry.get_mut(token) {
        // Update lifetime if provided in query
        let query_string = get_query_string(&request);
        let params       = parse_query(&query_string);
        if let Some(lt) = params.get("lt").and_then(|v| v.parse().ok()) {
            session.lifetime = lt;
        }
        // Update object links if payload present
        if !request.message.payload.is_empty() {
            session.objects = parse_object_links(&request.message.payload);
        }
        debug!(token = %token, peer = %peer, "LwM2M registration updated");

        let mut resp = request.response?;
        resp.set_status(ResponseType::Changed);
        finalize_response(&mut resp.message, is_confirmable, message_id);
        resp.message.to_bytes().ok()
    } else {
        warn!("LwM2M: Update for unknown token '{}' from {}", token, peer);
        let mut resp = request.response?;
        resp.set_status(ResponseType::NotFound);
        resp.message.payload = b"Registration not found".to_vec();
        finalize_response(&mut resp.message, is_confirmable, message_id);
        resp.message.to_bytes().ok()
    }
}

// ── Deregistration (DELETE /rd/<token>) ──────────────────────────────────────

async fn handle_deregister(
    request:        CoapRequest<SocketAddr>,
    token:          &str,
    peer:           SocketAddr,
    is_confirmable: bool,
    message_id:     u16,
    ctx:            &Lwm2mContext,
) -> Option<Vec<u8>> {
    let removed = ctx.registry.write().await.remove(token);
    if let Some(session) = removed {
        let ts = chrono::Utc::now().timestamp_millis();
        let _ = ctx.activity_tx.try_send(ActivityEvent::Disconnected {
            device_id: session.device_id,
            ts,
        });
        info!(
            device_id = %session.device_id,
            endpoint  = %session.endpoint,
            token     = %token,
            "LwM2M device deregistered"
        );
    } else {
        warn!("LwM2M: Deregister for unknown token '{}' from {}", token, peer);
    }

    let mut resp = request.response?;
    resp.set_status(ResponseType::Deleted);
    finalize_response(&mut resp.message, is_confirmable, message_id);
    resp.message.to_bytes().ok()
}

// ── Notify (POST /rd/<token>) ─────────────────────────────────────────────────

async fn handle_notify(
    request:        CoapRequest<SocketAddr>,
    token:          &str,
    peer:           SocketAddr,
    is_confirmable: bool,
    message_id:     u16,
    ctx:            &Lwm2mContext,
) -> Option<Vec<u8>> {
    let session = {
        let registry = ctx.registry.read().await;
        registry.get(token).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => {
            warn!("LwM2M: Notify for unknown token '{}' from {}", token, peer);
            let mut resp = request.response?;
            resp.set_status(ResponseType::NotFound);
            resp.message.payload = b"Registration not found".to_vec();
            finalize_response(&mut resp.message, is_confirmable, message_id);
            return resp.message.to_bytes().ok();
        }
    };

    let payload = &request.message.payload;

    // Try to parse as SenML first; fall back to plain ThingsBoard JSON
    let kv_json: serde_json::Value = if let Some(records) = try_parse_senml(payload) {
        senml_to_kv(&records)
    } else {
        // Assume plain TB JSON: {"key": value, ...}
        match serde_json::from_slice(payload) {
            Ok(v) => v,
            Err(e) => {
                warn!(
                    device_id = %session.device_id,
                    error = %e,
                    "LwM2M: Notify payload is neither SenML nor valid JSON"
                );
                let mut resp = request.response?;
                resp.set_status(ResponseType::BadRequest);
                resp.message.payload = b"Invalid payload".to_vec();
                finalize_response(&mut resp.message, is_confirmable, message_id);
                return resp.message.to_bytes().ok();
            }
        }
    };

    // Serialize back to bytes for save_telemetry
    let kv_bytes = match serde_json::to_vec(&kv_json) {
        Ok(b) => b,
        Err(e) => {
            warn!(device_id = %session.device_id, error = %e, "LwM2M: Failed to re-serialize kv");
            let mut resp = request.response?;
            resp.set_status(ResponseType::InternalServerError);
            finalize_response(&mut resp.message, is_confirmable, message_id);
            return resp.message.to_bytes().ok();
        }
    };

    match save_telemetry(&ctx.ts_dao, "DEVICE", session.device_id, &kv_bytes).await {
        Ok(n) => {
            debug!(device_id = %session.device_id, entries = n, "LwM2M telemetry saved");

            let ts = chrono::Utc::now().timestamp_millis();
            let _ = ctx.activity_tx.try_send(ActivityEvent::Telemetry {
                device_id: session.device_id,
                ts,
            });

            let data = String::from_utf8_lossy(&kv_bytes).to_string();
            let msg  = TbMsg::new(msg_type::POST_TELEMETRY_REQUEST, session.device_id, "DEVICE", &data);
            send_to_rule_engine(&ctx.rule_engine_tx, msg.clone());
            publish_to_queue(&ctx.queue_producer, topics::VL_TRANSPORT_API_REQUESTS, &msg).await;
            let _ = ctx.ws_tx.send(msg);

            let mut resp = request.response?;
            resp.set_status(ResponseType::Changed);
            finalize_response(&mut resp.message, is_confirmable, message_id);
            resp.message.to_bytes().ok()
        }
        Err(e) => {
            warn!(device_id = %session.device_id, error = %e, "LwM2M telemetry save failed");
            let mut resp = request.response?;
            resp.set_status(ResponseType::BadRequest);
            resp.message.payload = e.to_string().into_bytes();
            finalize_response(&mut resp.message, is_confirmable, message_id);
            resp.message.to_bytes().ok()
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Attempt SenML parse — only succeeds if the bytes start with `[` (JSON array).
fn try_parse_senml(data: &[u8]) -> Option<Vec<crate::lwm2m::senml::SenmlRecord>> {
    // SenML is a JSON array; plain TB telemetry is a JSON object
    let trimmed = data.iter().position(|&b| b == b'[' || b == b'{')?;
    if data[trimmed] == b'[' {
        parse_senml(data)
    } else {
        None
    }
}

/// Parse LwM2M object links from registration payload.
/// Format: `</1>,</1/0>,</3/0>,</3303/0/5700>`
fn parse_object_links(payload: &[u8]) -> Vec<Lwm2mObjectLink> {
    let text = match std::str::from_utf8(payload) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut links = Vec::new();
    for part in text.split(',') {
        let part = part.trim();
        // Strip surrounding < >
        let inner = part.trim_start_matches('<').trim_end_matches('>');
        let inner = inner.trim_start_matches('/');
        if inner.is_empty() {
            continue;
        }
        let mut segments = inner.splitn(3, '/');
        if let Some(obj_str) = segments.next() {
            if let Ok(object_id) = obj_str.parse::<u16>() {
                let instance_id = segments.next().and_then(|s| s.parse::<u16>().ok());
                links.push(Lwm2mObjectLink { object_id, instance_id });
            }
        }
    }
    links
}

/// Collect CoAP URI-Query options into a single `key=value&...` string.
fn get_query_string(request: &CoapRequest<SocketAddr>) -> String {
    request
        .message
        .get_option(coap_lite::CoapOption::UriQuery)
        .map(|opts| {
            opts.iter()
                .filter_map(|v| std::str::from_utf8(v).ok())
                .collect::<Vec<_>>()
                .join("&")
        })
        .unwrap_or_default()
}

/// Parse `key=value&key2=value2` into a HashMap.
fn parse_query(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut it = pair.splitn(2, '=');
            let key = it.next()?.to_string();
            let val = it.next().unwrap_or("").to_string();
            if key.is_empty() { None } else { Some((key, val)) }
        })
        .collect()
}

/// Set ACK header if message was Confirmable.
fn finalize_response(msg: &mut coap_lite::Packet, is_confirmable: bool, message_id: u16) {
    if is_confirmable {
        msg.header.set_type(MessageType::Acknowledgement);
        msg.header.message_id = message_id;
    }
}

fn send_to_rule_engine(tx: &Arc<Option<mpsc::Sender<TbMsg>>>, msg: TbMsg) {
    if let Some(sender) = tx.as_ref() {
        if let Err(e) = sender.try_send(msg) {
            debug!("LwM2M: Rule engine channel full: {}", e);
        }
    }
}

async fn publish_to_queue(producer: &Arc<dyn TbProducer>, topic: &str, msg: &TbMsg) {
    if let Err(e) = producer.send_tb_msg(topic, msg).await {
        debug!("LwM2M: Queue publish error on {}: {}", topic, e);
    }
}
