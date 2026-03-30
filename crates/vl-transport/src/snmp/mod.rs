pub mod auth;
pub mod handler;
pub mod oid_registry;
pub mod session;

use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info, warn};

use vl_cache::TbCache;
use vl_config::SnmpConfig;
use vl_core::entities::{ActivityEvent, TbMsg, msg_type};
use vl_dao::{postgres::kv::KvDao, DbPool, TimeseriesDao};
use vl_queue::{TbProducer, topics};

use crate::mqtt::telemetry::build_ts_record;
use handler::{pdu_tag_to_version, varbinds_to_telemetry};
use session::{extract_uptime, SessionRegistry};

// ── BER tag constants ─────────────────────────────────────────────────────────

const TAG_INTEGER:     u8 = 0x02;
const TAG_OCTETSTRING: u8 = 0x04;
const TAG_OID:         u8 = 0x06;
const TAG_SEQUENCE:    u8 = 0x30;
const TAG_IPADDRESS:   u8 = 0x40;
const TAG_COUNTER32:   u8 = 0x41;
const TAG_GAUGE32:     u8 = 0x42;
const TAG_TIMETICKS:   u8 = 0x43;
const TAG_COUNTER64:   u8 = 0x46;
const TAG_TRAPV2:      u8 = 0xa7;
const TAG_TRAPV1:      u8 = 0xa4;

// ── BER reader ────────────────────────────────────────────────────────────────

/// Read a BER T+L pair. Advances `pos` past TL to point at start of value.
/// Returns `(tag, value_end)`.
fn read_tl(data: &[u8], pos: &mut usize) -> Option<(u8, usize)> {
    if *pos >= data.len() { return None; }
    let tag = data[*pos];
    *pos += 1;

    if *pos >= data.len() { return None; }
    let first = data[*pos];
    *pos += 1;

    let len = if first & 0x80 == 0 {
        first as usize
    } else {
        let n = (first & 0x7F) as usize;
        if n == 0 || n > 4 || *pos + n > data.len() { return None; }
        let mut l = 0usize;
        for _ in 0..n {
            l = (l << 8) | data[*pos] as usize;
            *pos += 1;
        }
        l
    };

    let end = *pos + len;
    if end > data.len() { return None; }
    Some((tag, end))
}

fn skip_tlv(data: &[u8], pos: &mut usize) -> bool {
    match read_tl(data, pos) {
        Some((_, end)) => { *pos = end; true }
        None => false,
    }
}

/// Decode a BER OID value bytes into dotted notation.
fn decode_oid(value: &[u8]) -> String {
    if value.is_empty() { return String::new(); }
    let first = value[0] as u32;
    let mut parts = vec![(first / 40).to_string(), (first % 40).to_string()];
    let mut i = 1;
    while i < value.len() {
        let mut c: u64 = 0;
        loop {
            if i >= value.len() { break; }
            let b = value[i]; i += 1;
            c = (c << 7) | (b & 0x7F) as u64;
            if b & 0x80 == 0 { break; }
        }
        parts.push(c.to_string());
    }
    parts.join(".")
}

fn decode_uint(value: &[u8]) -> u64 {
    value.iter().fold(0u64, |a, &b| (a << 8) | b as u64)
}

fn decode_int(value: &[u8]) -> i64 {
    if value.is_empty() { return 0; }
    let mut v = if value[0] & 0x80 != 0 { -1i64 } else { 0i64 };
    for &b in value { v = (v << 8) | b as i64; }
    v
}

// ── SNMP value → JSON ─────────────────────────────────────────────────────────

fn snmp_value_to_json(tag: u8, value: &[u8]) -> serde_json::Value {
    match tag {
        TAG_INTEGER     => serde_json::json!(decode_int(value)),
        TAG_OCTETSTRING => {
            let s = String::from_utf8_lossy(value).into_owned();
            serde_json::json!(s)
        }
        TAG_IPADDRESS if value.len() == 4 => {
            serde_json::json!(format!("{}.{}.{}.{}", value[0], value[1], value[2], value[3]))
        }
        TAG_COUNTER32 | TAG_GAUGE32 | TAG_TIMETICKS | TAG_COUNTER64 => {
            serde_json::json!(decode_uint(value))
        }
        other => serde_json::json!(format!(
            "0x{:02x}:{}",
            other,
            value.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join("")
        )),
    }
}

// ── SNMP trap parser ──────────────────────────────────────────────────────────

struct SnmpTrap {
    community: String,
    /// SNMP version: 1 (SNMPv1) or 2 (SNMPv2c)
    version:   u8,
    varbinds:  Vec<(String, serde_json::Value)>,
}

