//! DeviceKeyRegistry — auto-discover all telemetry keys seen per device from live WS updates.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use uuid::Uuid;

use crate::events::TelemetryUpdate;

// ── Resource ──────────────────────────────────────────────────────────────────

/// Tracks all telemetry keys observed per device from live WS updates.
/// Populated automatically by `observe_telemetry_keys` each frame.
#[derive(Resource, Default)]
pub struct DeviceKeyRegistry {
    keys: HashMap<Uuid, HashSet<String>>,
}

impl DeviceKeyRegistry {
    /// Register a key as seen for the given device.
    pub fn observe(&mut self, device_id: Uuid, key: impl Into<String>) {
        self.keys.entry(device_id).or_default().insert(key.into());
    }

    /// All known keys for a device, sorted alphabetically.
    pub fn keys_for(&self, device_id: Uuid) -> Vec<String> {
        let mut v: Vec<String> = self
            .keys
            .get(&device_id)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default();
        v.sort();
        v
    }

    /// All keys observed across ALL devices — union set, sorted.
    pub fn all_keys(&self) -> Vec<String> {
        let mut set: HashSet<String> = HashSet::new();
        for keys in self.keys.values() {
            set.extend(keys.iter().cloned());
        }
        let mut v: Vec<String> = set.into_iter().collect();
        v.sort();
        v
    }
}

// ── System ────────────────────────────────────────────────────────────────────

/// Read TelemetryUpdate events each frame and populate DeviceKeyRegistry.
pub fn observe_telemetry_keys(
    mut events:   MessageReader<TelemetryUpdate>,
    mut registry: ResMut<DeviceKeyRegistry>,
) {
    for ev in events.read() {
        registry.observe(ev.device_id, ev.key.clone());
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_observe_and_query() {
        let mut r  = DeviceKeyRegistry::default();
        let id     = Uuid::new_v4();
        r.observe(id, "temperature");
        r.observe(id, "humidity");
        r.observe(id, "temperature"); // duplicate — should not appear twice
        let keys = r.keys_for(id);
        assert_eq!(keys, vec!["humidity", "temperature"]);
    }

    #[test]
    fn registry_all_keys_merged() {
        let mut r = DeviceKeyRegistry::default();
        r.observe(Uuid::new_v4(), "temperature");
        r.observe(Uuid::new_v4(), "vibration");
        let all = r.all_keys();
        assert!(all.contains(&"temperature".to_string()));
        assert!(all.contains(&"vibration".to_string()));
    }

    #[test]
    fn registry_empty_device_returns_empty() {
        let r  = DeviceKeyRegistry::default();
        let id = Uuid::new_v4();
        assert!(r.keys_for(id).is_empty());
    }

    #[test]
    fn registry_keys_for_are_sorted_alphabetically() {
        let mut r  = DeviceKeyRegistry::default();
        let id     = Uuid::new_v4();
        r.observe(id, "vibration");
        r.observe(id, "temperature");
        r.observe(id, "humidity");
        let keys = r.keys_for(id);
        assert_eq!(keys, vec!["humidity", "temperature", "vibration"]);
    }

    #[test]
    fn registry_all_keys_deduplicates_across_devices() {
        let mut r  = DeviceKeyRegistry::default();
        let id1    = Uuid::new_v4();
        let id2    = Uuid::new_v4();
        r.observe(id1, "temperature");
        r.observe(id2, "temperature"); // same key on different device
        r.observe(id2, "rpm");
        let all = r.all_keys();
        // temperature appears in both devices but should only be listed once
        assert_eq!(all.iter().filter(|k| k.as_str() == "temperature").count(), 1);
        assert!(all.contains(&"rpm".to_string()));
    }

    #[test]
    fn registry_observe_same_key_multiple_times_is_idempotent() {
        let mut r  = DeviceKeyRegistry::default();
        let id     = Uuid::new_v4();
        for _ in 0..100 {
            r.observe(id, "temperature");
        }
        assert_eq!(r.keys_for(id).len(), 1, "observing same key repeatedly should not duplicate");
    }
}
