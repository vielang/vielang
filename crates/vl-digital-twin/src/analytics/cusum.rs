//! CUSUM (Cumulative Sum) change-point detection.
//!
//! Detects shifts in the mean of a process. Used in Statistical Process Control
//! (SPC) for industrial quality monitoring.
//!
//! Reference: Page (1954) "Continuous Inspection Schemes"

use std::collections::VecDeque;
use uuid::Uuid;

/// Configuration for the CUSUM detector.
#[derive(Debug, Clone)]
pub struct CusumConfig {
    /// Target (expected) mean of the process.
    pub target: f64,
    /// Slack parameter — allowable deviation before accumulating (typically 0.5 * delta).
    pub slack: f64,
    /// Decision threshold — alarm when cumulative sum exceeds this.
    pub threshold: f64,
    /// Whether to detect both upward and downward shifts.
    pub two_sided: bool,
}

impl Default for CusumConfig {
    fn default() -> Self {
        Self {
            target: 0.0,
            slack: 0.5,
            threshold: 5.0,
            two_sided: true,
        }
    }
}

/// Result of a CUSUM detection.
#[derive(Debug, Clone)]
pub struct CusumAlert {
    pub device_id: Uuid,
    pub key: String,
    pub direction: ShiftDirection,
    /// The CUSUM statistic value when threshold was exceeded.
    pub cusum_value: f64,
    /// The observation that triggered the alert.
    pub trigger_value: f64,
    /// Index/timestamp of the trigger.
    pub timestamp_ms: u64,
    /// Estimated number of samples since the shift began.
    pub run_length: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShiftDirection {
    Upward,
    Downward,
}

impl std::fmt::Display for ShiftDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiftDirection::Upward => write!(f, "Upward"),
            ShiftDirection::Downward => write!(f, "Downward"),
        }
    }
}

/// Per-key CUSUM state.
struct CusumState {
    /// Upper CUSUM statistic (detects upward shift).
    s_high: f64,
    /// Lower CUSUM statistic (detects downward shift).
    s_low: f64,
    /// Number of consecutive samples since last reset.
    run_length: usize,
    /// Flag to prevent repeated alerts without reset.
    in_alarm: bool,
}

impl CusumState {
    fn new() -> Self {
        Self {
            s_high: 0.0,
            s_low: 0.0,
            run_length: 0,
            in_alarm: false,
        }
    }

    fn reset(&mut self) {
        self.s_high = 0.0;
        self.s_low = 0.0;
        self.run_length = 0;
        self.in_alarm = false;
    }
}

/// CUSUM change-point detector for multiple devices and keys.
pub struct CusumDetector {
    pub config: CusumConfig,
    state: std::collections::HashMap<(Uuid, String), CusumState>,
}

impl CusumDetector {
    pub fn new(config: CusumConfig) -> Self {
        Self {
            config,
            state: std::collections::HashMap::new(),
        }
    }

    /// Feed a new observation. Returns an alert if a change point is detected.
    pub fn feed(
        &mut self,
        device_id: Uuid,
        key: &str,
        value: f64,
        timestamp_ms: u64,
    ) -> Option<CusumAlert> {
        let cfg = &self.config;
        let state = self.state
            .entry((device_id, key.to_string()))
            .or_insert_with(CusumState::new);

        state.run_length += 1;

        // Update upper CUSUM: detects upward shift
        state.s_high = (state.s_high + value - cfg.target - cfg.slack).max(0.0);

        // Update lower CUSUM: detects downward shift
        state.s_low = (state.s_low - value + cfg.target - cfg.slack).max(0.0);

        if state.in_alarm {
            // Already in alarm — check if process returned to normal
            if state.s_high < cfg.threshold * 0.5 && state.s_low < cfg.threshold * 0.5 {
                state.reset();
            }
            return None;
        }

        // Check for threshold exceedance
        if state.s_high > cfg.threshold {
            state.in_alarm = true;
            return Some(CusumAlert {
                device_id,
                key: key.into(),
                direction: ShiftDirection::Upward,
                cusum_value: state.s_high,
                trigger_value: value,
                timestamp_ms,
                run_length: state.run_length,
            });
        }

        if cfg.two_sided && state.s_low > cfg.threshold {
            state.in_alarm = true;
            return Some(CusumAlert {
                device_id,
                key: key.into(),
                direction: ShiftDirection::Downward,
                cusum_value: state.s_low,
                trigger_value: value,
                timestamp_ms,
                run_length: state.run_length,
            });
        }

        None
    }