fn parse_snmp_trap(data: &[u8]) -> Option<SnmpTrap> {
    let mut pos = 0;

    // Top-level SEQUENCE
    let (tag, _) = read_tl(data, &mut pos)?;
    if tag != TAG_SEQUENCE { return None; }

    // Version INTEGER
    let (vtag, ver_end) = read_tl(data, &mut pos)?;
    if vtag != TAG_INTEGER { return None; }
    pos = ver_end;

    // Community OCTET STRING
    let (ctag, com_end) = read_tl(data, &mut pos)?;
    if ctag != TAG_OCTETSTRING { return None; }
    let community = String::from_utf8_lossy(&data[pos..com_end]).into_owned();
    pos = com_end;

    // PDU type (0xa7 = v2c TrapPDU, 0xa4 = v1 Trap-PDU)
    let (pdu_tag, _pdu_end) = read_tl(data, &mut pos)?;
    if pdu_tag != TAG_TRAPV2 && pdu_tag != TAG_TRAPV1 { return None; }
    let version = pdu_tag_to_version(pdu_tag);

    if pdu_tag == TAG_TRAPV1 {
        // v1: enterprise OID, agent-addr, generic-trap, specific-trap, time-stamp
        for _ in 0..5 { if !skip_tlv(data, &mut pos) { return None; } }
    } else {
        // v2c: request-id, error-status, error-index
        for _ in 0..3 { if !skip_tlv(data, &mut pos) { return None; } }
    }

    // VarBindList SEQUENCE
    let (vbl_tag, vbl_end) = read_tl(data, &mut pos)?;
    if vbl_tag != TAG_SEQUENCE { return None; }

    let mut varbinds = Vec::new();
    while pos < vbl_end {
        let (vb_tag, vb_end) = read_tl(data, &mut pos)?;
        if vb_tag != TAG_SEQUENCE { break; }

        // OID
        let (oid_tag, oid_end) = read_tl(data, &mut pos)?;
        if oid_tag != TAG_OID { break; }
        let oid_str = decode_oid(&data[pos..oid_end]);
        pos = oid_end;

        // Value
        let (val_tag, val_end) = read_tl(data, &mut pos)?;
        let json_val = snmp_value_to_json(val_tag, &data[pos..val_end]);

        varbinds.push((oid_str, json_val));
        pos = vb_end;
    }

    Some(SnmpTrap { community, version, varbinds })
}

// ── Transport run loop ────────────────────────────────────────────────────────

pub async fn run(
    pool:            DbPool,
    ts_dao:          Arc<dyn TimeseriesDao>,
    config:          SnmpConfig,
    rule_engine_tx:  Option<mpsc::Sender<TbMsg>>,
    queue_producer:  Arc<dyn TbProducer>,
    _cache:          Arc<dyn TbCache>,
    ws_tx:           broadcast::Sender<TbMsg>,
    activity_tx:     mpsc::Sender<ActivityEvent>,
) {
    let addr = format!("0.0.0.0:{}", config.bind_port);

    let socket = match UdpSocket::bind(&addr).await {
        Ok(s) => {
            info!(community = %config.community, "SNMP transport listening on udp:{}", addr);
            s
        }
        Err(e) => {
            error!("Failed to bind SNMP transport on {}: {}", addr, e);
            return;
        }
    };

    let rule_engine_tx = Arc::new(rule_engine_tx);
    let community      = config.community.clone();
    let sessions       = Arc::new(SessionRegistry::new(600)); // 10-min inactivity timeout
    let mut buf        = vec![0u8; 65535];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, peer)) => {
                debug!(peer = %peer, bytes = len, "SNMP trap received");

                let trap = match parse_snmp_trap(&buf[..len]) {
                    Some(t) => t,
                    None    => {
                        warn!(peer = %peer, "SNMP: failed to parse trap PDU");
                        continue;
                    }
                };

                // Authenticate community string
                if trap.community != community {
                    warn!(
                        peer      = %peer,
                        got       = %trap.community,
                        expected  = %community,
                        "SNMP community mismatch — trap dropped"
                    );
                    continue;
                }

                if trap.varbinds.is_empty() { continue; }

                // Update session state
                let uptime_cs = extract_uptime(&trap.varbinds);
                let is_new = sessions.record_trap(peer.ip(), trap.version, &trap.community, uptime_cs);
                if is_new {
                    info!(
                        peer    = %peer,
                        version = trap.version,
                        "New SNMP device connected"
                    );
                }

                // Resolve device by source IP via "snmpHost" server attribute
                let kv_dao    = KvDao::new(pool.clone());
                let device_id = kv_dao
                    .find_device_by_server_attr("snmpHost", &peer.ip().to_string())
                    .await
                    .ok()
                    .flatten();

                let ts = chrono::Utc::now().timestamp_millis();

                // Map OIDs to friendly names and filter meta-varbinds
                let telemetry_map = varbinds_to_telemetry(&trap.varbinds, true);
                let kv_json       = serde_json::Value::Object(telemetry_map);
                let data_str      = kv_json.to_string();

                // Save each mapped varbind as a timeseries entry
                if let serde_json::Value::Object(ref map) = kv_json {
                    let entity_id = device_id.unwrap_or(uuid::Uuid::nil());
                    let mut saved = 0usize;
                    for (key, val) in map {
                        let record = build_ts_record(entity_id, key, ts, val);
                        if let Err(e) = ts_dao.save_latest("DEVICE", &record).await {
                            warn!(peer = %peer, error = %e, "SNMP save_latest failed");
                        } else {
                            saved += 1;
                        }
                    }
                    debug!(peer = %peer, entries = saved, "SNMP telemetry saved");
                }

                // Forward to rule engine if device is known
                if let Some(dev_id) = device_id {
                    activity_tx.send(ActivityEvent::Telemetry { device_id: dev_id, ts }).await.ok();
                    let msg = TbMsg::new(msg_type::POST_TELEMETRY_REQUEST, dev_id, "DEVICE", &data_str);
                    if let Some(ref sender) = *rule_engine_tx {
                        if let Err(e) = sender.try_send(msg.clone()) {
                            debug!("SNMP rule engine channel: {}", e);
                        }
                    }
                    if let Err(e) = queue_producer.send_tb_msg(topics::VL_TRANSPORT_API_REQUESTS, &msg).await {
                        debug!("SNMP queue publish error: {}", e);
                    }
                    let _ = ws_tx.send(msg);
                } else {
                    debug!(peer = %peer, "SNMP trap from unknown device (no snmpHost match)");
                }
            }
            Err(e) => {
                error!("SNMP UDP receive error: {}", e);
            }
        }
    }
}
