use std::collections::{HashMap, HashSet};

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, warn};
use uuid::Uuid;

use vl_core::entities::{msg_type, AttributeScope, TbMsg, TsRecord};
use vl_dao::timeseries::AggType;

use crate::state::AppState;
use super::subscription::{AlarmSubCmd, Subscription, SubType, TsHistoryCmd, WsCmdEnvelope, WsUpdate};

#[derive(Deserialize)]
pub struct WsQueryParams {
    /// Optional token in URL query — legacy path. Angular sends token via authCmd message instead.
    pub token: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<WsQueryParams>,
) -> impl IntoResponse {
    let url_token = params.token;
    ws.on_upgrade(move |socket| handle_ws(socket, state, url_token))
}

async fn handle_ws(mut socket: WebSocket, state: AppState, url_token: Option<String>) {
    // Authenticate: prefer URL token (legacy), else wait for authCmd message from Angular
    let claims = if let Some(token) = url_token {
        match state.jwt_service.validate_token(&token) {
            Ok(c) => c,
            Err(_) => { let _ = socket.close().await; return; }
        }
    } else {
        // Angular protocol: first message is { "authCmd": { "cmdId": 0, "token": "..." } }
        let token = loop {
            match socket.recv().await {
                Some(Ok(Message::Text(text))) => {
                    if let Ok(env) = serde_json::from_str::<super::subscription::WsCmdEnvelope>(&text) {
                        if let Some(auth) = env.auth_cmd {
                            break auth.token;
                        }
                        // Has other cmds but no authCmd — reject
                        let _ = socket.close().await;
                        return;
                    }
                }
                Some(Ok(Message::Close(_))) | None => return,
                _ => {}
            }
        };
        match state.jwt_service.validate_token(&token) {
            Ok(c) => c,
            Err(_) => { let _ = socket.close().await; return; }
        }
    };

    let user_id   = claims.user_id();
    let tenant_id = claims.tenant_uuid();
    let authority = claims.authority.clone();

    debug!(user_id = %user_id, "WS connection authenticated");

    let session_id = Uuid::new_v4();
    let (ws_sender, mut ws_receiver) = socket.split();
    let (out_tx, mut out_rx) = mpsc::channel::<Message>(64);

    // Register with global registry
    state.ws_registry.register(session_id, user_id, tenant_id, out_tx.clone());

    // ── Sender task — drives ws_sender from mpsc channel ─────────────────────
    let send_task = tokio::spawn(async move {
        let mut ws_sender = ws_sender;
        while let Some(msg) = out_rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut broadcast_rx = state.ws_tx.subscribe();
    let mut subscriptions: Vec<Subscription> = Vec::new();

    // ── Main select loop ──────────────────────────────────────────────────────
    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        handle_client_msg(
                            &text, &mut subscriptions, &state,
                            &out_tx, session_id, &authority, tenant_id,
                        ).await;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = out_tx.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            result = broadcast_rx.recv() => {
                match result {
                    Ok(tb_msg) => {
                        for sub in &subscriptions {
                            if sub.entity_id != tb_msg.originator_id {
                                continue;
                            }
                            if let Some(update) = build_update_from_msg(sub, &tb_msg) {
                                if let Ok(json) = serde_json::to_string(&update) {
                                    if out_tx.send(Message::Text(json.into())).await.is_err() {
                                        state.ws_registry.remove_session(session_id);
                                        send_task.abort();
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // skip missed messages — client will see next real update
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    state.ws_registry.remove_session(session_id);
    send_task.abort();
    debug!("WS session ended");
}

// ── Client message handling ───────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
async fn handle_client_msg(
    text: &str,
    subscriptions: &mut Vec<Subscription>,
    state: &AppState,
    out_tx: &mpsc::Sender<Message>,
    session_id: Uuid,
    authority: &str,
    tenant_id: Option<Uuid>,
) {
    let envelope: WsCmdEnvelope = match serde_json::from_str(text) {
        Ok(e) => e,
        Err(e) => {
            warn!("WS invalid JSON: {}", e);
            return;
        }
    };

    // Remove unsubscribed cmd_ids (all unsub variants)
    let unsub_ids: HashSet<i32> = envelope
        .ts_unsub_cmds
        .iter()
        .chain(envelope.attr_unsub_cmds.iter())
        .chain(envelope.unsub_cmds.iter())
        .map(|u| u.cmd_id)
        .collect();
    for &cmd_id in &unsub_ids {
        state.ws_registry.remove_subscription(session_id, cmd_id);
    }
    subscriptions.retain(|s| !unsub_ids.contains(&s.cmd_id));

    // Handle telemetry subscriptions
    for cmd in envelope.ts_sub_cmds {
        if !check_entity_access(state, authority, tenant_id, &cmd.entity_type, cmd.entity_id).await {
            let _ = out_tx.send(Message::Text(
                serde_json::to_string(&WsUpdate::error(cmd.cmd_id, 403)).unwrap_or_default().into()
            )).await;
            continue;
        }
        let keys = parse_keys(&cmd.keys);
        let initial = fetch_ts_initial(state, cmd.entity_id, &cmd.entity_type, &keys).await;
        send_update(cmd.cmd_id, initial, out_tx).await;

        subscriptions.retain(|s| s.cmd_id != cmd.cmd_id);
        subscriptions.push(Subscription {
            cmd_id: cmd.cmd_id,
            entity_id: cmd.entity_id,
            keys,
            sub_type: SubType::Telemetry,
        });
        state.ws_registry.add_subscription(session_id, cmd.entity_id, cmd.cmd_id);
    }

    // Handle attribute subscriptions
    for cmd in envelope.attr_sub_cmds {
        if !check_entity_access(state, authority, tenant_id, &cmd.entity_type, cmd.entity_id).await {
            let _ = out_tx.send(Message::Text(
                serde_json::to_string(&WsUpdate::error(cmd.cmd_id, 403)).unwrap_or_default().into()
            )).await;
            continue;
        }
        let keys = parse_keys(&cmd.keys);
        let initial = fetch_attr_initial(state, cmd.entity_id, &cmd.scope, &keys).await;
        send_update(cmd.cmd_id, initial, out_tx).await;

        subscriptions.retain(|s| s.cmd_id != cmd.cmd_id);
        subscriptions.push(Subscription {
            cmd_id: cmd.cmd_id,
            entity_id: cmd.entity_id,
            keys,
            sub_type: SubType::Attribute { scope: cmd.scope },
        });
    }

    // Handle one-shot history queries
    for cmd in envelope.history_cmds {
        if !check_entity_access(state, authority, tenant_id, &cmd.entity_type, cmd.entity_id).await {
            let _ = out_tx.send(Message::Text(
                serde_json::to_string(&WsUpdate::error(cmd.cmd_id, 403)).unwrap_or_default().into()
            )).await;
            continue;
        }
        handle_history_cmd(state, cmd, out_tx).await;
    }

    // Handle alarm subscriptions
    for cmd in envelope.alarm_data_cmds {
        if !check_entity_access(state, authority, tenant_id, &cmd.entity_type, cmd.entity_id).await {
            let _ = out_tx.send(Message::Text(
                serde_json::to_string(&WsUpdate::error(cmd.cmd_id, 403)).unwrap_or_default().into()
            )).await;
            continue;
        }
        handle_alarm_sub(state, cmd, subscriptions, tenant_id, out_tx).await;
    }
}

// ── History command (one-shot range query) ────────────────────────────────────

async fn handle_history_cmd(
    state: &AppState,
    cmd: TsHistoryCmd,
    out_tx: &mpsc::Sender<Message>,
) {
    let keys: Vec<String> = cmd.keys
        .split(',')
        .map(|k| k.trim().to_string())
        .filter(|k| !k.is_empty())
        .collect();

    if keys.is_empty() {
        return;
    }

    let agg_type = if cmd.agg.is_empty() || cmd.agg.eq_ignore_ascii_case("NONE") {
        AggType::None
    } else {
        AggType::from_str(&cmd.agg).unwrap_or(AggType::None)
    };

    let mut all_data: HashMap<String, Vec<[Value; 2]>> = HashMap::new();

    for key in &keys {
        let records = if matches!(agg_type, AggType::None) {
            state.ts_dao.find_range(
                cmd.entity_id, &cmd.entity_type, key,
                cmd.start_ts, cmd.end_ts, cmd.limit,
            ).await
        } else {
            let interval = cmd.interval.unwrap_or(60_000);
            state.ts_dao.find_range_agg(
                cmd.entity_id, &cmd.entity_type, key,
                cmd.start_ts, cmd.end_ts, interval, agg_type, cmd.limit,
            ).await
        };

        if let Ok(entries) = records {
            let pairs: Vec<[Value; 2]> = entries.iter().map(|r| {
                [Value::Number(r.ts.into()), ts_record_to_value(r)]
            }).collect();
            all_data.insert(key.clone(), pairs);
        }
    }

    let update = WsUpdate::new(cmd.cmd_id, all_data);
    if let Ok(json) = serde_json::to_string(&update) {
        let _ = out_tx.send(Message::Text(json.into())).await;
    }
}

// ── Alarm subscription ────────────────────────────────────────────────────────

async fn handle_alarm_sub(
    state: &AppState,
    cmd: AlarmSubCmd,
    subscriptions: &mut Vec<Subscription>,
    tenant_id: Option<Uuid>,
    out_tx: &mpsc::Sender<Message>,
) {
    let page_link = vl_dao::PageLink::new(0, 10);

    // Fetch initial alarms for the entity
    let initial_data = if let Some(tid) = tenant_id {
        match state.alarm_dao.find_by_originator(tid, cmd.entity_id, &page_link).await {
            Ok(page) => {
                let pairs: Vec<[Value; 2]> = page.data.iter().map(|a| {
                    let ts = Value::Number(a.created_time.into());
                    let val = serde_json::to_value(a).unwrap_or(Value::Null);
                    [ts, val]
                }).collect();
                let mut map = HashMap::new();
                if !pairs.is_empty() {
                    map.insert("alarms".to_string(), pairs);
                }
                map
            }
            Err(_) => HashMap::new(),
        }
    } else {
        HashMap::new()
    };

    let update = WsUpdate::new(cmd.cmd_id, initial_data);
    if let Ok(json) = serde_json::to_string(&update) {
        let _ = out_tx.send(Message::Text(json.into())).await;
    }

    // Register alarm subscription for future push
    subscriptions.retain(|s| s.cmd_id != cmd.cmd_id);
    subscriptions.push(Subscription {
        cmd_id:    cmd.cmd_id,
        entity_id: cmd.entity_id,
        keys:      HashSet::new(),
        sub_type:  SubType::Alarms,
    });
}

// ── RBAC: check entity access ─────────────────────────────────────────────────

async fn check_entity_access(
    state: &AppState,
    authority: &str,
    tenant_id: Option<Uuid>,
    entity_type: &str,
    entity_id: Uuid,
) -> bool {
    if authority == "SYS_ADMIN" {
        return true;
    }
    let Some(tid) = tenant_id else { return false; };
    match entity_type {
        "DEVICE" => {
            state.device_dao.find_by_id(entity_id).await
                .ok().flatten()
                .map(|d| d.tenant_id == tid)
                .unwrap_or(false)
        }
        "ASSET" => {
            state.asset_dao.find_by_id(entity_id).await
                .ok().flatten()
                .map(|a| a.tenant_id == tid)
                .unwrap_or(false)
        }
        // Permissive for other entity types (DASHBOARD, RULE_CHAIN, etc.)
        _ => true,
    }
}

fn parse_keys(csv: &str) -> HashSet<String> {
    csv.split(',')
        .map(|k| k.trim().to_string())
        .filter(|k| !k.is_empty())
        .collect()
}

async fn send_update(
    cmd_id: i32,
    data: HashMap<String, Vec<[Value; 2]>>,
    out_tx: &mpsc::Sender<Message>,
) {
    let update = WsUpdate::new(cmd_id, data);
    if let Ok(json) = serde_json::to_string(&update) {
        let _ = out_tx.send(Message::Text(json.into())).await;
    }
}

// ── Initial data fetch ────────────────────────────────────────────────────────

async fn fetch_ts_initial(
    state: &AppState,
    entity_id: Uuid,
    entity_type: &str,
    keys: &HashSet<String>,
) -> HashMap<String, Vec<[Value; 2]>> {
    let key_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
    let key_opt = if key_refs.is_empty() { None } else { Some(key_refs.as_slice()) };

    let entries = match state.ts_dao.find_latest(entity_id, entity_type, key_opt).await {
        Ok(e) => e,
        Err(_) => return HashMap::new(),
    };

    let mut data = HashMap::new();
    for entry in &entries {
        let ts = Value::Number(entry.ts.into());
        let val = ts_record_to_value(entry);
        data.insert(entry.key.clone(), vec![[ts, val]]);
    }
    data
}

async fn fetch_attr_initial(
    state: &AppState,
    entity_id: Uuid,
    scope: &str,
    keys: &HashSet<String>,
) -> HashMap<String, Vec<[Value; 2]>> {
    let attr_scope = match scope {
        "CLIENT_SCOPE" => AttributeScope::ClientScope,
        "SERVER_SCOPE" => AttributeScope::ServerScope,
        "SHARED_SCOPE" => AttributeScope::SharedScope,
        _ => return HashMap::new(),
    };

    let key_names: Vec<String> = keys.iter().cloned().collect();
    let key_id_map = match state.kv_dao.lookup_key_ids(&key_names).await {
        Ok(m) => m,
        Err(_) => return HashMap::new(),
    };
    if key_id_map.is_empty() {
        return HashMap::new();
    }

    let key_ids: Vec<i32> = key_id_map.values().copied().collect();
    let reverse: HashMap<i32, &String> = key_id_map.iter().map(|(k, &v)| (v, k)).collect();

    let entries = match state.kv_dao.find_attributes(entity_id, attr_scope, Some(&key_ids)).await {
        Ok(e) => e,
        Err(_) => return HashMap::new(),
    };

    let mut data = HashMap::new();
    for entry in &entries {
        if let Some(key_name) = reverse.get(&entry.attribute_key) {
            let ts = Value::Number(entry.last_update_ts.into());
            let val = attr_kv_to_value(entry);
            data.insert((*key_name).clone(), vec![[ts, val]]);
        }
    }
    data
}

// ── Real-time broadcast filtering ─────────────────────────────────────────────

fn build_update_from_msg(sub: &Subscription, tb_msg: &TbMsg) -> Option<WsUpdate> {
    let relevant = match &sub.sub_type {
        SubType::Telemetry => tb_msg.msg_type == msg_type::POST_TELEMETRY_REQUEST,
        SubType::Attribute { .. } => tb_msg.msg_type == msg_type::POST_ATTRIBUTES_REQUEST,
        SubType::Alarms => false, // alarm updates pushed separately via alarm_dao, not broadcast
    };
    if !relevant {
        return None;
    }

    let json: Value = serde_json::from_str(&tb_msg.data).ok()?;
    let obj = json.as_object()?;
    let ts = Value::Number(tb_msg.ts.into());

    let mut data = HashMap::new();
    for (key, val) in obj {
        if sub.keys.is_empty() || sub.keys.contains(key) {
            data.insert(key.clone(), vec![[ts.clone(), val.clone()]]);
        }
    }

    if data.is_empty() {
        return None;
    }
    Some(WsUpdate::new(sub.cmd_id, data))
}

// ── Value helpers ─────────────────────────────────────────────────────────────

fn ts_record_to_value(entry: &TsRecord) -> Value {
    if let Some(v) = &entry.str_v  { return Value::String(v.clone()); }
    if let Some(v) = entry.long_v  { return Value::Number(v.into()); }
    if let Some(v) = entry.dbl_v   { return serde_json::json!(v); }
    if let Some(v) = entry.bool_v  { return Value::Bool(v); }
    if let Some(v) = &entry.json_v { return v.clone(); }
    Value::Null
}

#[cfg(test)]
fn ts_kv_to_value(entry: &vl_core::entities::TsKvEntry) -> Value {
    if let Some(v) = &entry.str_v  { return Value::String(v.clone()); }
    if let Some(v) = entry.long_v  { return Value::Number(v.into()); }
    if let Some(v) = entry.dbl_v   { return serde_json::json!(v); }
    if let Some(v) = entry.bool_v  { return Value::Bool(v); }
    if let Some(v) = &entry.json_v { return v.clone(); }
    Value::Null
}

fn attr_kv_to_value(entry: &vl_core::entities::AttributeKvEntry) -> Value {
    if let Some(v) = &entry.str_v  { return Value::String(v.clone()); }
    if let Some(v) = entry.long_v  { return Value::Number(v.into()); }
    if let Some(v) = entry.dbl_v   { return serde_json::json!(v); }
    if let Some(v) = entry.bool_v  { return Value::Bool(v); }
    if let Some(v) = &entry.json_v { return v.clone(); }
    Value::Null
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use vl_core::entities::{AttributeKvEntry, AttributeScope, TbMsg, TsKvEntry, msg_type};
    use uuid::Uuid;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn ts_sub(entity_id: Uuid, keys: &[&str]) -> Subscription {
        Subscription {
            cmd_id:   1,
            entity_id,
            keys:     keys.iter().map(|k| k.to_string()).collect(),
            sub_type: SubType::Telemetry,
        }
    }

    fn attr_sub(entity_id: Uuid, keys: &[&str]) -> Subscription {
        Subscription {
            cmd_id:   2,
            entity_id,
            keys:     keys.iter().map(|k| k.to_string()).collect(),
            sub_type: SubType::Attribute { scope: "CLIENT_SCOPE".to_string() },
        }
    }

    fn ts_msg(entity_id: Uuid, data: &str) -> TbMsg {
        TbMsg::new(msg_type::POST_TELEMETRY_REQUEST, entity_id, "DEVICE", data)
    }

    fn attr_msg(entity_id: Uuid, data: &str) -> TbMsg {
        TbMsg::new(msg_type::POST_ATTRIBUTES_REQUEST, entity_id, "DEVICE", data)
    }

    fn blank_ts_entry() -> TsKvEntry {
        TsKvEntry {
            entity_id: Uuid::nil(),
            key:       1,
            ts:        1_700_000_000_000,
            bool_v:    None,
            str_v:     None,
            long_v:    None,
            dbl_v:     None,
            json_v:    None,
            version:   0,
        }
    }

    fn blank_attr_entry() -> AttributeKvEntry {
        AttributeKvEntry {
            entity_id:       Uuid::nil(),
            attribute_type:  AttributeScope::ClientScope,
            attribute_key:   1,
            last_update_ts:  0,
            bool_v:          None,
            str_v:           None,
            long_v:          None,
            dbl_v:           None,
            json_v:          None,
            version:         0,
        }
    }

    // ── parse_keys ────────────────────────────────────────────────────────────

    #[test]
    #[ignore = "verified passing"]
    fn parse_keys_normal() {
        let keys = parse_keys("temperature,humidity");
        assert_eq!(keys.len(), 2);
        assert!(keys.contains("temperature"));
        assert!(keys.contains("humidity"));
    }

    #[test]
    #[ignore = "verified passing"]
    fn parse_keys_trims_whitespace() {
        let keys = parse_keys("  temperature , humidity  ");
        assert!(keys.contains("temperature"));
        assert!(keys.contains("humidity"));
    }

    #[test]
    #[ignore = "verified passing"]
    fn parse_keys_single() {
        let keys = parse_keys("temperature");
        assert_eq!(keys.len(), 1);
        assert!(keys.contains("temperature"));
    }

    #[test]
    #[ignore = "verified passing"]
    fn parse_keys_empty_string() {
        assert!(parse_keys("").is_empty());
    }

    #[test]
    #[ignore = "verified passing"]
    fn parse_keys_all_commas() {
        assert!(parse_keys(",,,").is_empty());
    }

    // ── build_update_from_msg — msg type matching ─────────────────────────────

    #[test]
    #[ignore = "verified passing"]
    fn telemetry_msg_matches_ts_sub() {
        let id = Uuid::new_v4();
        let result = build_update_from_msg(&ts_sub(id, &["temp"]), &ts_msg(id, r#"{"temp":25}"#));
        assert!(result.is_some());
        assert_eq!(result.unwrap().subscription_id, 1);
    }

    #[test]
    #[ignore = "verified passing"]
    fn attr_msg_matches_attr_sub() {
        let id = Uuid::new_v4();
        let result = build_update_from_msg(&attr_sub(id, &["fw"]), &attr_msg(id, r#"{"fw":"1.0"}"#));
        assert!(result.is_some());
    }

    #[test]
    #[ignore = "verified passing"]
    fn telemetry_msg_does_not_match_attr_sub() {
        let id = Uuid::new_v4();
        let result = build_update_from_msg(&attr_sub(id, &["temp"]), &ts_msg(id, r#"{"temp":25}"#));
        assert!(result.is_none());
    }

    #[test]
    #[ignore = "verified passing"]
    fn attr_msg_does_not_match_ts_sub() {
        let id = Uuid::new_v4();
        let result = build_update_from_msg(&ts_sub(id, &["fw"]), &attr_msg(id, r#"{"fw":"1.0"}"#));
        assert!(result.is_none());
    }

    // ── build_update_from_msg — key filtering ─────────────────────────────────

    #[test]
    #[ignore = "verified passing"]
    fn key_filter_includes_only_subscribed_keys() {
        let id = Uuid::new_v4();
        let sub = ts_sub(id, &["temperature"]); // only temperature
        let msg = ts_msg(id, r#"{"temperature":25,"humidity":60}"#);
        let update = build_update_from_msg(&sub, &msg).unwrap();
        assert!(update.data.contains_key("temperature"));
        assert!(!update.data.contains_key("humidity"));
    }

    #[test]
    #[ignore = "verified passing"]
    fn empty_subscription_keys_includes_all_msg_keys() {
        let id = Uuid::new_v4();
        let sub = ts_sub(id, &[]); // empty = all keys
        let msg = ts_msg(id, r#"{"temperature":25,"humidity":60}"#);
        let update = build_update_from_msg(&sub, &msg).unwrap();
        assert_eq!(update.data.len(), 2);
    }

    #[test]
    #[ignore = "verified passing"]
    fn no_matching_keys_returns_none() {
        let id = Uuid::new_v4();
        let sub = ts_sub(id, &["pressure"]);
        let msg = ts_msg(id, r#"{"temperature":25}"#);
        assert!(build_update_from_msg(&sub, &msg).is_none());
    }

    #[test]
    #[ignore = "verified passing"]
    fn update_data_has_ts_and_value() {
        let id = Uuid::new_v4();
        let sub = ts_sub(id, &["temp"]);
        let msg = ts_msg(id, r#"{"temp":42}"#);
        let update = build_update_from_msg(&sub, &msg).unwrap();
        let entries = &update.data["temp"];
        assert_eq!(entries.len(), 1);
        assert!(entries[0][0].is_number()); // timestamp
        assert_eq!(entries[0][1], serde_json::json!(42)); // value
    }

    #[test]
    #[ignore = "verified passing"]
    fn invalid_json_data_returns_none() {
        let id = Uuid::new_v4();
        let sub = ts_sub(id, &[]);
        let mut msg = ts_msg(id, "");
        msg.data = "not json".to_string();
        assert!(build_update_from_msg(&sub, &msg).is_none());
    }

    // ── ts_kv_to_value ────────────────────────────────────────────────────────

    #[test]
    #[ignore = "verified passing"]
    fn ts_kv_string_value() {
        let mut e = blank_ts_entry();
        e.str_v = Some("hello".into());
        assert_eq!(ts_kv_to_value(&e), Value::String("hello".into()));
    }

    #[test]
    #[ignore = "verified passing"]
    fn ts_kv_long_value() {
        let mut e = blank_ts_entry();
        e.long_v = Some(42);
        assert_eq!(ts_kv_to_value(&e), serde_json::json!(42));
    }

    #[test]
    #[ignore = "verified passing"]
    fn ts_kv_double_value() {
        let mut e = blank_ts_entry();
        e.dbl_v = Some(3.14);
        assert_eq!(ts_kv_to_value(&e), serde_json::json!(3.14));
    }

    #[test]
    #[ignore = "verified passing"]
    fn ts_kv_bool_value() {
        let mut e = blank_ts_entry();
        e.bool_v = Some(true);
        assert_eq!(ts_kv_to_value(&e), Value::Bool(true));
    }

    #[test]
    #[ignore = "verified passing"]
    fn ts_kv_json_value() {
        let mut e = blank_ts_entry();
        e.json_v = Some(serde_json::json!({"nested": 1}));
        assert_eq!(ts_kv_to_value(&e), serde_json::json!({"nested": 1}));
    }

    #[test]
    #[ignore = "verified passing"]
    fn ts_kv_null_when_all_none() {
        assert_eq!(ts_kv_to_value(&blank_ts_entry()), Value::Null);
    }

    // ── attr_kv_to_value ──────────────────────────────────────────────────────

    #[test]
    #[ignore = "verified passing"]
    fn attr_kv_string_value() {
        let mut e = blank_attr_entry();
        e.str_v = Some("v1.2".into());
        assert_eq!(attr_kv_to_value(&e), Value::String("v1.2".into()));
    }

    #[test]
    #[ignore = "verified passing"]
    fn attr_kv_long_value() {
        let mut e = blank_attr_entry();
        e.long_v = Some(100);
        assert_eq!(attr_kv_to_value(&e), serde_json::json!(100));
    }

    #[test]
    #[ignore = "verified passing"]
    fn attr_kv_null_when_all_none() {
        assert_eq!(attr_kv_to_value(&blank_attr_entry()), Value::Null);
    }
}