    /// Reset the CUSUM state for a specific device/key.
    pub fn reset(&mut self, device_id: Uuid, key: &str) {
        if let Some(state) = self.state.get_mut(&(device_id, key.to_string())) {
            state.reset();
        }
    }

    /// Get current CUSUM statistics for a device/key.
    pub fn get_stats(&self, device_id: Uuid, key: &str) -> Option<(f64, f64)> {
        self.state.get(&(device_id, key.to_string()))
            .map(|s| (s.s_high, s.s_low))
    }
}

// ── Seasonal decomposition ───────────────────────────────────────────────────

/// Simple additive seasonal decomposition (moving average method).
///
/// Decomposes time series `y[t] = trend[t] + seasonal[t] + residual[t]`.
pub struct SeasonalDecomposition {
    pub trend: Vec<f64>,
    pub seasonal: Vec<f64>,
    pub residual: Vec<f64>,
}

impl SeasonalDecomposition {
    /// Decompose a time series with a given period (e.g., 24 for hourly data with daily pattern).
    pub fn decompose(data: &[f64], period: usize) -> Option<Self> {
        if data.len() < period * 2 || period == 0 {
            return None;
        }

        let n = data.len();

        // Step 1: Compute trend via centered moving average
        let mut trend = vec![f64::NAN; n];
        let half = period / 2;
        for i in half..(n - half) {
            let sum: f64 = data[i.saturating_sub(half)..=(i + half).min(n - 1)]
                .iter()
                .sum();
            let count = (i + half).min(n - 1) - i.saturating_sub(half) + 1;
            trend[i] = sum / count as f64;
        }

        // Fill edges with nearest valid value
        let first_valid = trend.iter().find(|v| !v.is_nan()).copied();
        if let Some(fv) = first_valid {
            for v in trend.iter_mut() {
                if v.is_nan() { *v = fv; break; }
            }
        }
        // Forward fill
        let mut last = trend[0];
        for v in trend.iter_mut() {
            if v.is_nan() { *v = last; } else { last = *v; }
        }

        // Step 2: Compute seasonal component
        let detrended: Vec<f64> = data.iter()
            .zip(trend.iter())
            .map(|(d, t)| d - t)
            .collect();

        // Average detrended values by seasonal position
        let mut seasonal_avg = vec![0.0; period];
        let mut seasonal_count = vec![0usize; period];
        for (i, val) in detrended.iter().enumerate() {
            seasonal_avg[i % period] += val;
            seasonal_count[i % period] += 1;
        }
        for i in 0..period {
            if seasonal_count[i] > 0 {
                seasonal_avg[i] /= seasonal_count[i] as f64;
            }
        }

        // Center seasonal (subtract mean)
        let seasonal_mean: f64 = seasonal_avg.iter().sum::<f64>() / period as f64;
        for v in seasonal_avg.iter_mut() {
            *v -= seasonal_mean;
        }

        // Repeat seasonal pattern across full series
        let seasonal: Vec<f64> = (0..n).map(|i| seasonal_avg[i % period]).collect();

        // Step 3: Residual = data - trend - seasonal
        let residual: Vec<f64> = data.iter()
            .zip(trend.iter())
            .zip(seasonal.iter())
            .map(|((d, t), s)| d - t - s)
            .collect();

        Some(Self { trend, seasonal, residual })
    }

    /// Detect anomalies in the residual component (Z-score based).
    pub fn residual_anomalies(&self, z_threshold: f64) -> Vec<usize> {
        let valid: Vec<f64> = self.residual.iter().copied().filter(|v| !v.is_nan()).collect();
        if valid.is_empty() { return vec![]; }

        let mean = valid.iter().sum::<f64>() / valid.len() as f64;
        let std = (valid.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / valid.len() as f64).sqrt();
        if std < 1e-9 { return vec![]; }

        self.residual.iter()
            .enumerate()
            .filter(|(_, v)| ((v.abs() - mean.abs()) / std).abs() > z_threshold)
            .map(|(i, _)| i)
            .collect()
    }
}

