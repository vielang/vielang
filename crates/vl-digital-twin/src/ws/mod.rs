//! WebSocket connection tới VieLang backend.
//!
//! Native: background thread với Tokio runtime, reconnect tự động với exponential backoff.
//! WASM:   spawn_local với gloo-net, kết nối một lần (reconnect là TODO).
//!
//! Events được pass sang Bevy main thread qua Arc<Mutex<VecDeque>>.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::config::TwinConfig;

// ── Shared event queue ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum WsIncomingEvent {
    Telemetry {
        subscription_id: i32,
        data: HashMap<String, Vec<[Value; 2]>>,
    },
    /// Shared attribute update (SHARED_SCOPE) — flat key→value map.
    AttributeUpdate {
        subscription_id: i32,
        data: HashMap<String, Value>,
    },
    Connected,
    /// WS mất kết nối — kèm lý do
    Disconnected(String),
    /// Đang chờ reconnect — dùng để update UI countdown
    Reconnecting { attempt: u32, delay_ms: u64 },
}

/// Thread-safe queue để pass WS events từ background thread sang Bevy.
#[derive(Resource, Clone)]
pub struct WsEventQueue(pub Arc<Mutex<VecDeque<WsIncomingEvent>>>);

impl WsEventQueue {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(VecDeque::new())))
    }

    pub fn push(&self, event: WsIncomingEvent) {
        if let Ok(mut q) = self.0.lock() {
            q.push_back(event);
        }
    }

    /// Drain tất cả pending events — gọi mỗi frame bởi Bevy system.
    pub fn drain(&self) -> Vec<WsIncomingEvent> {
        if let Ok(mut q) = self.0.lock() {
            q.drain(..).collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for WsEventQueue {
    fn default() -> Self { Self::new() }
}

// ── Config ────────────────────────────────────────────────────────────────────

/// WS connection config — được tạo từ TwinConfig khi khởi động.
#[derive(Resource, Debug, Clone)]
pub struct WsConfig {
    /// e.g. "ws://localhost:8080"
    pub server_url:        String,
    /// JWT access token
    pub jwt_token:         String,
    /// Thời gian chờ đầu tiên trước khi reconnect (ms)
    pub initial_delay_ms:  u64,
    /// Hệ số nhân backoff
    pub backoff_factor:    f64,
    /// Thời gian chờ tối đa (ms)
    pub max_delay_ms:      u64,
}

impl WsConfig {
    pub fn from_twin_config(cfg: &TwinConfig) -> Self {
        Self {
            server_url:       cfg.server.ws_url.clone(),
            jwt_token:        cfg.auth.token.clone(),
            initial_delay_ms: cfg.reconnect.initial_delay_ms,
            backoff_factor:   cfg.reconnect.backoff_factor,
            max_delay_ms:     cfg.reconnect.max_delay_ms,
        }
    }
}

impl Default for WsConfig {
    fn default() -> Self {
        // Fallback khi không có TwinConfig — đọc env vars
        Self {
            server_url:       std::env::var("TB_SERVER_URL").unwrap_or_else(|_| "ws://localhost:8080".into()),
            jwt_token:        std::env::var("TB_TOKEN").unwrap_or_default(),
            initial_delay_ms: 1_000,
            backoff_factor:   2.0,
            max_delay_ms:     60_000,
        }
    }
}

/// Connection status — resource được update bởi drain_ws_events mỗi frame.
#[derive(Resource, Debug, Default)]
pub struct WsConnectionStatus {
    pub connected:           bool,
    pub error:               Option<String>,
    /// Số lần reconnect đã thực hiện trong session
    pub reconnect_attempts:  u32,
    /// Đang trong quá trình chờ reconnect
    pub reconnecting:        bool,
    /// Delay (ms) trước lần thử tiếp theo
    pub next_retry_ms:       Option<u64>,
    /// Timestamp (ms) khi mất kết nối lần cuối
    pub disconnected_at_ms:  Option<i64>,
}

// ── WS protocol types ─────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WsCmdEnvelope {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ts_sub_cmds:   Vec<TsSubCmd>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attr_sub_cmds: Vec<AttrSubCmd>,
}

/// Subscribe to SHARED_SCOPE attributes for a device.
/// cmd_id is offset by ATTR_CMD_OFFSET from the telemetry cmd_id.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttrSubCmd {
    pub cmd_id:      i32,
    pub entity_type: String,
    pub entity_id:   Uuid,
    pub scope:       String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TsSubCmd {
    pub cmd_id:      i32,
    pub entity_type: String,
    pub entity_id:   Uuid,
    pub keys:        String,
}

/// Update nhận từ backend.
/// Format: { subscriptionId, errorCode, data: { key: [[ts_ms, value_str], ...] } }
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WsUpdate {
    pub subscription_id: i32,
    #[serde(default)]
    pub error_code: i32,
    #[serde(default)]
    pub data: HashMap<String, Vec<[Value; 2]>>,
}

// ── Subscriptions ─────────────────────────────────────────────────────────────

/// Attribute cmd_ids are telemetry cmd_id + this offset to avoid conflicts.
pub const ATTR_CMD_OFFSET: i32 = 1_000;

/// Mapping cmdId → device_id for active telemetry and attribute subscriptions.
#[derive(Resource, Default, Clone)]
pub struct WsSubscriptions {
    /// Telemetry subscriptions: (cmd_id, device_id)
    pub entries:      Vec<(i32, Uuid)>,
    /// Attribute subscriptions: (attr_cmd_id, device_id)
    pub attr_entries: Vec<(i32, Uuid)>,
    next_cmd_id:      i32,
}

impl WsSubscriptions {
    /// Register a telemetry subscription. Returns the assigned cmd_id.
    pub fn add(&mut self, device_id: Uuid) -> i32 {
        self.next_cmd_id += 1;
        let cmd_id = self.next_cmd_id;
        self.entries.push((cmd_id, device_id));
        self.attr_entries.push((cmd_id + ATTR_CMD_OFFSET, device_id));
        cmd_id
    }

    pub fn device_for_cmd(&self, cmd_id: i32) -> Option<Uuid> {
        self.entries.iter().find(|(c, _)| *c == cmd_id).map(|(_, d)| *d)
    }

    pub fn device_for_attr_cmd(&self, cmd_id: i32) -> Option<Uuid> {
        self.attr_entries.iter().find(|(c, _)| *c == cmd_id).map(|(_, d)| *d)
    }

    /// Returns attr sub commands for all current subscriptions.
    pub fn attr_sub_cmds(&self) -> Vec<AttrSubCmd> {
        self.attr_entries
            .iter()
            .map(|(cmd_id, device_id)| AttrSubCmd {
                cmd_id:      *cmd_id,
                entity_type: "DEVICE".into(),
                entity_id:   *device_id,
                scope:       "SHARED_SCOPE".into(),
            })
            .collect()
    }
}

// ── Native WS implementation ──────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    use super::*;
    use std::time::Duration;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    /// Spawn background thread chạy reconnect loop liên tục.
    pub fn start_ws(
        config:        WsConfig,
        queue:         WsEventQueue,
        subscriptions: Vec<(i32, Uuid)>,
        attr_subs:     Vec<(i32, Uuid)>,
    ) {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            rt.block_on(reconnect_loop(config, queue, subscriptions, attr_subs));
        });
    }

    /// Reconnect loop với exponential backoff.
    /// Chạy mãi mãi — dừng khi thread bị drop (app exit).
    async fn reconnect_loop(
        config:        WsConfig,
        queue:         WsEventQueue,
        subscriptions: Vec<(i32, Uuid)>,
        attr_subs:     Vec<(i32, Uuid)>,
    ) {
        let url = build_ws_url(&config);
        let mut delay_ms = config.initial_delay_ms;
        let mut attempt: u32 = 0;

        loop {
            attempt += 1;
            tracing::info!(attempt, url = %url, "WS connecting");

            match connect_and_run(&url, &queue, &subscriptions, &attr_subs).await {
                Ok(_) => {
                    // Đóng bình thường (server restart?) — reset backoff và reconnect ngay
                    tracing::info!("WS closed cleanly, reconnecting shortly");
                    attempt = 0;
                    delay_ms = config.initial_delay_ms;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Err(e) => {
                    tracing::warn!(attempt, error = %e, delay_ms, "WS error, will retry");
                    queue.push(WsIncomingEvent::Reconnecting { attempt, delay_ms });
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    delay_ms = ((delay_ms as f64 * config.backoff_factor) as u64)
                        .min(config.max_delay_ms);
                }
            }
        }
    }

    /// Kết nối một lần và đọc messages cho đến khi đóng.
    /// Trả về Ok nếu đóng bình thường, Err nếu lỗi kết nối.
    async fn connect_and_run(
        url:           &str,
        queue:         &WsEventQueue,
        subscriptions: &[(i32, Uuid)],
        attr_subs:     &[(i32, Uuid)],
    ) -> Result<(), String> {
        // Validate URL
        url::Url::parse(url).map_err(|e| format!("Invalid URL: {e}"))?;

        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| format!("Connect failed: {e}"))?;

        queue.push(WsIncomingEvent::Connected);
        tracing::info!("WS connected");

        let (mut write, mut read) = ws_stream.split();

        // Gửi telemetry + attribute subscription commands together
        if !subscriptions.is_empty() {
            let ts_cmds: Vec<TsSubCmd> = subscriptions
                .iter()
                .map(|(cmd_id, device_id)| TsSubCmd {
                    cmd_id:      *cmd_id,
                    entity_type: "DEVICE".into(),
                    entity_id:   *device_id,
                    keys:        String::new(), // empty = subscribe tất cả keys
                })
                .collect();

            let attr_cmds: Vec<AttrSubCmd> = attr_subs
                .iter()
                .map(|(cmd_id, device_id)| AttrSubCmd {
                    cmd_id:      *cmd_id,
                    entity_type: "DEVICE".into(),
                    entity_id:   *device_id,
                    scope:       "SHARED_SCOPE".into(),
                })
                .collect();

            let envelope = WsCmdEnvelope {
                ts_sub_cmds:   ts_cmds,
                attr_sub_cmds: attr_cmds,
            };
            if let Ok(json) = serde_json::to_string(&envelope) {
                write.send(Message::Text(json.into())).await
                    .map_err(|e| format!("Send subscription failed: {e}"))?;
                tracing::debug!(
                    ts   = subscriptions.len(),
                    attr = attr_subs.len(),
                    "Subscriptions sent"
                );
            }
        }

        // Build an attr_cmd_id set for fast lookup
        let attr_cmd_ids: std::collections::HashSet<i32> =
            attr_subs.iter().map(|(id, _)| *id).collect();

        // Read loop
        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(Message::Text(raw)) => {
                    if let Ok(update) = serde_json::from_str::<WsUpdate>(&raw) {
                        if update.error_code != 0 { continue; }

                        if attr_cmd_ids.contains(&update.subscription_id) {
                            // Attribute update — flat key→Value map
                            if !update.data.is_empty() {
                                let flat: HashMap<String, Value> = update.data
                                    .into_iter()
                                    .filter_map(|(k, v)| {
                                        // Attribute values are [[ts, val]] arrays; take val
                                        v.last().and_then(|e| e.get(1).cloned())
                                            .map(|val| (k, val))
                                    })
                                    .collect();
                                if !flat.is_empty() {
                                    queue.push(WsIncomingEvent::AttributeUpdate {
                                        subscription_id: update.subscription_id,
                                        data:            flat,
                                    });
                                }
                            }
                        } else if !update.data.is_empty() {
                            // Telemetry update
                            queue.push(WsIncomingEvent::Telemetry {
                                subscription_id: update.subscription_id,
                                data:            update.data,
                            });
                        }
                    }
                }
                Ok(Message::Ping(d)) => {
                    let _ = write.send(Message::Pong(d)).await;
                }
                Ok(Message::Close(_)) => {
                    tracing::info!("WS close frame received");
                    queue.push(WsIncomingEvent::Disconnected("Server closed connection".into()));
                    return Ok(());
                }
                Ok(_) => {}
                Err(e) => {
                    queue.push(WsIncomingEvent::Disconnected(format!("Read error: {e}")));
                    return Err(format!("Read error: {e}"));
                }
            }
        }

        queue.push(WsIncomingEvent::Disconnected("Stream ended".into()));
        Ok(())
    }

    fn build_ws_url(config: &WsConfig) -> String {
        // Đảm bảo URL dùng ws:// hoặc wss://
        let base = config.server_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        let base = base.trim_end_matches('/');
        format!("{base}/api/ws/plugins/telemetry?token={}", config.jwt_token)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::start_ws;

// ── WASM WS implementation ────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use gloo_net::websocket::{futures::WebSocket, Message as GlooMsg};
    use gloo_timers::future::sleep;
    use wasm_bindgen_futures::spawn_local;

    /// WASM: spawn async task với reconnect loop dùng gloo-timers.
    pub fn start_ws(
        config:        WsConfig,
        queue:         WsEventQueue,
        subscriptions: Vec<(i32, Uuid)>,
        attr_subs:     Vec<(i32, Uuid)>,
    ) {
        spawn_local(async move {
            reconnect_loop(config, queue, subscriptions, attr_subs).await;
        });
    }

    /// Exponential backoff reconnect loop — mirrors native implementation.
    async fn reconnect_loop(
        config:        WsConfig,
        queue:         WsEventQueue,
        subscriptions: Vec<(i32, Uuid)>,
        attr_subs:     Vec<(i32, Uuid)>,
    ) {
        let base = config.server_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        let base = base.trim_end_matches('/');
        let url  = format!("{base}/api/ws/plugins/telemetry?token={}", config.jwt_token);

        let mut delay_ms = config.initial_delay_ms;
        let mut attempt: u32 = 0;

        loop {
            attempt += 1;
            match ws_connect_once(&url, &queue, &subscriptions, &attr_subs).await {
                Ok(_) => {
                    // Clean close — reset backoff
                    attempt  = 0;
                    delay_ms = config.initial_delay_ms;
                    sleep(std::time::Duration::from_millis(500)).await;
                }
                Err(_) => {
                    queue.push(WsIncomingEvent::Reconnecting { attempt, delay_ms });
                    sleep(std::time::Duration::from_millis(delay_ms)).await;
                    delay_ms = ((delay_ms as f64 * config.backoff_factor) as u64)
                        .min(config.max_delay_ms);
                }
            }
        }
    }

    /// Connect once and read until close/error.
    async fn ws_connect_once(
        url:           &str,
        queue:         &WsEventQueue,
        subscriptions: &[(i32, Uuid)],
        attr_subs:     &[(i32, Uuid)],
    ) -> Result<(), String> {
        let ws = WebSocket::open(url)
            .map_err(|e| format!("WASM WS connect failed: {e:?}"))?;

        queue.push(WsIncomingEvent::Connected);
        tracing::info!("WASM WS connected");

        let (mut sink, mut stream) = ws.split();

        // Send telemetry + attribute subscriptions together
        if !subscriptions.is_empty() {
            let ts_cmds: Vec<TsSubCmd> = subscriptions
                .iter()
                .map(|(cmd_id, device_id)| TsSubCmd {
                    cmd_id:      *cmd_id,
                    entity_type: "DEVICE".into(),
                    entity_id:   *device_id,
                    keys:        String::new(),
                })
                .collect();

            let attr_cmds: Vec<AttrSubCmd> = attr_subs
                .iter()
                .map(|(cmd_id, device_id)| AttrSubCmd {
                    cmd_id:      *cmd_id,
                    entity_type: "DEVICE".into(),
                    entity_id:   *device_id,
                    scope:       "SHARED_SCOPE".into(),
                })
                .collect();

            let envelope = WsCmdEnvelope { ts_sub_cmds: ts_cmds, attr_sub_cmds: attr_cmds };
            if let Ok(json) = serde_json::to_string(&envelope) {
                sink.send(GlooMsg::Text(json)).await
                    .map_err(|e| format!("Subscription send failed: {e:?}"))?;
            }
        }

        let attr_cmd_ids: std::collections::HashSet<i32> =
            attr_subs.iter().map(|(id, _)| *id).collect();

        // Read loop
        while let Some(msg_result) = stream.next().await {
            match msg_result {
                Ok(GlooMsg::Text(raw)) => {
                    if let Ok(update) = serde_json::from_str::<WsUpdate>(&raw) {
                        if update.error_code != 0 { continue; }

                        if attr_cmd_ids.contains(&update.subscription_id) {
                            if !update.data.is_empty() {
                                let flat: HashMap<String, Value> = update.data
                                    .into_iter()
                                    .filter_map(|(k, v)| {
                                        v.last()
                                            .and_then(|e| e.get(1).cloned())
                                            .map(|val| (k, val))
                                    })
                                    .collect();
                                if !flat.is_empty() {
                                    queue.push(WsIncomingEvent::AttributeUpdate {
                                        subscription_id: update.subscription_id,
                                        data:            flat,
                                    });
                                }
                            }
                        } else if !update.data.is_empty() {
                            queue.push(WsIncomingEvent::Telemetry {
                                subscription_id: update.subscription_id,
                                data:            update.data,
                            });
                        }
                    }
                }
                Ok(GlooMsg::Bytes(_)) => {}
                Err(e) => {
                    queue.push(WsIncomingEvent::Disconnected(format!("WASM WS error: {e:?}")));
                    return Err(format!("WS error: {e:?}"));
                }
            }
        }

        queue.push(WsIncomingEvent::Disconnected("WASM WS stream ended".into()));
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::start_ws;

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_event_queue_push_drain() {
        let q = WsEventQueue::new();
        q.push(WsIncomingEvent::Connected);
        q.push(WsIncomingEvent::Disconnected("lost".into()));

        let events = q.drain();
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], WsIncomingEvent::Connected));
        assert!(matches!(&events[1], WsIncomingEvent::Disconnected(r) if r == "lost"));

        assert!(q.drain().is_empty());
    }

    #[test]
    fn ws_event_queue_clone_shares_state() {
        let q1 = WsEventQueue::new();
        let q2 = q1.clone();

        q1.push(WsIncomingEvent::Connected);
        assert_eq!(q2.drain().len(), 1);
    }

    #[test]
    fn ws_subscriptions_add_and_lookup() {
        let mut subs = WsSubscriptions::default();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let cmd1 = subs.add(id1);
        let cmd2 = subs.add(id2);

        assert_ne!(cmd1, cmd2);
        assert_eq!(subs.device_for_cmd(cmd1), Some(id1));
        assert_eq!(subs.device_for_cmd(cmd2), Some(id2));
        assert_eq!(subs.device_for_cmd(999), None);
    }

    #[test]
    fn reconnecting_event_has_correct_fields() {
        let ev = WsIncomingEvent::Reconnecting { attempt: 3, delay_ms: 4_000 };
        if let WsIncomingEvent::Reconnecting { attempt, delay_ms } = ev {
            assert_eq!(attempt, 3);
            assert_eq!(delay_ms, 4_000);
        }
    }

    #[test]
    fn ws_config_from_twin_config() {
        use crate::config::TwinConfig;
        let twin = TwinConfig::default();
        let ws_cfg = WsConfig::from_twin_config(&twin);
        assert_eq!(ws_cfg.server_url, "ws://localhost:8080");
        assert_eq!(ws_cfg.initial_delay_ms, 1_000);
        assert_eq!(ws_cfg.max_delay_ms, 60_000);
    }
}
