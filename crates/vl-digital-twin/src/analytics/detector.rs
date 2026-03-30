//! Anomaly detector resource — Z-score + linear regression per device/key.

use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

use super::stats::{linear_regression, mean_std, z_score, Ewma};

// ── Config ────────────────────────────────────────────────────────────────────

/// Configuration for anomaly detection thresholds and window size.
#[derive(Debug, Clone)]
pub struct AnomalyConfig {
    /// Number of recent samples to keep for rolling statistics.
    pub window_size:     usize,
    /// |z-score| threshold to flag a statistical outlier (default: 3.0).
    pub z_threshold:     f32,
    /// |slope| threshold (units/second) to flag rapid change (default: 5.0).
    pub slope_threshold: f32,
    /// Minimum samples before any anomaly is reported.
    pub min_samples:     usize,
    /// Deduplication window — suppress repeated anomalies for the same key (ms).
    pub dedup_ms:        u64,
    pub enabled:         bool,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            window_size:     60,
            z_threshold:     3.0,
            slope_threshold: 5.0,
            min_samples:     10,
            dedup_ms:        60_000,
            enabled:         true,
        }
    }
}

// ── Result types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum AnomalyKind {
    StatisticalOutlier,
    RapidIncrease,
    RapidDecrease,
}

impl std::fmt::Display for AnomalyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnomalyKind::StatisticalOutlier => write!(f, "Outlier"),
            AnomalyKind::RapidIncrease      => write!(f, "RisingFast"),
            AnomalyKind::RapidDecrease      => write!(f, "FallingFast"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnomalyResult {
    pub device_id:    Uuid,
    pub key:          String,
    pub value:        f32,
    pub timestamp_ms: u64,
    pub kind:         AnomalyKind,
    pub z_score:      Option<f32>,
    pub description:  String,
}

// ── Internal state ────────────────────────────────────────────────────────────

struct KeyState {
    window:         VecDeque<(u64, f32)>,
    ewma:           Ewma,
    last_anomaly_ms: u64,
}

impl KeyState {
    fn new(window_size: usize) -> Self {
        Self {
            window:          VecDeque::with_capacity(window_size),
            ewma:            Ewma::new(0.1),
            last_anomaly_ms: 0,
        }
    }
}

// ── Resource ──────────────────────────────────────────────────────────────────

/// Per-device/key rolling anomaly detector.
#[derive(bevy::prelude::Resource)]
pub struct AnomalyDetector {
    pub config: AnomalyConfig,
    state:      HashMap<(Uuid, String), KeyState>,
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self { config: AnomalyConfig::default(), state: HashMap::new() }
    }
}

