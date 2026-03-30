//! Pure statistical functions for client-side anomaly detection (Phase 30).

/// Compute mean and standard deviation of a slice.
///
/// Returns `(0.0, 0.0)` for an empty slice.
pub fn mean_std(values: &[f32]) -> (f32, f32) {
    if values.is_empty() { return (0.0, 0.0); }
    let n    = values.len() as f32;
    let mean = values.iter().sum::<f32>() / n;
    let var  = values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n;
    (mean, var.sqrt())
}

/// Z-score of a single value against a distribution.
///
/// Returns `0.0` when `std < 1e-6` (constant signal — no anomaly possible).
pub fn z_score(value: f32, mean: f32, std: f32) -> f32 {
    if std < 1e-6 { return 0.0; }
    (value - mean) / std
}

/// Simple ordinary least-squares linear regression over `(x, y)` pairs.
///
/// Returns `(slope, intercept)`. Requires at least 3 points.
/// Returns `None` when the x-variance is near zero (all same x).
pub fn linear_regression(points: &[(f32, f32)]) -> Option<(f32, f32)> {
    if points.len() < 3 { return None; }
    let n      = points.len() as f32;
    let mean_x = points.iter().map(|(x, _)| x).sum::<f32>() / n;
    let mean_y = points.iter().map(|(_, y)| y).sum::<f32>() / n;
    let ss_xx  = points.iter().map(|(x, _)| (x - mean_x).powi(2)).sum::<f32>();
    let ss_xy  = points.iter().map(|(x, y)| (x - mean_x) * (y - mean_y)).sum::<f32>();
    if ss_xx.abs() < 1e-9 { return None; }
    let slope     = ss_xy / ss_xx;
    let intercept = mean_y - slope * mean_x;
    Some((slope, intercept))
}

/// Exponential Weighted Moving Average — suitable for online streaming data.
pub struct Ewma {
    /// Smoothing factor (0 < alpha < 1). Higher = more reactive.
    pub alpha: f32,
    /// Current EWMA value; `None` until first update.
    pub value: Option<f32>,
}

impl Ewma {
    pub fn new(alpha: f32) -> Self {
        Self { alpha, value: None }
    }

    /// Feed a new observation; returns the updated EWMA value.
    pub fn update(&mut self, x: f32) -> f32 {
        let v = match self.value {
            None    => x,
            Some(v) => self.alpha * x + (1.0 - self.alpha) * v,
        };
        self.value = Some(v);
        v
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_std_basic() {
        let (m, s) = mean_std(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!((m - 3.0).abs() < 0.01);
        // Population std = sqrt(2) ≈ 1.4142
        assert!((s - std::f32::consts::SQRT_2).abs() < 0.01);
    }

    #[test]
    fn mean_std_empty() {
        let (m, s) = mean_std(&[]);
        assert_eq!(m, 0.0);
        assert_eq!(s, 0.0);
    }

    #[test]
    fn z_score_boundary() {
        // Zero std → always 0
        assert_eq!(z_score(5.0, 5.0, 0.0), 0.0);
        // 3 std devs away
        assert!((z_score(8.0, 5.0, 1.0) - 3.0).abs() < 0.01);
    }

    #[test]
    fn linear_regression_slope() {
        // y = 2x + 1
        let pts: Vec<(f32, f32)> = (0..10).map(|i| (i as f32, i as f32 * 2.0 + 1.0)).collect();
        let (slope, intercept) = linear_regression(&pts).unwrap();
        assert!((slope - 2.0).abs() < 0.01);
        assert!((intercept - 1.0).abs() < 0.1);
    }

    #[test]
    fn linear_regression_too_few_points() {
        assert!(linear_regression(&[(0.0, 1.0), (1.0, 2.0)]).is_none());
    }

    #[test]
    fn ewma_converges_to_constant_input() {
        let mut e = Ewma::new(0.3);
        for _ in 0..100 { e.update(10.0); }
        assert!((e.value.unwrap() - 10.0).abs() < 0.01);
    }

    #[test]
    fn ewma_first_update_is_value() {
        let mut e = Ewma::new(0.5);
        assert_eq!(e.update(42.0), 42.0);
    }
}
