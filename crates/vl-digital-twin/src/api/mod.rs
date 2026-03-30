pub mod client;

pub use client::{AggregationType, ApiClient, ApiConfig, DataPoint, DeviceListCache};

// ── Historical data cache ─────────────────────────────────────────────────────

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use uuid::Uuid;

use crate::events::RpcResult;
use crate::playback::TimeRange;

/// In-memory cache for historical timeseries data.
///
/// Uses BTreeMap<timestamp_ms, value> so range queries are O(log n).
#[derive(Resource, Default)]
pub struct HistoricalDataCache {
    /// device_id → key → timestamp_ms → value
    entries:        HashMap<Uuid, HashMap<String, BTreeMap<i64, f64>>>,
    /// Ranges already fetched per device — avoid redundant queries.
    fetched_ranges: HashMap<Uuid, Vec<(i64, i64)>>,
}

impl HistoricalDataCache {
    /// Store data points into the cache.
    pub fn insert(&mut self, device_id: Uuid, key: &str, points: &[DataPoint]) {
        let dev   = self.entries.entry(device_id).or_default();
        let store = dev.entry(key.to_string()).or_default();
        for pt in points {
            store.insert(pt.ts, pt.value);
        }
    }

    /// Get the value at (or just before) `ts` — floor interpolation.
    pub fn get_at(&self, device_id: Uuid, key: &str, ts: i64) -> Option<f64> {
        self.entries.get(&device_id)?
            .get(key)?
            .range(..=ts)
            .next_back()
            .map(|(_, &v)| v)
    }

    /// Get all data points in [start, end].
    pub fn get_range(&self, device_id: Uuid, key: &str, start: i64, end: i64) -> Vec<DataPoint> {
        let Some(dev) = self.entries.get(&device_id) else { return vec![] };
        let Some(store) = dev.get(key) else { return vec![] };
        store.range(start..=end)
            .map(|(&ts, &value)| DataPoint { ts, value })
            .collect()
    }

    /// True if the given range is already fully covered in the cache.
    pub fn is_fetched(&self, device_id: Uuid, range: &TimeRange) -> bool {
        self.fetched_ranges
            .get(&device_id)
            .map(|rs| rs.iter().any(|&(s, e)| s <= range.start && e >= range.end))
            .unwrap_or(false)
    }

    /// Mark a range as fetched.
    pub fn mark_fetched(&mut self, device_id: Uuid, range: &TimeRange) {
        self.fetched_ranges
            .entry(device_id)
            .or_default()
            .push((range.start, range.end));
    }

    /// Drop data points older than `threshold_ms`.
    pub fn evict_older_than(&mut self, threshold_ms: i64) {
        for dev in self.entries.values_mut() {
            for store in dev.values_mut() {
                *store = store.split_off(&threshold_ms);
            }
        }
    }
}

/// Thread-safe queue for completed RPC call results — mirrors WsEventQueue pattern.
/// Background threads push results; Bevy drain system converts them to events each frame.
#[derive(Resource, Clone)]
pub struct RpcResponseQueue(pub Arc<Mutex<VecDeque<RpcResult>>>);

impl RpcResponseQueue {
    pub fn push(&self, resp: RpcResult) {
        if let Ok(mut q) = self.0.lock() {
            q.push_back(resp);
        }
    }

    pub fn drain(&self) -> Vec<RpcResult> {
        if let Ok(mut q) = self.0.lock() {
            q.drain(..).collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for RpcResponseQueue {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(VecDeque::new())))
    }
}
