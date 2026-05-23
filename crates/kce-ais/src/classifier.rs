//! Self/non-self classifier — determines if a context is trusted or suspicious.
//!
//! Inspired by the negative selection algorithm in biological immune systems.
//! "Self" = known good patterns. "Non-self" = anything that deviates beyond threshold.

/// Classification result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    /// Known good pattern — trusted.
    Self_,
    /// Unknown pattern — potentially dangerous.
    NonSelf,
}

/// Probabilistic classification result with confidence score.
#[derive(Debug, Clone)]
pub struct ClassificationResult {
    /// The classification label.
    pub label: Classification,
    /// Confidence score in the range [0.0, 1.0].
    /// Higher values indicate greater certainty.
    pub confidence: f64,
}

/// Configuration for the self/non-self classifier.
#[derive(Debug, Clone)]
pub struct ClassifierConfig {
    /// Distance threshold for self-recognition.
    pub recognition_threshold: f64,
    /// Minimum number of self-patterns required before classification.
    pub min_patterns: usize,
}

impl Default for ClassifierConfig {
    fn default() -> Self {
        Self {
            recognition_threshold: 0.3,
            min_patterns: 3,
        }
    }
}

/// Classifier that distinguishes self from non-self contexts.
///
/// Implements a simplified negative selection algorithm (NSA) with probabilistic
/// confidence scoring. Detectors are random vectors that fall outside the
/// self-recognition radius, enabling stronger non-self detection.
pub struct SelfNonSelfClassifier {
    /// Known "self" patterns.
    self_patterns: Vec<Vec<f64>>,
    /// NSA detectors — random vectors that don't match any self-pattern.
    detectors: Vec<Vec<f64>>,
    config: ClassifierConfig,
    /// Xorshift64 PRNG state for reproducible detector generation.
    rng_state: u64,
}

impl SelfNonSelfClassifier {
    /// Create a new classifier with the given configuration.
    pub fn new(config: ClassifierConfig) -> Self {
        Self {
            self_patterns: Vec::new(),
            detectors: Vec::new(),
            config,
            rng_state: 42,
        }
    }

    /// Register a known "self" pattern.
    pub fn register_self(&mut self, pattern: Vec<f64>) {
        self.self_patterns.push(pattern);
    }

    /// Compute the L2 (Euclidean) distance between two vectors.
    ///
    /// Returns `f64::INFINITY` if the lengths differ.
    fn l2_distance(a: &[f64], b: &[f64]) -> f64 {
        if a.len() != b.len() {
            return f64::INFINITY;
        }
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    /// Xorshift64 PRNG — produces a pseudo-random `u64` and advances state.
    fn xorshift64(&mut self) -> u64 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        x
    }

    /// Generate a random `f64` in the range [-1.0, 1.0] using the internal PRNG.
    fn random_f64(&mut self) -> f64 {
        let bits = self.xorshift64();
        // Use the top 53 bits for f64 precision, scale to [-1.0, 1.0].
        let normalized = (bits >> 11) as f64 / ((1u64 << 53) as f64);
        normalized * 2.0 - 1.0
    }

    /// Generate NSA detectors that do not match any self-pattern.
    ///
    /// Each detector is a random vector whose distance to every self-pattern
    /// exceeds the recognition threshold. This implements the negative selection
    /// principle: only detectors that survive self-tolerance are retained.
    ///
    /// # Parameters
    /// - `num_detectors`: maximum number of detectors to generate (may produce
    ///   fewer if many candidates are rejected as self-matching).
    /// - `dimensions`: length of each detector vector.
    pub fn generate_detectors(&mut self, num_detectors: usize, dimensions: usize) {
        let threshold = self.config.recognition_threshold;
        self.detectors.clear();

        // We try up to num_detectors * 10 candidates to avoid infinite loops
        // when self-patterns cover much of the space.
        let max_attempts = num_detectors.saturating_mul(10).max(num_detectors);
        for _ in 0..max_attempts {
            if self.detectors.len() >= num_detectors {
                break;
            }
            let candidate: Vec<f64> = (0..dimensions).map(|_| self.random_f64()).collect();

            // Check that this candidate is "non-self" — its distance to every
            // self-pattern must exceed the recognition threshold.
            let is_non_self = self
                .self_patterns
                .iter()
                .all(|sp| Self::l2_distance(&candidate, sp) > threshold);

            if is_non_self {
                self.detectors.push(candidate);
            }
        }
    }