impl AnomalyDetector {
    /// Feed a new telemetry data point.
    ///
    /// Returns any anomalies detected (may be empty). Each anomaly is deduplicated
    /// against `config.dedup_ms` to prevent flooding.
    pub fn feed(
        &mut self,
        device_id:    Uuid,
        key:          &str,
        value:        f32,
        timestamp_ms: u64,
    ) -> Vec<AnomalyResult> {
        if !self.config.enabled { return vec![]; }

        let cfg = self.config.clone();
        let st  = self.state
            .entry((device_id, key.to_string()))
            .or_insert_with(|| KeyState::new(cfg.window_size));

        st.ewma.update(value);
        st.window.push_back((timestamp_ms, value));
        if st.window.len() > cfg.window_size {
            st.window.pop_front();
        }

        if st.window.len() < cfg.min_samples { return vec![]; }

        // Dedup: skip if we already raised an anomaly for this key recently
        if timestamp_ms.saturating_sub(st.last_anomaly_ms) < cfg.dedup_ms {
            return vec![];
        }

        let mut anomalies = Vec::new();
        let values: Vec<f32> = st.window.iter().map(|(_, v)| *v).collect();
        let (mean, std)      = mean_std(&values);
        let z                = z_score(value, mean, std);

        // ── Z-score outlier ───────────────────────────────────────────────
        if z.abs() >= cfg.z_threshold {
            anomalies.push(AnomalyResult {
                device_id,
                key:          key.to_string(),
                value,
                timestamp_ms,
                kind:         AnomalyKind::StatisticalOutlier,
                z_score:      Some(z),
                description:  format!(
                    "{key} = {value:.2} (z={z:.1}, mean={mean:.1}, σ={std:.1})"
                ),
            });
        }

        // ── Slope check ───────────────────────────────────────────────────
        let points: Vec<(f32, f32)> = st.window.iter()
            .map(|(ts, v)| (*ts as f32 / 1000.0, *v))
            .collect();
        if let Some((slope, _)) = linear_regression(&points) {
            if slope >= cfg.slope_threshold {
                anomalies.push(AnomalyResult {
                    device_id,
                    key:          key.to_string(),
                    value,
                    timestamp_ms,
                    kind:         AnomalyKind::RapidIncrease,
                    z_score:      None,
                    description:  format!("{key} rising fast: +{slope:.2}/s"),
                });
            } else if slope <= -cfg.slope_threshold {
                anomalies.push(AnomalyResult {
                    device_id,
                    key:          key.to_string(),
                    value,
                    timestamp_ms,
                    kind:         AnomalyKind::RapidDecrease,
                    z_score:      None,
                    description:  format!("{key} dropping fast: {slope:.2}/s"),
                });
            }
        }

        if !anomalies.is_empty() {
            st.last_anomaly_ms = timestamp_ms;
        }

        anomalies
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_detector() -> AnomalyDetector {
        let mut d = AnomalyDetector::default();
        d.config.min_samples = 5;
        d.config.dedup_ms    = 0; // disable dedup in tests
        d
    }

    #[test]
    fn no_anomaly_stable_signal() {
        let mut d = make_detector();
        let id    = Uuid::nil();
        // Feed 20 identical values — no anomaly expected
        for i in 0..20 {
            let r = d.feed(id, "temp", 25.0, i * 1000);
            assert!(r.is_empty(), "stable signal should not flag anomaly");
        }
    }

    #[test]
    fn outlier_detected() {
        let mut d = make_detector();
        d.config.z_threshold = 2.0;
        let id = Uuid::nil();
        // Build baseline
        for i in 0..20u64 {
            d.feed(id, "temp", 20.0, i * 1000);
        }
        // Spike
        let results = d.feed(id, "temp", 200.0, 20_000);
        assert!(results.iter().any(|r| r.kind == AnomalyKind::StatisticalOutlier));
    }

    #[test]
    fn rapid_increase_detected() {
        let mut d = make_detector();
        d.config.slope_threshold = 0.5;
        d.config.z_threshold     = 100.0; // disable z-score
        let id = Uuid::nil();
        // Steadily increasing values: +2 per second
        for i in 0..20u64 {
            d.feed(id, "temp", i as f32 * 2.0, i * 1000);
        }
        let results = d.feed(id, "temp", 42.0, 21_000);
        assert!(results.iter().any(|r| r.kind == AnomalyKind::RapidIncrease));
    }

    // ── Dedup window ─────────────────────────────────────────────────────────

    #[test]
    fn dedup_suppresses_repeated_anomaly_within_window() {
        let mut d = make_detector();
        d.config.z_threshold = 2.0;
        d.config.dedup_ms    = 60_000; // 60 s dedup window
        let id = Uuid::nil();

        // Build baseline starting well past epoch 0 so the initial dedup
        // window (0 … dedup_ms) does not block the first spike.
        // Baseline: t = 200_000 ms … 219_000 ms (200 s to 219 s)
        for i in 0..20u64 {
            d.feed(id, "temp", 20.0, 200_000 + i * 1000);
        }
        // First spike at t = 220_000 — 220 s from epoch, far past initial dedup
        let r1 = d.feed(id, "temp", 200.0, 220_000);
        assert!(!r1.is_empty(), "first anomaly should be reported");

        // Second spike at t = 250_000 — only 30 s after first, within 60 s window
        let r2 = d.feed(id, "temp", 200.0, 250_000);
        assert!(r2.is_empty(), "second anomaly within dedup window should be suppressed");

        // Third spike at t = 282_000 — 62 s after first, past dedup window
        let r3 = d.feed(id, "temp", 200.0, 282_000);
        assert!(!r3.is_empty(), "anomaly after dedup window should be reported again");
    }

    // ── Below min_samples threshold ───────────────────────────────────────────

    #[test]
    fn no_anomaly_before_min_samples() {
        let mut d = make_detector();
        d.config.min_samples = 10;
        d.config.z_threshold = 0.01; // very sensitive
        let id = Uuid::nil();
        // Feed 9 samples (less than min_samples)
        for i in 0..9u64 {
            let r = d.feed(id, "temp", if i == 8 { 1000.0 } else { 20.0 }, i * 1000);
            assert!(r.is_empty(), "should not flag before min_samples reached: i={i}");
        }
    }

    // ── Disabled detector ─────────────────────────────────────────────────────

    #[test]
    fn disabled_detector_returns_nothing() {
        let mut d = make_detector();
        d.config.enabled     = false;
        d.config.z_threshold = 0.001; // would trigger if enabled
        let id = Uuid::nil();
        for i in 0..30u64 {
            let r = d.feed(id, "temp", if i % 2 == 0 { 1000.0 } else { 0.0 }, i * 1000);
            assert!(r.is_empty(), "disabled detector should never return anomalies");
        }
    }

    // ── Multiple devices don't interfere ─────────────────────────────────────

    #[test]
    fn multiple_devices_tracked_independently() {
        let mut d = make_detector();
        d.config.z_threshold = 2.0;
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        // Device 1: stable signal
        for i in 0..20u64 {
            d.feed(id1, "temp", 20.0, i * 1000);
        }
        // Device 2: stable signal
        for i in 0..20u64 {
            d.feed(id2, "temp", 20.0, i * 1000);
        }
        // Device 1 spikes — should only affect device 1
        let r1 = d.feed(id1, "temp", 200.0, 20_000);
        assert!(!r1.is_empty(), "device 1 spike should be detected");
        assert_eq!(r1[0].device_id, id1, "anomaly should be attributed to device 1");

        // Device 2 remains stable — no anomaly
        let r2 = d.feed(id2, "temp", 20.0, 20_000);
        assert!(r2.is_empty(), "device 2 should remain clean");
    }

    // ── Rapid decrease ────────────────────────────────────────────────────────

    #[test]
    fn rapid_decrease_detected() {
        let mut d = make_detector();
        d.config.slope_threshold = 0.5;
        d.config.z_threshold     = 100.0; // disable z-score check
        let id = Uuid::nil();
        // Steadily decreasing: -2 per second
        for i in 0..20u64 {
            d.feed(id, "temp", 40.0 - i as f32 * 2.0, i * 1000);
        }
        let results = d.feed(id, "temp", 0.0, 21_000);
        assert!(results.iter().any(|r| r.kind == AnomalyKind::RapidDecrease));
    }

    // ── Anomaly result fields ─────────────────────────────────────────────────

    #[test]
    fn anomaly_result_has_correct_device_and_key() {
        let mut d = make_detector();
        d.config.z_threshold = 2.0;
        let id = Uuid::new_v4();
        for i in 0..20u64 {
            d.feed(id, "humidity", 50.0, i * 1000);
        }
        let results = d.feed(id, "humidity", 500.0, 20_000);
        assert!(!results.is_empty());
        assert_eq!(results[0].device_id, id);
        assert_eq!(results[0].key, "humidity");
        assert_eq!(results[0].value, 500.0);
    }

    #[test]
    fn anomaly_kind_display() {
        assert_eq!(AnomalyKind::StatisticalOutlier.to_string(), "Outlier");
        assert_eq!(AnomalyKind::RapidIncrease.to_string(),      "RisingFast");
        assert_eq!(AnomalyKind::RapidDecrease.to_string(),      "FallingFast");
    }
}
