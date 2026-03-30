use std::collections::HashMap;

use axum::extract::ws::Message;
use dashmap::DashMap;
use serde_json::Value;
use tokio::sync::mpsc;
use uuid::Uuid;

use super::subscription::WsUpdate;

/// Info stored per active WS session
pub struct WsSessionInfo {
    pub user_id:   Uuid,
    pub tenant_id: Option<Uuid>,
    pub tx:        mpsc::Sender<Message>,
}

/// Global WebSocket session registry.
///
/// Provides entity-indexed subscription tracking so the transport layer
/// (MQTT/HTTP) can push realtime updates only to sessions that subscribed
/// to a given entity — O(subscriptions) instead of O(all_sessions).
pub struct WsSessionRegistry {
    /// session_id → session info
    sessions: DashMap<Uuid, WsSessionInfo>,
    /// entity_id → Vec<(session_id, cmd_id)>
    entity_subs: DashMap<Uuid, Vec<(Uuid, i32)>>,
}

impl WsSessionRegistry {
    pub fn new() -> Self {
        Self {
            sessions:    DashMap::new(),
            entity_subs: DashMap::new(),
        }
    }

    /// Register a new WS session.
    pub fn register(
        &self,
        session_id: Uuid,
        user_id:    Uuid,
        tenant_id:  Option<Uuid>,
        tx:         mpsc::Sender<Message>,
    ) {
        self.sessions.insert(session_id, WsSessionInfo { user_id, tenant_id, tx });
    }

    /// Remove session and all its subscriptions on disconnect.
    pub fn remove_session(&self, session_id: Uuid) {
        self.sessions.remove(&session_id);
        self.entity_subs.retain(|_, subs: &mut Vec<(Uuid, i32)>| {
            subs.retain(|(sid, _)| *sid != session_id);
            !subs.is_empty()
        });
    }

    /// Record that session_id is now subscribing to entity_id with the given cmd_id.
    pub fn add_subscription(&self, session_id: Uuid, entity_id: Uuid, cmd_id: i32) {
        self.entity_subs
            .entry(entity_id)
            .or_default()
            .push((session_id, cmd_id));
    }

    /// Remove a specific subscription by cmd_id from a session.
    pub fn remove_subscription(&self, session_id: Uuid, cmd_id: i32) {
        self.entity_subs.retain(|_, subs: &mut Vec<(Uuid, i32)>| {
            subs.retain(|(sid, cid)| !(*sid == session_id && *cid == cmd_id));
            !subs.is_empty()
        });
    }

    /// Get the send channel for a session (clone to avoid holding Ref across await).
    pub fn get_tx(&self, session_id: Uuid) -> Option<mpsc::Sender<Message>> {
        self.sessions.get(&session_id).map(|s| s.tx.clone())
    }

    /// Push a telemetry update to all sessions currently subscribed to entity_id.
    ///
    /// Called from the transport layer (REST or MQTT) after saving telemetry.
    /// Uses `try_send` to be non-blocking — slow consumers silently drop updates
    /// and will receive the next broadcast instead.
    pub fn push_ts_update(
        &self,
        entity_id: Uuid,
        data: HashMap<String, Vec<[Value; 2]>>,
    ) {
        let subs: Vec<(Uuid, i32)> = self.entity_subs
            .get(&entity_id)
            .map(|s| s.value().clone())  // collect before dropping Ref
            .unwrap_or_default();

        if subs.is_empty() {
            return;
        }

        for (session_id, cmd_id) in subs {
            if let Some(tx) = self.sessions.get(&session_id).map(|s| s.tx.clone()) {
                let update = WsUpdate::new(cmd_id, data.clone());
                if let Ok(json) = serde_json::to_string(&update) {
                    let _ = tx.try_send(Message::Text(json.into()));
                }
            }
        }
    }
}

impl Default for WsSessionRegistry {
    fn default() -> Self { Self::new() }
}
