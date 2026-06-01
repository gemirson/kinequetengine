//! Immune anomaly detection.
//!
//! Detects anomalous context vectors by comparing them against a set of
//! known "normal" patterns. Uses distance-based thresholding inspired
//! by the biological immune system's antigen recognition.

use std::collections::VecDeque;

use kce_core::error::MetricError;
use serde::{Deserialize, Serialize};

/// Result of anomaly detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    /// Whether an anomaly was detected.
    pub is_anomaly: bool,
    /// Normalized anomaly score (0.0 = normal, 1.0 = anomalous).
    pub score: f64,
    /// The raw minimum distance to any known pattern.
    pub min_distance: f64,
    /// The detection threshold that was used.
    pub threshold: f64,
    /// Index of the closest matching pattern (-1 if none).
    pub closest_pattern_idx: isize,
    /// Current false positive rate estimate (0.0..1.0).
    pub fpr: f64,
}

/// Configuration for the immune detector.
#[derive(Debug, Clone)]
pub struct DetectorConfig {
    /// Distance threshold above which a vector is considered anomalous.
    pub threshold: f64,
    /// Number of initial interactions for warm-up phase.
    pub warmup_count: usize,
    /// Sliding window size for baseline computation.
    pub window_size: usize,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            threshold: 0.7,
            warmup_count: 1000,
            window_size: 10_000,
        }
    }
}

/// Anomaly detector based on distance to known patterns.
pub struct ImmuneDetector {
    /// Known "normal" patterns (antibodies).
    patterns: Vec<Vec<f64>>,
    config: DetectorConfig,
    /// Number of detect() calls made (for warm-up tracking).
    call_count: usize,
    /// Maximum distance seen (for normalization).
    max_distance_seen: f64,
    /// Sliding window of recent distances for baseline computation.
    window: VecDeque<f64>,
    /// Running mean of window distances (baseline).
    baseline_sum: f64,
    /// Count of false positives: vectors flagged anomalous but within
    /// baseline + 1 stddev.
    false_positives: u64,
    /// Total detections after warm-up.
    total_post_warmup: u64,
}

impl ImmuneDetector {
    /// Create a new detector with the given configuration.
    pub fn new(config: DetectorConfig) -> Self {
        Self {
            patterns: Vec::new(),
            config,
            call_count: 0,
            max_distance_seen: 1.0,
            window: VecDeque::new(),
            baseline_sum: 0.0,
            false_positives: 0,
            total_post_warmup: 0,
        }
    }

    /// Add a known normal pattern.
    pub fn add_pattern(&mut self, pattern: Vec<f64>) {
        self.patterns.push(pattern);
    }

    /// Get the number of stored patterns.
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Whether the detector has completed its warm-up phase.
    pub fn is_warmed_up(&self) -> bool {
        self.call_count >= self.config.warmup_count
    }

    /// Get the current call count.
    pub fn call_count(&self) -> usize {
        self.call_count
    }

