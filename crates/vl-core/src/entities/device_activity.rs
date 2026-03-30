use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Trạng thái hoạt động của device — khớp Java DeviceStateService
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceActivity {
    pub device_id:          Uuid,
    pub last_connect_ts:    i64,
    pub last_disconnect_ts: i64,
    pub last_activity_ts:   i64,
    pub last_telemetry_ts:  i64,
    pub last_rpc_ts:        i64,
    pub active:             bool,
}

impl DeviceActivity {
    pub fn new(device_id: Uuid) -> Self {
        Self {
            device_id,
            last_connect_ts:    0,
            last_disconnect_ts: 0,
            last_activity_ts:   0,
            last_telemetry_ts:  0,
            last_rpc_ts:        0,
            active:             false,
        }
    }
}

/// Event gửi từ transport layer → ActivityService qua mpsc channel
#[derive(Debug, Clone)]
pub enum ActivityEvent {
    Connected    { device_id: Uuid, ts: i64 },
    Disconnected { device_id: Uuid, ts: i64 },
    Telemetry    { device_id: Uuid, ts: i64 },
    Rpc          { device_id: Uuid, ts: i64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_new_defaults_zero() {
        let id = Uuid::new_v4();
        let a = DeviceActivity::new(id);
        assert_eq!(a.device_id, id);
        assert_eq!(a.last_connect_ts, 0);
        assert_eq!(a.last_disconnect_ts, 0);
        assert_eq!(a.last_activity_ts, 0);
        assert_eq!(a.last_telemetry_ts, 0);
        assert_eq!(a.last_rpc_ts, 0);
        assert!(!a.active);
    }

    #[test]
    fn test_activity_active_state() {
        let mut a = DeviceActivity::new(Uuid::new_v4());
        a.active = true;
        a.last_connect_ts = 1_000_000;
        assert!(a.active);
        assert_eq!(a.last_connect_ts, 1_000_000);
    }
}
