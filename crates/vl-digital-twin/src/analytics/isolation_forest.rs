//! Isolation Forest — unsupervised anomaly detection algorithm.
//!
//! Builds a forest of random binary trees. Anomalies are isolated in fewer
//! splits (shorter average path length) than normal points.
//!
//! Reference: Liu, Ting & Zhou (2008) "Isolation Forest"

use std::collections::VecDeque;

/// Configuration for the Isolation Forest.
#[derive(Debug, Clone)]
pub struct IsolationForestConfig {
    /// Number of trees in the forest.
    pub n_trees: usize,
    /// Sub-sampling size for each tree.
    pub sample_size: usize,
    /// Anomaly score threshold (0.0 - 1.0). Scores above this are anomalies.
    pub threshold: f64,
    /// Maximum tree depth (default: ceil(log2(sample_size))).
    pub max_depth: Option<usize>,
}

impl Default for IsolationForestConfig {
    fn default() -> Self {
        Self {
            n_trees: 100,
            sample_size: 256,
            threshold: 0.6,
            max_depth: None,
        }
    }
}

/// A single isolation tree node.
#[derive(Debug)]
enum ITreeNode {
    Internal {
        feature_idx: usize,
        split_value: f64,
        left: Box<ITreeNode>,
        right: Box<ITreeNode>,
    },
    Leaf {
        size: usize,
    },
}

/// A single isolation tree.
struct IsolationTree {
    root: ITreeNode,
}

/// The Isolation Forest model.
pub struct IsolationForest {
    config: IsolationForestConfig,
    trees: Vec<IsolationTree>,
    n_features: usize,
}

impl IsolationForest {
    /// Fit the forest on a dataset (rows of feature vectors).
    pub fn fit(data: &[Vec<f64>], config: IsolationForestConfig) -> Self {
        if data.is_empty() {
            return Self {
                config,
                trees: Vec::new(),
                n_features: 0,
            };
        }

        let n_features = data[0].len();
        let max_depth = config.max_depth.unwrap_or_else(|| {
            (config.sample_size as f64).log2().ceil() as usize
        });

        let mut trees = Vec::with_capacity(config.n_trees);
        let mut rng = SimpleRng::new(42);

        for _ in 0..config.n_trees {
            let sample = subsample(data, config.sample_size, &mut rng);
            let root = build_itree(&sample, n_features, 0, max_depth, &mut rng);
            trees.push(IsolationTree { root });
        }

        Self {
            config,
            trees,
            n_features,
        }
    }

    /// Compute anomaly score for a single data point.
    /// Returns a score between 0.0 (normal) and 1.0 (anomalous).
    pub fn score(&self, point: &[f64]) -> f64 {
        if self.trees.is_empty() || point.len() != self.n_features {
            return 0.0;
        }

        let avg_path_length: f64 = self.trees.iter()
            .map(|tree| path_length(&tree.root, point, 0) as f64)
            .sum::<f64>() / self.trees.len() as f64;

        let c = c_factor(self.config.sample_size);
        // s(x, n) = 2^(-E[h(x)] / c(n))
        2.0_f64.powf(-avg_path_length / c)
    }

    /// Check if a point is an anomaly.
    pub fn is_anomaly(&self, point: &[f64]) -> bool {
        self.score(point) > self.config.threshold
    }

    /// Score a batch of points, returning (index, score) pairs for anomalies.
    pub fn detect_anomalies(&self, data: &[Vec<f64>]) -> Vec<(usize, f64)> {
        data.iter()
            .enumerate()
            .filter_map(|(i, point)| {
                let score = self.score(point);
                if score > self.config.threshold {
                    Some((i, score))
                } else {
                    None
                }
            })
            .collect()
    }
}

// ── Tree construction ────────────────────────────────────────────────────────

fn build_itree(
    data: &[Vec<f64>],
    n_features: usize,
    depth: usize,
    max_depth: usize,
    rng: &mut SimpleRng,
) -> ITreeNode {
    if depth >= max_depth || data.len() <= 1 {
        return ITreeNode::Leaf { size: data.len() };
    }

    // Random feature selection
    let feature_idx = rng.next_usize() % n_features;

    // Find min/max of selected feature
    let (min_val, max_val) = data.iter().fold((f64::MAX, f64::MIN), |(min, max), row| {
        let v = row[feature_idx];
        (min.min(v), max.max(v))
    });

    if (max_val - min_val).abs() < 1e-10 {
        return ITreeNode::Leaf { size: data.len() };
    }

    // Random split point
    let split_value = min_val + rng.next_f64() * (max_val - min_val);

    let (left_data, right_data): (Vec<_>, Vec<_>) = data.iter()
        .cloned()
        .partition(|row| row[feature_idx] < split_value);

    ITreeNode::Internal {
        feature_idx,
        split_value,
        left: Box::new(build_itree(&left_data, n_features, depth + 1, max_depth, rng)),
        right: Box::new(build_itree(&right_data, n_features, depth + 1, max_depth, rng)),
    }
}

fn path_length(node: &ITreeNode, point: &[f64], depth: usize) -> usize {
    match node {
        ITreeNode::Leaf { size } => depth + c_factor(*size) as usize,
        ITreeNode::Internal { feature_idx, split_value, left, right } => {
            if point[*feature_idx] < *split_value {
                path_length(left, point, depth + 1)
            } else {
                path_length(right, point, depth + 1)
            }
        }
    }
}