// ── Remaining Useful Life (RUL) estimator ────────────────────────────────────

/// Simple RUL estimator based on linear degradation trend.
pub struct RulEstimator {
    /// Failure threshold — when the degradation metric reaches this value, the asset has failed.
    pub failure_threshold: f64,
    /// Historical degradation observations (timestamp_ms, value).
    observations: VecDeque<(u64, f64)>,
    /// Maximum observations to keep.
    max_observations: usize,
}

impl RulEstimator {
    pub fn new(failure_threshold: f64, max_observations: usize) -> Self {
        Self {
            failure_threshold,
            observations: VecDeque::with_capacity(max_observations),
            max_observations,
        }
    }

    /// Add a degradation observation.
    pub fn observe(&mut self, timestamp_ms: u64, value: f64) {
        self.observations.push_back((timestamp_ms, value));
        if self.observations.len() > self.max_observations {
            self.observations.pop_front();
        }
    }

    /// Estimate remaining useful life in milliseconds.
    /// Returns None if insufficient data or no degradation trend.
    pub fn estimate_rul_ms(&self) -> Option<u64> {
        if self.observations.len() < 3 {
            return None;
        }

        // Linear regression: value = slope * time + intercept
        let points: Vec<(f32, f32)> = self.observations.iter()
            .map(|(ts, v)| (*ts as f32 / 1000.0, *v as f32))
            .collect();

        let (slope, intercept) = crate::analytics::stats::linear_regression(&points)?;

        if slope.abs() < 1e-9 {
            return None; // No degradation trend
        }

        // Time when value reaches failure_threshold
        // threshold = slope * t_failure + intercept
        // t_failure = (threshold - intercept) / slope
        let t_failure = (self.failure_threshold as f32 - intercept) / slope;
        let last_time = self.observations.back()?.0 as f32 / 1000.0;

        let rul_seconds = t_failure - last_time;
        if rul_seconds <= 0.0 {
            return Some(0); // Already past threshold
        }

        Some((rul_seconds * 1000.0) as u64)
    }

    /// Estimate RUL in human-readable format.
    pub fn estimate_rul_human(&self) -> Option<String> {
        let ms = self.estimate_rul_ms()?;
        let hours = ms / 3_600_000;
        let days = hours / 24;

        if days > 0 {
            Some(format!("{}d {}h", days, hours % 24))
        } else if hours > 0 {
            Some(format!("{}h {}m", hours, (ms / 60_000) % 60))
        } else {
            Some(format!("{}m", ms / 60_000))
        }
    }

    /// Current degradation rate (units per second).
    pub fn degradation_rate(&self) -> Option<f64> {
        if self.observations.len() < 3 {
            return None;
        }
        let points: Vec<(f32, f32)> = self.observations.iter()
            .map(|(ts, v)| (*ts as f32 / 1000.0, *v as f32))
            .collect();
        crate::analytics::stats::linear_regression(&points).map(|(slope, _)| slope as f64)
    }

