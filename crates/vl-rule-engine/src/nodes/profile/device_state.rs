use serde::{Deserialize, Serialize};

/// Device connectivity and activity state machine.
/// Mirrors ThingsBoard DeviceState / DeviceStateService logic.
///
/// States:
/// - ACTIVE: recently received telemetry (within inactivity timeout)
/// - INACTIVE: no telemetry for longer than inactivity timeout
/// - CONNECTED: transport session open (MQTT/CoAP)
/// - DISCONNECTED: transport session closed
///
/// Active/Inactive is driven by telemetry timestamps.
/// Connected/Disconnected is driven by transport events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityState {
    Active,
    Inactive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectivityState {
    Connected,
    Disconnected,
}

/// Full device state snapshot — both dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceState {
    pub activity:     ActivityState,
    pub connectivity: ConnectivityState,
    /// Timestamp (ms) of the last received message
    pub last_activity_time: i64,
    /// Timestamp (ms) when current state was entered
    pub last_connect_time:  i64,
    pub last_disconnect_time: i64,
}

impl Default for DeviceState {
    fn default() -> Self {
        Self {
            activity:             ActivityState::Inactive,
            connectivity:         ConnectivityState::Disconnected,
            last_activity_time:   0,
            last_connect_time:    0,
            last_disconnect_time: 0,
        }
    }
}

/// Events that drive device state transitions
#[derive(Debug, Clone)]
pub enum DeviceStateEvent {
    /// Message received from device (telemetry/attribute update)
    MessageReceived { ts: i64 },
    /// Transport session opened
    Connected { ts: i64 },
    /// Transport session closed
    Disconnected { ts: i64 },
    /// Inactivity timeout fired (no message for `inactivity_timeout_ms`)
    InactivityTimeout { ts: i64 },
}

impl DeviceState {
    /// Apply an event and return true if the state changed.
    /// Java: DeviceStateService.onDeviceEvent() / checkInactivity()
    pub fn apply(&mut self, event: DeviceStateEvent) -> bool {
        match event {
            DeviceStateEvent::MessageReceived { ts } => {
                self.last_activity_time = ts;
                if self.activity != ActivityState::Active {
                    self.activity = ActivityState::Active;
                    return true;
                }
                false
            }

            DeviceStateEvent::Connected { ts } => {
                self.last_connect_time = ts;
                if self.connectivity != ConnectivityState::Connected {
                    self.connectivity = ConnectivityState::Connected;
                    return true;
                }
                false
            }

            DeviceStateEvent::Disconnected { ts } => {
                self.last_disconnect_time = ts;
                if self.connectivity != ConnectivityState::Disconnected {
                    self.connectivity = ConnectivityState::Disconnected;
                    return true;
                }
                false
            }

            DeviceStateEvent::InactivityTimeout { ts: _ } => {
                if self.activity != ActivityState::Inactive {
                    self.activity = ActivityState::Inactive;
                    return true;
                }
                false
            }
        }
    }

    /// Check if the device has become inactive given the current time and timeout.
    /// Returns true if the state transitioned to Inactive.
    /// Java: DeviceStateService.checkInactivity()
    pub fn check_inactivity(&mut self, now_ms: i64, inactivity_timeout_ms: i64) -> bool {
        if self.activity == ActivityState::Active
            && (now_ms - self.last_activity_time) > inactivity_timeout_ms
        {
            self.activity = ActivityState::Inactive;
            return true;
        }
        false
    }

    /// Whether this device is both connected and active
    pub fn is_online(&self) -> bool {
        self.activity == ActivityState::Active
            && self.connectivity == ConnectivityState::Connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_inactive_disconnected() {
        let s = DeviceState::default();
        assert_eq!(s.activity, ActivityState::Inactive);
        assert_eq!(s.connectivity, ConnectivityState::Disconnected);
    }

    #[test]
    fn message_transitions_to_active() {
        let mut s = DeviceState::default();
        let changed = s.apply(DeviceStateEvent::MessageReceived { ts: 1000 });
        assert!(changed);
        assert_eq!(s.activity, ActivityState::Active);
        assert_eq!(s.last_activity_time, 1000);
    }

    #[test]
    fn duplicate_active_no_change() {
        let mut s = DeviceState::default();
        s.apply(DeviceStateEvent::MessageReceived { ts: 1000 });
        let changed = s.apply(DeviceStateEvent::MessageReceived { ts: 2000 });
        assert!(!changed); // already active
        assert_eq!(s.last_activity_time, 2000); // timestamp still updated
    }

    #[test]
    fn connected_disconnected_transitions() {
        let mut s = DeviceState::default();
        let c = s.apply(DeviceStateEvent::Connected { ts: 100 });
        assert!(c);
        assert_eq!(s.connectivity, ConnectivityState::Connected);

        let d = s.apply(DeviceStateEvent::Disconnected { ts: 200 });
        assert!(d);
        assert_eq!(s.connectivity, ConnectivityState::Disconnected);
        assert_eq!(s.last_disconnect_time, 200);
    }

    #[test]
    fn inactivity_timeout_transitions_to_inactive() {
        let mut s = DeviceState::default();
        s.apply(DeviceStateEvent::MessageReceived { ts: 1000 });
        assert_eq!(s.activity, ActivityState::Active);

        let changed = s.check_inactivity(11_001, 10_000);
        assert!(changed);
        assert_eq!(s.activity, ActivityState::Inactive);
    }

    #[test]
    fn inactivity_not_triggered_within_timeout() {
        let mut s = DeviceState::default();
        s.apply(DeviceStateEvent::MessageReceived { ts: 1000 });

        let changed = s.check_inactivity(5_000, 10_000);
        assert!(!changed);
        assert_eq!(s.activity, ActivityState::Active);
    }

    #[test]
    fn is_online_requires_both_active_and_connected() {
        let mut s = DeviceState::default();
        assert!(!s.is_online());

        s.apply(DeviceStateEvent::MessageReceived { ts: 1 });
        assert!(!s.is_online()); // active but not connected

        s.apply(DeviceStateEvent::Connected { ts: 2 });
        assert!(s.is_online()); // both active + connected
    }
}