/// Average path length of unsuccessful search in BST (harmonic number approximation).
fn c_factor(n: usize) -> f64 {
    if n <= 1 {
        return 0.0;
    }
    let n = n as f64;
    2.0 * (n.ln() + 0.5772156649) - 2.0 * (n - 1.0) / n
}

// ── Simple RNG (no external dependency) ──────────────────────────────────────

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.state
    }

    fn next_usize(&mut self) -> usize {
        self.next_u64() as usize
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn subsample(data: &[Vec<f64>], sample_size: usize, rng: &mut SimpleRng) -> Vec<Vec<f64>> {
    if data.len() <= sample_size {
        return data.to_vec();
    }
    let mut indices: Vec<usize> = (0..data.len()).collect();
    // Fisher-Yates shuffle (partial)
    for i in 0..sample_size.min(indices.len()) {
        let j = i + rng.next_usize() % (indices.len() - i);
        indices.swap(i, j);
    }
    indices[..sample_size].iter().map(|&i| data[i].clone()).collect()
}

// ── Streaming adapter ────────────────────────────────────────────────────────

/// Online Isolation Forest that maintains a sliding window and periodically retrains.
pub struct StreamingIsolationForest {
    config: IsolationForestConfig,
    window: VecDeque<Vec<f64>>,
    window_size: usize,
    retrain_interval: usize,
    samples_since_retrain: usize,
    forest: Option<IsolationForest>,
}

impl StreamingIsolationForest {
    pub fn new(config: IsolationForestConfig, window_size: usize, retrain_interval: usize) -> Self {
        Self {
            config,
            window: VecDeque::with_capacity(window_size),
            window_size,
            retrain_interval,
            samples_since_retrain: 0,
            forest: None,
        }
    }

    /// Feed a new data point. Returns anomaly score if the forest is trained.
    pub fn feed(&mut self, point: Vec<f64>) -> Option<f64> {
        self.window.push_back(point.clone());
        if self.window.len() > self.window_size {
            self.window.pop_front();
        }
        self.samples_since_retrain += 1;

        // Retrain periodically
        if self.samples_since_retrain >= self.retrain_interval
            && self.window.len() >= self.config.sample_size
        {
            let data: Vec<Vec<f64>> = self.window.iter().cloned().collect();
            self.forest = Some(IsolationForest::fit(&data, self.config.clone()));
            self.samples_since_retrain = 0;
        }

        self.forest.as_ref().map(|f| f.score(&point))
    }

    pub fn is_trained(&self) -> bool {
        self.forest.is_some()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn normal_data(n: usize) -> Vec<Vec<f64>> {
        let mut rng = SimpleRng::new(123);
        (0..n)
            .map(|_| {
                vec![
                    50.0 + (rng.next_f64() - 0.5) * 10.0,
                    100.0 + (rng.next_f64() - 0.5) * 20.0,
                ]
            })
            .collect()
    }

    #[test]
    fn fit_and_score_normal() {
        let data = normal_data(500);
        let forest = IsolationForest::fit(&data, IsolationForestConfig::default());

        // Normal points should have low scores
        let score = forest.score(&[50.0, 100.0]);
        assert!(score < 0.6, "Normal point score {score} should be below threshold");
    }

    #[test]
    fn anomaly_has_high_score() {
        let data = normal_data(500);
        let forest = IsolationForest::fit(&data, IsolationForestConfig::default());

        // Outlier far from cluster
        let score = forest.score(&[200.0, 500.0]);
        assert!(score > 0.5, "Anomaly score {score} should be high");
    }

    #[test]
    fn detect_anomalies_batch() {
        let mut data = normal_data(500);
        // Inject anomalies
        data.push(vec![300.0, 800.0]);
        data.push(vec![-200.0, -500.0]);

        let forest = IsolationForest::fit(&data, IsolationForestConfig {
            threshold: 0.55,
            ..Default::default()
        });

        let anomalies = forest.detect_anomalies(&data);
        // Should detect the injected anomalies
        assert!(!anomalies.is_empty(), "Should detect at least some anomalies");
    }

    #[test]
    fn empty_data() {
        let forest = IsolationForest::fit(&[], IsolationForestConfig::default());
        assert_eq!(forest.score(&[1.0, 2.0]), 0.0);
    }

    #[test]
    fn streaming_forest() {
        let mut sf = StreamingIsolationForest::new(
            IsolationForestConfig { sample_size: 50, n_trees: 10, ..Default::default() },
            200,
            100,
        );

        let mut rng = SimpleRng::new(99);
        // Feed normal data
        for _ in 0..150 {
            let point = vec![50.0 + (rng.next_f64() - 0.5) * 10.0];
            sf.feed(point);
        }

        assert!(sf.is_trained(), "Forest should be trained after enough samples");

        // Feed an anomaly
        let score = sf.feed(vec![500.0]);
        assert!(score.is_some());
    }

    #[test]
    fn c_factor_values() {
        assert_eq!(c_factor(0), 0.0);
        assert_eq!(c_factor(1), 0.0);
        assert!(c_factor(256) > 0.0);
    }
}