    /// Detect whether a vector is anomalous.
    ///
    /// Computes the L2 distance to each known pattern and returns
    /// the minimum, normalized to 0.0-1.0 range.
    /// If no patterns are stored, everything is anomalous.
    ///
    /// Maintains a sliding window of recent distances for dynamic baseline
    /// and tracks false positive rate after warm-up.
    ///
    /// # Errors
    ///
    /// Returns [`MetricError::DimensionMismatch`] if the vector has a
    /// different dimension than stored patterns.
    pub fn detect(&mut self, vector: &[f64]) -> Result<DetectionResult, MetricError> {
        self.call_count += 1;

        if self.patterns.is_empty() {
            return Ok(DetectionResult {
                is_anomaly: true,
                score: 1.0,
                min_distance: f64::INFINITY,
                threshold: self.config.threshold,
                closest_pattern_idx: -1,
                fpr: 0.0,
            });
        }

        let mut min_dist = f64::INFINITY;
        let mut closest_idx = 0usize;

        for (i, pattern) in self.patterns.iter().enumerate() {
            if pattern.len() != vector.len() {
                return Err(MetricError::DimensionMismatch(vector.len(), pattern.len()));
            }

            let dist: f64 = vector
                .iter()
                .zip(pattern.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();

            if dist < min_dist {
                min_dist = dist;
                closest_idx = i;
            }
        }

        // Update max distance for normalization
        if min_dist > self.max_distance_seen {
            self.max_distance_seen = min_dist;
        }

        // Normalize score to 0.0-1.0
        let normalized_score = (min_dist / self.max_distance_seen).min(1.0);
        let is_anomaly = normalized_score > self.config.threshold;

        // Sliding window: add current distance, evict oldest if full
        self.window.push_back(min_dist);
        self.baseline_sum += min_dist;
        if self.window.len() > self.config.window_size {
            if let Some(old) = self.window.pop_front() {
                self.baseline_sum -= old;
            }
        }

        // After warm-up, track FPR: anomaly flagged but distance within
        // baseline_mean + 1 stddev is considered a false positive.
        if self.is_warmed_up() {
            self.total_post_warmup += 1;
            if is_anomaly && self.window.len() >= 2 {
                let mean = self.baseline_sum / self.window.len() as f64;
                let variance: f64 = self
                    .window
                    .iter()
                    .map(|d| (d - mean).powi(2))
                    .sum::<f64>()
                    / self.window.len() as f64;
                let stddev = variance.sqrt();
                if min_dist <= mean + stddev {
                    self.false_positives += 1;
                }
            }
        }

        let fpr = if self.total_post_warmup > 0 {
            self.false_positives as f64 / self.total_post_warmup as f64
        } else {
            0.0
        };

        Ok(DetectionResult {
            is_anomaly,
            score: normalized_score,
            min_distance: min_dist,
            threshold: self.config.threshold,
            closest_pattern_idx: closest_idx as isize,
            fpr,
        })
    }

    /// Get the detection threshold.
    pub fn threshold(&self) -> f64 {
        self.config.threshold
    }

    /// Get the current sliding window baseline mean distance.
    pub fn baseline_mean(&self) -> f64 {
        if self.window.is_empty() {
            0.0
        } else {
            self.baseline_sum / self.window.len() as f64
        }
    }

    /// Get the current false positive rate estimate.
    pub fn fpr(&self) -> f64 {
        if self.total_post_warmup > 0 {
            self.false_positives as f64 / self.total_post_warmup as f64
        } else {
            0.0
        }
    }

    /// Get the current sliding window size.
    pub fn window_len(&self) -> usize {
        self.window.len()
    }
}

impl Default for ImmuneDetector {
    fn default() -> Self {
        Self::new(DetectorConfig::default())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn empty_patterns_detects_everything() {
        let mut detector = ImmuneDetector::default();
        let r = detector.detect(&[1.0, 2.0]).unwrap();
        assert!(r.is_anomaly);
        assert!((r.score - 1.0).abs() < 1e-10);
    }

    #[test]
    fn close_vector_not_anomaly() {
        let mut detector = ImmuneDetector::new(DetectorConfig {
            threshold: 0.7,
            ..Default::default()
        });
        detector.add_pattern(vec![1.0, 0.0]);
        let r = detector.detect(&[1.0, 0.1]).unwrap();
        assert!(!r.is_anomaly);
    }

    #[test]
    fn far_vector_is_anomaly() {
        let mut detector = ImmuneDetector::new(DetectorConfig {
            threshold: 0.3,
            ..Default::default()
        });
        detector.add_pattern(vec![1.0, 0.0]);
        let r = detector.detect(&[0.0, 1.0]).unwrap();
        assert!(r.is_anomaly);
    }

    #[test]
    fn closest_pattern_index_correct() {
        let mut detector = ImmuneDetector::new(DetectorConfig {
            threshold: 2.0,
            ..Default::default()
        });
        detector.add_pattern(vec![0.0, 0.0]);
        detector.add_pattern(vec![1.0, 0.0]);
        detector.add_pattern(vec![10.0, 0.0]);
        let r = detector.detect(&[0.9, 0.0]).unwrap();
        assert_eq!(r.closest_pattern_idx, 1);
    }

    #[test]
    fn dimension_mismatch_returns_error() {
        let mut detector = ImmuneDetector::default();
        detector.add_pattern(vec![1.0, 2.0]);
        assert!(detector.detect(&[1.0]).is_err());
    }

    #[test]
    fn normalized_score_range() {
        let mut detector = ImmuneDetector::default();
        detector.add_pattern(vec![0.0, 0.0]);
        detector.add_pattern(vec![1.0, 0.0]);
        let r = detector.detect(&[0.5, 0.0]).unwrap();
        assert!((0.0..=1.0).contains(&r.score));
    }

    #[test]
    fn warmup_tracking() {
        let config = DetectorConfig {
            warmup_count: 3,
            ..Default::default()
        };
        let mut detector = ImmuneDetector::new(config);
        detector.add_pattern(vec![1.0]);
        assert!(!detector.is_warmed_up());
        detector.detect(&[1.0]).unwrap();
        detector.detect(&[1.0]).unwrap();
        assert!(!detector.is_warmed_up());
        detector.detect(&[1.0]).unwrap();
        assert!(detector.is_warmed_up());
    }

    #[test]
    fn sliding_window_tracks_distances() {
        let config = DetectorConfig {
            window_size: 5,
            warmup_count: 0,
            ..Default::default()
        };
        let mut detector = ImmuneDetector::new(config);
        detector.add_pattern(vec![0.0, 0.0]);
        for _ in 0..10 {
            detector.detect(&[0.5, 0.0]).unwrap();
        }
        assert_eq!(detector.window_len(), 5);
    }

    #[test]
    fn baseline_mean_updates() {
        let config = DetectorConfig {
            window_size: 100,
            warmup_count: 0,
            ..Default::default()
        };
        let mut detector = ImmuneDetector::new(config);
        detector.add_pattern(vec![0.0, 0.0]);
        detector.detect(&[0.5, 0.0]).unwrap();
        assert!(detector.baseline_mean() > 0.0);
    }

    #[test]
    fn fpr_zero_before_warmup() {
        let config = DetectorConfig {
            warmup_count: 100,
            ..Default::default()
        };
        let mut detector = ImmuneDetector::new(config);
        detector.add_pattern(vec![0.0, 0.0]);
        let r = detector.detect(&[1.0, 0.0]).unwrap();
        assert_eq!(r.fpr, 0.0);
    }

    #[test]
    fn fpr_reported_after_warmup() {
        let config = DetectorConfig {
            warmup_count: 2,
            window_size: 100,
            threshold: 0.1,
            ..Default::default()
        };
        let mut detector = ImmuneDetector::new(config);
        detector.add_pattern(vec![0.0, 0.0]);
        // 3 calls to get past warmup
        detector.detect(&[0.5, 0.0]).unwrap();
        detector.detect(&[0.5, 0.0]).unwrap();
        let r = detector.detect(&[0.5, 0.0]).unwrap();
        // FPR is a valid ratio
        assert!(r.fpr >= 0.0 && r.fpr <= 1.0);
    }
}