    /// Classify a vector as self or non-self with a probabilistic confidence score.
    ///
    /// Computes the minimum L2 distance to all registered self-patterns and
    /// returns a `ClassificationResult` with both the label and a confidence
    /// value in [0.0, 1.0].
    ///
    /// # Confidence calculation
    ///
    /// - If fewer than `min_patterns` are registered → `NonSelf` with confidence 0.5.
    /// - If `self_dist <= recognition_threshold` → `Self_`:
    ///   `confidence = 1.0 - (self_dist / recognition_threshold)` (closer = more confident).
    /// - If `self_dist > recognition_threshold` → `NonSelf`:
    ///   `confidence = (self_dist - threshold) / (1.0 + self_dist - threshold)`.
    ///   If NSA detectors are present and any detector matches the input vector
    ///   (distance <= threshold), the confidence is boosted.
    pub fn classify(&self, vector: &[f64]) -> ClassificationResult {
        let threshold = self.config.recognition_threshold;

        if self.self_patterns.len() < self.config.min_patterns {
            return ClassificationResult {
                label: Classification::NonSelf,
                confidence: 0.5,
            };
        }

        let self_dist = self
            .self_patterns
            .iter()
            .map(|pattern| Self::l2_distance(vector, pattern))
            .fold(f64::INFINITY, f64::min);

        if self_dist <= threshold {
            // Self: closer to center of self-region → higher confidence.
            let confidence = 1.0 - (self_dist / threshold);
            ClassificationResult {
                label: Classification::Self_,
                confidence: confidence.clamp(0.0, 1.0),
            }
        } else {
            // NonSelf: compute base confidence from distance above threshold.
            let excess = self_dist - threshold;
            let mut confidence = excess / (1.0 + excess);

            // If NSA detectors are available, check whether any detector
            // matches this vector (distance <= threshold). A matching detector
            // independently confirms non-self, boosting confidence.
            if !self.detectors.is_empty() {
                let detector_match = self
                    .detectors
                    .iter()
                    .any(|det| Self::l2_distance(vector, det) <= threshold);
                if detector_match {
                    // Boost: bring confidence closer to 1.0.
                    confidence = confidence + (1.0 - confidence) * 0.2;
                }
            }

            ClassificationResult {
                label: Classification::NonSelf,
                confidence: confidence.clamp(0.0, 1.0),
            }
        }
    }

    /// Get the number of registered self-patterns.
    pub fn self_pattern_count(&self) -> usize {
        self.self_patterns.len()
    }

    /// Get the number of generated NSA detectors.
    pub fn detector_count(&self) -> usize {
        self.detectors.len()
    }
}

impl Default for SelfNonSelfClassifier {
    fn default() -> Self {
        Self::new(ClassifierConfig::default())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn insufficient_patterns_returns_nonself() {
        let mut cls = SelfNonSelfClassifier::default();
        cls.register_self(vec![1.0, 0.0]);
        cls.register_self(vec![0.0, 1.0]);
        // Only 2 patterns, min is 3
        assert_eq!(cls.classify(&[1.0, 0.0]).label, Classification::NonSelf);
    }

    #[test]
    fn close_pattern_is_self() {
        let mut cls = SelfNonSelfClassifier::default();
        cls.register_self(vec![1.0, 0.0]);
        cls.register_self(vec![0.0, 1.0]);
        cls.register_self(vec![0.5, 0.5]);
        assert_eq!(cls.classify(&[1.0, 0.05]).label, Classification::Self_);
    }

    #[test]
    fn far_pattern_is_nonself() {
        let mut cls = SelfNonSelfClassifier::default();
        cls.register_self(vec![1.0, 0.0]);
        cls.register_self(vec![0.0, 1.0]);
        cls.register_self(vec![0.5, 0.5]);
        assert_eq!(cls.classify(&[-1.0, -1.0]).label, Classification::NonSelf);
    }

    #[test]
    fn confidence_range() {
        let mut cls = SelfNonSelfClassifier::default();
        cls.register_self(vec![1.0, 0.0]);
        cls.register_self(vec![0.0, 1.0]);
        cls.register_self(vec![0.5, 0.5]);

        // Self pattern — close
        let result = cls.classify(&[1.0, 0.05]);
        assert!(
            (0.0..=1.0).contains(&result.confidence),
            "Self confidence out of range: {}",
            result.confidence
        );

        // NonSelf — far
        let result = cls.classify(&[-1.0, -1.0]);
        assert!(
            (0.0..=1.0).contains(&result.confidence),
            "NonSelf confidence out of range: {}",
            result.confidence
        );

        // Insufficient patterns — should be 0.5
        let mut cls2 = SelfNonSelfClassifier::default();
        cls2.register_self(vec![0.0]);
        let result = cls2.classify(&[0.0]);
        assert_eq!(result.confidence, 0.5);
    }

    #[test]
    fn nsa_detectors_generated() {
        let mut cls = SelfNonSelfClassifier::default();
        cls.register_self(vec![1.0, 0.0]);
        cls.register_self(vec![0.0, 1.0]);
        cls.register_self(vec![0.5, 0.5]);

        assert_eq!(cls.detector_count(), 0);
        cls.generate_detectors(50, 2);
        assert!(
            cls.detector_count() > 0,
            "Expected at least one detector, got 0"
        );
    }

    #[test]
    fn nsa_detectors_dont_match_self() {
        let mut cls = SelfNonSelfClassifier::default();
        cls.register_self(vec![0.0, 0.0]);
        cls.register_self(vec![1.0, 0.0]);
        cls.register_self(vec![0.0, 1.0]);

        cls.generate_detectors(100, 2);
        let threshold = cls.config.recognition_threshold;

        for detector in &cls.detectors {
            for self_pattern in &cls.self_patterns {
                let dist = SelfNonSelfClassifier::l2_distance(detector, self_pattern);
                assert!(
                    dist > threshold,
                    "Detector {:?} is too close to self-pattern {:?}: dist={}",
                    detector,
                    self_pattern,
                    dist
                );
            }
        }
    }

    #[test]
    fn high_confidence_for_close_match() {
        let mut cls = SelfNonSelfClassifier::default();
        cls.register_self(vec![0.0, 0.0]);
        cls.register_self(vec![1.0, 0.0]);
        cls.register_self(vec![0.0, 1.0]);

        // Very close to [0.0, 0.0] — distance ≈ 0.01
        let result = cls.classify(&[0.005, 0.005]);
        assert_eq!(result.label, Classification::Self_);
        assert!(
            result.confidence > 0.8,
            "Expected confidence > 0.8 for close match, got {}",
            result.confidence
        );
    }
}