    /// Health percentage (0-100) based on current degradation vs threshold.
    pub fn health_pct(&self) -> Option<f64> {
        let latest = self.observations.back()?.1;
        let first = self.observations.front()?.1;
        let range = (self.failure_threshold - first).abs();
        if range < 1e-9 {
            return Some(0.0);
        }
        let degradation = (latest - first).abs();
        Some(((1.0 - degradation / range) * 100.0).clamp(0.0, 100.0))
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── CUSUM tests ──────────────────────────────────────────────────────────

    #[test]
    fn cusum_no_shift() {
        let mut det = CusumDetector::new(CusumConfig {
            target: 10.0,
            slack: 0.5,
            threshold: 5.0,
            two_sided: true,
        });
        let id = Uuid::nil();
        // Values close to target — no alert
        for i in 0..100 {
            let value = 10.0 + (i % 2) as f64 * 0.1;
            assert!(det.feed(id, "temp", value, i * 1000).is_none());
        }
    }

    #[test]
    fn cusum_upward_shift() {
        let mut det = CusumDetector::new(CusumConfig {
            target: 10.0,
            slack: 0.5,
            threshold: 5.0,
            two_sided: true,
        });
        let id = Uuid::nil();
        let mut alerted = false;
        // Values consistently above target
        for i in 0..50 {
            if det.feed(id, "temp", 12.0, i * 1000).is_some() {
                alerted = true;
                break;
            }
        }
        assert!(alerted, "CUSUM should detect upward shift");
    }

    #[test]
    fn cusum_downward_shift() {
        let mut det = CusumDetector::new(CusumConfig {
            target: 10.0,
            slack: 0.5,
            threshold: 5.0,
            two_sided: true,
        });
        let id = Uuid::nil();
        let mut alert = None;
        for i in 0..50 {
            if let Some(a) = det.feed(id, "temp", 8.0, i * 1000) {
                alert = Some(a);
                break;
            }
        }
        assert!(alert.is_some());
        assert_eq!(alert.unwrap().direction, ShiftDirection::Downward);
    }

    #[test]
    fn cusum_get_stats() {
        let mut det = CusumDetector::new(CusumConfig::default());
        let id = Uuid::nil();
        det.feed(id, "temp", 5.0, 0);
        let stats = det.get_stats(id, "temp");
        assert!(stats.is_some());
    }

    // ── Seasonal decomposition tests ─────────────────────────────────────────

    #[test]
    fn seasonal_decomposition_basic() {
        // Generate data: trend + seasonal + noise
        let period = 12;
        let n = 120;
        let data: Vec<f64> = (0..n).map(|i| {
            let trend = 100.0 + i as f64 * 0.1;
            let seasonal = 5.0 * ((2.0 * std::f64::consts::PI * i as f64 / period as f64).sin());
            trend + seasonal
        }).collect();

        let decomp = SeasonalDecomposition::decompose(&data, period).expect("should decompose");
        assert_eq!(decomp.trend.len(), n);
        assert_eq!(decomp.seasonal.len(), n);
        assert_eq!(decomp.residual.len(), n);

        // Residuals should be small (no noise added)
        let max_residual = decomp.residual.iter().map(|r| r.abs()).fold(0.0f64, f64::max);
        assert!(max_residual < 5.0, "Residuals should be small, got {max_residual}");
    }

    #[test]
    fn seasonal_decomposition_too_short() {
        assert!(SeasonalDecomposition::decompose(&[1.0, 2.0], 12).is_none());
    }

    // ── RUL estimator tests ──────────────────────────────────────────────────

    #[test]
    fn rul_linear_degradation() {
        let mut rul = RulEstimator::new(100.0, 1000);
        // Linear degradation: starts at 0, increases 1.0 per second
        for i in 0..50 {
            rul.observe(i * 1000, i as f64); // 0, 1, 2, ... 49
        }
        let rul_ms = rul.estimate_rul_ms().expect("should estimate");
        // Should reach 100 in about 50 more seconds = 50000ms
        assert!(rul_ms > 30_000 && rul_ms < 70_000,
            "RUL should be ~50s, got {}ms", rul_ms);
    }

    #[test]
    fn rul_no_degradation() {
        let mut rul = RulEstimator::new(100.0, 1000);
        for i in 0..50 {
            rul.observe(i * 1000, 25.0); // constant
        }
        assert!(rul.estimate_rul_ms().is_none(), "No degradation trend = no RUL");
    }

    #[test]
    fn rul_insufficient_data() {
        let mut rul = RulEstimator::new(100.0, 1000);
        rul.observe(0, 10.0);
        assert!(rul.estimate_rul_ms().is_none());
    }

    #[test]
    fn rul_health_pct() {
        let mut rul = RulEstimator::new(100.0, 1000);
        rul.observe(0, 0.0);
        for i in 1..=50 {
            rul.observe(i * 1000, i as f64);
        }
        let health = rul.health_pct().expect("should compute");
        assert!(health > 0.0 && health < 100.0, "Health should be partial, got {health}");
    }

    #[test]
    fn rul_human_format() {
        let mut rul = RulEstimator::new(100.0, 1000);
        for i in 0..50 {
            rul.observe(i * 1000, i as f64);
        }
        let human = rul.estimate_rul_human().expect("should format");
        assert!(!human.is_empty());
    }
}
