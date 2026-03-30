/// LwM2M device registration lifecycle management.
///
/// Wraps the raw session map with lifetime-aware operations:
/// - Expiry detection (session expires after `lifetime` seconds without update)
/// - Periodic eviction of expired sessions
/// - Session lookup by endpoint name (in addition to registration token)
///
/// Mirrors ThingsBoard's session inactivity timeout and report_timeout logic
/// from `LwM2MTransportServerConfig.java` (inactivity_timeout, report_timeout).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, RwLock};
use tokio::time;
use tracing::{debug, info, warn};
use uuid::Uuid;

use vl_core::entities::ActivityEvent;

use super::handler::Lwm2mSession;

/// Extended session with expiry tracking.
#[derive(Debug, Clone)]
pub struct ManagedSession {
    pub session:    Lwm2mSession,
    /// Unix timestamp (ms) when this session expires if not renewed.
    pub expires_at: i64,
}

impl ManagedSession {
    pub fn new(session: Lwm2mSession) -> Self {
        let expires_at = Self::compute_expiry(session.registered_at, session.lifetime);
        Self { session, expires_at }
    }

    fn compute_expiry(registered_at: i64, lifetime_secs: u32) -> i64 {
        registered_at + (lifetime_secs as i64 * 1000)
    }

    /// Renew the session lifetime on a Registration Update.
    pub fn renew(&mut self, new_lifetime: Option<u32>) {
        let now = chrono::Utc::now().timestamp_millis();
        if let Some(lt) = new_lifetime {
            self.session.lifetime = lt;
        }
        self.expires_at = Self::compute_expiry(now, self.session.lifetime);
    }

    /// True if the session has exceeded its lifetime.
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp_millis();
        now > self.expires_at
    }

    /// Seconds remaining before this session expires (0 if already expired).
    pub fn ttl_secs(&self) -> i64 {
        let now = chrono::Utc::now().timestamp_millis();
        ((self.expires_at - now) / 1000).max(0)
    }
}

/// Thread-safe registration store with lifetime management.
pub struct RegistrationStore {
    /// token → managed session
    by_token:    Arc<RwLock<HashMap<String, ManagedSession>>>,
    /// endpoint → token (for reverse lookup)
    by_endpoint: Arc<RwLock<HashMap<String, String>>>,
}

impl RegistrationStore {
    pub fn new() -> Self {
        Self {
            by_token:    Arc::new(RwLock::new(HashMap::new())),
            by_endpoint: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new device session.
    pub async fn register(&self, token: String, session: Lwm2mSession) {
        let endpoint = session.endpoint.clone();
        let managed  = ManagedSession::new(session);
        self.by_token.write().await.insert(token.clone(), managed);
        self.by_endpoint.write().await.insert(endpoint, token);
    }

    /// Update an existing registration's lifetime and object links.
    pub async fn update(&self, token: &str, new_lifetime: Option<u32>) -> bool {
        let mut store = self.by_token.write().await;
        if let Some(managed) = store.get_mut(token) {
            managed.renew(new_lifetime);
            true
        } else {
            false
        }
    }

    /// Remove a session by token. Returns the removed session if found.
    pub async fn deregister(&self, token: &str) -> Option<ManagedSession> {
        let removed = self.by_token.write().await.remove(token);
        if let Some(ref m) = removed {
            self.by_endpoint.write().await.remove(&m.session.endpoint);
        }
        removed
    }

    /// Look up a session by registration token.
    pub async fn get_by_token(&self, token: &str) -> Option<ManagedSession> {
        self.by_token.read().await.get(token).cloned()
    }

    /// Look up a session by endpoint name.
    pub async fn get_by_endpoint(&self, endpoint: &str) -> Option<ManagedSession> {
        let token = self.by_endpoint.read().await.get(endpoint).cloned()?;
        self.by_token.read().await.get(&token).cloned()
    }

    /// Returns the number of active (non-expired) sessions.
    pub async fn active_count(&self) -> usize {
        self.by_token.read().await.values().filter(|s| !s.is_expired()).count()
    }

    /// Evict all expired sessions. Returns the device IDs of expired sessions.
    pub async fn evict_expired(&self) -> Vec<Uuid> {
        let mut expired_tokens: Vec<String> = Vec::new();
        {
            let store = self.by_token.read().await;
            for (token, managed) in store.iter() {
                if managed.is_expired() {
                    expired_tokens.push(token.clone());
                }
            }
        }

        let mut expired_ids = Vec::new();
        let mut token_store   = self.by_token.write().await;
        let mut endpoint_store = self.by_endpoint.write().await;
        for token in expired_tokens {
            if let Some(managed) = token_store.remove(&token) {
                endpoint_store.remove(&managed.session.endpoint);
                expired_ids.push(managed.session.device_id);
                warn!(
                    device_id = %managed.session.device_id,
                    endpoint  = %managed.session.endpoint,
                    "LwM2M session expired — device did not renew registration"
                );
            }
        }
        expired_ids
    }
}

impl Default for RegistrationStore {
    fn default() -> Self { Self::new() }
}

/// Spawn a background task that periodically evicts expired sessions and
/// sends `Disconnected` activity events for each expired device.
pub fn spawn_expiry_task(
    store:       Arc<RegistrationStore>,
    activity_tx: mpsc::Sender<ActivityEvent>,
    interval:    Duration,
) {
    tokio::spawn(async move {
        let mut ticker = time::interval(interval);
        loop {
            ticker.tick().await;
            let expired = store.evict_expired().await;
            if !expired.is_empty() {
                info!(count = expired.len(), "LwM2M: evicted expired sessions");
                let ts = chrono::Utc::now().timestamp_millis();
                for device_id in expired {
                    let _ = activity_tx
                        .send(ActivityEvent::Disconnected { device_id, ts })
                        .await;
                    debug!(device_id = %device_id, "LwM2M: session expired → Disconnected");
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lwm2m::handler::Lwm2mSession;

    fn make_session(endpoint: &str, lifetime: u32) -> Lwm2mSession {
        Lwm2mSession {
            device_id:     Uuid::new_v4(),
            endpoint:       endpoint.to_string(),
            lifetime,
            objects:        Vec::new(),
            registered_at: chrono::Utc::now().timestamp_millis(),
        }
    }

    #[tokio::test]
    async fn test_register_and_lookup() {
        let store = RegistrationStore::new();
        let sess  = make_session("sensor-01", 300);
        let id    = sess.device_id;
        store.register("token-abc".to_string(), sess).await;

        let found = store.get_by_token("token-abc").await.unwrap();
        assert_eq!(found.session.device_id, id);

        let by_ep = store.get_by_endpoint("sensor-01").await.unwrap();
        assert_eq!(by_ep.session.device_id, id);
    }

    #[tokio::test]
    async fn test_deregister() {
        let store = RegistrationStore::new();
        store.register("tok".to_string(), make_session("dev-x", 300)).await;
        let removed = store.deregister("tok").await;
        assert!(removed.is_some());
        assert!(store.get_by_token("tok").await.is_none());
    }

    #[tokio::test]
    async fn test_update_lifetime() {
        let store = RegistrationStore::new();
        store.register("tok2".to_string(), make_session("dev-y", 60)).await;
        let updated = store.update("tok2", Some(3600)).await;
        assert!(updated);
        let m = store.get_by_token("tok2").await.unwrap();
        assert_eq!(m.session.lifetime, 3600);
    }

    #[test]
    fn test_ttl_not_expired() {
        let sess = make_session("ep", 3600);
        let managed = ManagedSession::new(sess);
        assert!(!managed.is_expired());
        assert!(managed.ttl_secs() > 3500);
    }
}
