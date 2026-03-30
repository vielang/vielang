/// SNMP session tracking — per-device state across trap receptions.
///
/// ThingsBoard Java tracks device inactivity timeout and report_timeout via
/// a session management layer. This module provides equivalent tracking:
/// - Last-seen timestamp per source IP
/// - SNMP version detected from PDU
/// - Community string used (for audit)
/// - Uptime extracted from sysUpTime varbind (if present)

use std::net::IpAddr;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use tracing::debug;

/// State of a single SNMP agent session (one entry per source IP).
#[derive(Debug, Clone)]
pub struct SnmpSession {
    /// Source IP of the SNMP agent
    pub peer_addr:    IpAddr,
    /// SNMP version (1 = SNMPv1, 2 = SNMPv2c)
    pub version:      u8,
    /// Community string used by this agent (for audit logging)
    pub community:    String,
    /// Last time a trap was received from this agent
    pub last_seen:    Instant,
    /// Number of traps received in this session
    pub trap_count:   u64,
    /// sysUpTime value from the last trap (in centiseconds)
    pub sys_uptime_cs: Option<u64>,
}

impl SnmpSession {
    pub fn new(peer_addr: IpAddr, version: u8, community: &str) -> Self {
        Self {
            peer_addr,
            version,
            community:      community.to_string(),
            last_seen:      Instant::now(),
            trap_count:     1,
            sys_uptime_cs:  None,
        }
    }

    /// Update session on new trap from the same peer.
    pub fn touch(&mut self, uptime_cs: Option<u64>) {
        self.last_seen   = Instant::now();
        self.trap_count += 1;
        if let Some(u) = uptime_cs {
            self.sys_uptime_cs = Some(u);
        }
    }

    /// Seconds since last trap from this agent.
    pub fn idle_secs(&self) -> u64 {
        self.last_seen.elapsed().as_secs()
    }

    /// True if the session has been idle longer than `timeout`.
    pub fn is_expired(&self, timeout: Duration) -> bool {
        self.last_seen.elapsed() > timeout
    }
}

/// Concurrent registry of active SNMP sessions, keyed by source IP.
pub struct SessionRegistry {
    sessions:         DashMap<IpAddr, SnmpSession>,
    inactivity_timeout: Duration,
}

impl SessionRegistry {
    pub fn new(inactivity_timeout_secs: u64) -> Self {
        Self {
            sessions:           DashMap::new(),
            inactivity_timeout: Duration::from_secs(inactivity_timeout_secs),
        }
    }

    /// Record a trap received from `peer`. Returns true if this is a new session.
    pub fn record_trap(
        &self,
        peer:       IpAddr,
        version:    u8,
        community:  &str,
        uptime_cs:  Option<u64>,
    ) -> bool {
        if let Some(mut session) = self.sessions.get_mut(&peer) {
            session.touch(uptime_cs);
            false
        } else {
            let mut s = SnmpSession::new(peer, version, community);
            s.sys_uptime_cs = uptime_cs;
            self.sessions.insert(peer, s);
            debug!(peer = %peer, version, community, "New SNMP session registered");
            true
        }
    }

    /// Remove sessions that have been idle longer than the inactivity timeout.
    /// Returns the number of sessions removed.
    pub fn evict_expired(&self) -> usize {
        let before = self.sessions.len();
        self.sessions.retain(|_, s| !s.is_expired(self.inactivity_timeout));
        before - self.sessions.len()
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

/// Extract sysUpTime (OID 1.3.6.1.2.1.1.3.0) from a varbind list.
pub fn extract_uptime(varbinds: &[(String, serde_json::Value)]) -> Option<u64> {
    const SYS_UPTIME_OID: &str = "1.3.6.1.2.1.1.3.0";
    varbinds
        .iter()
        .find(|(oid, _)| oid == SYS_UPTIME_OID)
        .and_then(|(_, v)| v.as_u64())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn ip(a: u8, b: u8, c: u8, d: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(a, b, c, d))
    }

    #[test]
    fn test_new_session() {
        let reg = SessionRegistry::new(600);
        let is_new = reg.record_trap(ip(192, 168, 1, 10), 2, "public", None);
        assert!(is_new);
        assert_eq!(reg.session_count(), 1);
    }

    #[test]
    fn test_existing_session() {
        let reg = SessionRegistry::new(600);
        reg.record_trap(ip(10, 0, 0, 1), 2, "public", None);
        let is_new = reg.record_trap(ip(10, 0, 0, 1), 2, "public", Some(12345));
        assert!(!is_new);
        assert_eq!(reg.session_count(), 1);
    }

    #[test]
    fn test_extract_uptime() {
        let varbinds = vec![
            ("1.3.6.1.2.1.1.3.0".to_string(), serde_json::json!(98765u64)),
            ("1.3.6.1.2.1.1.5.0".to_string(), serde_json::json!("router1")),
        ];
        assert_eq!(extract_uptime(&varbinds), Some(98765));
    }

    #[test]
    fn test_extract_uptime_missing() {
        let varbinds = vec![
            ("1.3.6.1.2.1.1.5.0".to_string(), serde_json::json!("router1")),
        ];
        assert_eq!(extract_uptime(&varbinds), None);
    }
}
