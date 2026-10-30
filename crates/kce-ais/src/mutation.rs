//! Adaptive mutation — controlled mutation of embeddings for exploration.
//!
//! Inspired by somatic hypermutation in B-cells: when a pattern is
//! close but not quite matching, mutate it slightly to explore nearby
//! solution space. Includes elitism (keep the best variant).

/// Configuration for adaptive mutation.
#[derive(Debug, Clone)]
pub struct MutationConfig {
    /// Standard deviation of Gaussian noise applied during mutation.
    pub mutation_rate: f64,
    /// Number of variants to generate per mutation call.
    pub num_variants: usize,
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            mutation_rate: 0.05,
            num_variants: 5,
        }
    }
}

/// Result of a mutation operation.
#[derive(Debug, Clone)]
pub struct MutationResult {
    /// The mutated variants.
    pub variants: Vec<Vec<f64>>,
    /// Scores for each variant (higher = better).
    pub scores: Vec<f64>,
    /// Index of the best variant (elitism).
    pub best_idx: usize,
}

/// Result of an explore_mutations operation with elitism against the original.
#[derive(Debug, Clone)]
pub struct ExploreResult {
    /// The original vector that was explored from.
    pub original: Vec<f64>,
    /// Fitness score of the original vector.
    pub original_score: f64,
    /// The selected vector (original or best mutation).
    pub selected: Vec<f64>,
    /// Fitness score of the selected vector.
    pub selected_score: f64,
    /// Whether the selected vector is a mutation (true) or the original (false).
    pub is_mutation: bool,
    /// Number of mutations that improved on the original.
    pub improvements: usize,
    /// Total number of mutation attempts.
    pub total_attempts: usize,
}

/// Adaptive mutator for embedding vectors.
pub struct AdaptiveMutator {
    config: MutationConfig,
    rng_state: u64,
}

impl AdaptiveMutator {
    /// Create a new mutator with the given configuration.
    pub fn new(config: MutationConfig) -> Self {
        Self {
            config,
            rng_state: 42,
        }
    }

    /// Mutate a vector, producing multiple variants.
    ///
    /// Each variant is the original vector plus Gaussian noise with
    /// the configured mutation rate.
    pub fn mutate(&mut self, original: &[f64]) -> MutationResult {
        let mut variants = Vec::with_capacity(self.config.num_variants);

        for _ in 0..self.config.num_variants {
            let variant: Vec<f64> = original
                .iter()
                .map(|&v| v + self.gaussian_noise() * self.config.mutation_rate)
                .collect();
            variants.push(variant);
        }

        // Scores are placeholder — in production, these would be
        // evaluated by a fitness function
        let scores: Vec<f64> = variants.iter().map(|v| self.rough_norm(v)).collect();

        let best_idx = scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);

        MutationResult {
            variants,
            scores,
            best_idx,
        }
    }

    /// Set the PRNG seed for reproducibility.
    pub fn set_seed(&mut self, seed: u64) {
        self.rng_state = seed;
    }

    /// Explore mutations around an original vector using a fitness function.
    ///
    /// Generates `n` mutated variants and evaluates each with the provided
    /// fitness function. If any mutation scores higher than the original,
    /// the best mutation is returned; otherwise the original is preserved
    /// (elitism).
    pub fn explore_mutations<F>(
        &mut self,
        original: &[f64],
        n: usize,
        fitness: F,
    ) -> ExploreResult
    where
        F: Fn(&[f64]) -> f64,
    {
        let original_score = fitness(original);
        let mut best_score = original_score;
        let mut best_variant: Option<Vec<f64>> = None;
        let mut improvements = 0usize;

        for _ in 0..n {
            let variant: Vec<f64> = original
                .iter()
                .map(|&v| v + self.gaussian_noise() * self.config.mutation_rate)
                .collect();
            let score = fitness(&variant);
            if score > best_score {
                best_score = score;
                best_variant = Some(variant);
                improvements += 1;
            }
        }

        ExploreResult {
            original: original.to_vec(),
            original_score,
            selected: best_variant.clone().unwrap_or_else(|| original.to_vec()),
            selected_score: best_score,
            is_mutation: best_variant.is_some(),
            improvements,
            total_attempts: n,
        }
    }

    /// Re-normalize a vector to unit length (L2 norm).
    pub fn normalize(vector: &mut [f64]) {
        let norm: f64 = vector.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 0.0 {
            for v in vector.iter_mut() {
                *v /= norm;
            }
        }
    }

    /// Generate Gaussian noise using Box-Muller transform.
    fn gaussian_noise(&mut self) -> f64 {
        let u1 = self.next_f64().max(1e-10);
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }

    /// Simple xorshift64 PRNG.
    fn next_f64(&mut self) -> f64 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        (self.rng_state as f64) / (u64::MAX as f64)
    }

    /// Rough norm as fitness proxy (sum of absolute values).
    fn rough_norm(&self, v: &[f64]) -> f64 {
        v.iter().map(|x| x.abs()).sum()
    }
}

impl Default for AdaptiveMutator {
    fn default() -> Self {
        Self::new(MutationConfig::default())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn mutation_produces_variants() {
        let mut mutator = AdaptiveMutator::default();
        let result = mutator.mutate(&[1.0, 0.0, 0.0]);
        assert_eq!(result.variants.len(), 5);
        assert_eq!(result.scores.len(), 5);
    }

    #[test]
    fn variants_differ_from_original() {
        let mut mutator = AdaptiveMutator::new(MutationConfig {
            mutation_rate: 0.5,
            num_variants: 10,
        });
        let result = mutator.mutate(&[1.0, 0.0]);
        // At least some variants should differ
        let any_different = result
            .variants
            .iter()
            .any(|v| (v[0] - 1.0).abs() > 0.01 || v[1].abs() > 0.01);
        assert!(any_different);
    }

    #[test]
    fn normalize_produces_unit_vector() {
        let mut v = vec![3.0, 4.0];
        AdaptiveMutator::normalize(&mut v);
        let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((norm - 1.0).abs() < 1e-10);
    }

    #[test]
    fn best_idx_is_valid() {
        let mut mutator = AdaptiveMutator::default();
        let result = mutator.mutate(&[0.5, 0.5]);
        assert!(result.best_idx < result.variants.len());
    }

    #[test]
    fn default_mutation_rate_is_0_05() {
        let config = MutationConfig::default();
        assert!((config.mutation_rate - 0.05).abs() < 1e-10);
    }

    #[test]
    fn test_explore_mutations_improvement() {
        // Fitness function that strongly favors a specific direction (larger x)
        let fitness = |v: &[f64]| -> f64 { v[0] * 10.0 + v[1] };
        let mut mutator = AdaptiveMutator::new(MutationConfig {
            mutation_rate: 0.5,
            num_variants: 5,
        });
        mutator.set_seed(42);

        let original = &[0.0, 0.0];
        let result = mutator.explore_mutations(original, 100, fitness);

        // With a large mutation_rate and many attempts, at least one mutation
        // should push x positive enough to improve over 0.0
        assert!(
            result.improvements > 0 || result.selected_score > result.original_score,
            "Expected at least one improvement; got improvements={}, selected_score={}, original_score={}",
            result.improvements,
            result.selected_score,
            result.original_score
        );
    }

    #[test]
    fn test_explore_mutations_elitism() {
        // Fitness function that is maximized at the original point (0, 0)
        // Any mutation away from it will decrease the score
        let fitness = |v: &[f64]| -> f64 { -(v[0] * v[0] + v[1] * v[1]) };
        let mut mutator = AdaptiveMutator::new(MutationConfig {
            mutation_rate: 1.0,
            num_variants: 5,
        });
        mutator.set_seed(99);

        let original = &[0.0, 0.0];
        let result = mutator.explore_mutations(original, 50, fitness);

        // Since the original is the global optimum, no mutation should improve it
        assert!(!result.is_mutation, "Expected no mutation to beat the optimum");
        assert_eq!(result.improvements, 0);
        assert_eq!(result.selected, original);
        assert!((result.selected_score - result.original_score).abs() < 1e-15);
    }

    #[test]
    fn test_explore_mutations_is_mutation_flag() {
        // Fitness that rewards larger first component
        let fitness = |v: &[f64]| -> f64 { v[0] };
        let mut mutator = AdaptiveMutator::new(MutationConfig {
            mutation_rate: 0.5,
            num_variants: 5,
        });
        mutator.set_seed(123);

        let original = &[-10.0, -10.0];
        let result = mutator.explore_mutations(original, 200, fitness);

        // With such a large negative starting point and positive noise,
        // many mutations should improve
        if result.improvements > 0 {
            assert!(result.is_mutation, "is_mutation should be true when improvements > 0");
        } else {
            assert!(!result.is_mutation, "is_mutation should be false when no improvements");
        }
    }

    #[test]
    fn test_set_seed_reproducibility() {
        let fitness = |v: &[f64]| -> f64 { v.iter().sum::<f64>() };
        let original = &[1.0, 2.0, 3.0];

        // Run 1
        let mut mutator1 = AdaptiveMutator::default();
        mutator1.set_seed(7777);
        let result1 = mutator1.explore_mutations(original, 20, &fitness);

        // Run 2 with same seed
        let mut mutator2 = AdaptiveMutator::default();
        mutator2.set_seed(7777);
        let result2 = mutator2.explore_mutations(original, 20, &fitness);

        assert_eq!(result1.selected, result2.selected);
        assert!((result1.selected_score - result2.selected_score).abs() < 1e-15);
        assert_eq!(result1.improvements, result2.improvements);
    }

    #[test]
    fn test_explore_result_fields() {
        let fitness = |v: &[f64]| -> f64 { v[0] * v[0] + v[1] * v[1] };
        let mut mutator = AdaptiveMutator::new(MutationConfig {
            mutation_rate: 0.1,
            num_variants: 5,
        });
        mutator.set_seed(42);

        let original = &[1.0, 1.0];
        let result = mutator.explore_mutations(original, 10, &fitness);

        // All fields should be populated
        assert_eq!(result.original, vec![1.0, 1.0]);
        assert!((result.original_score - 2.0).abs() < 1e-15); // 1^2 + 1^2 = 2
        assert_eq!(result.selected.len(), 2);
        assert!(result.total_attempts == 10);
        // selected_score should be >= original_score (elitism)
        assert!(result.selected_score >= result.original_score - 1e-15);
        // is_mutation consistency
        if result.improvements > 0 {
            assert!(result.is_mutation);
            assert_ne!(result.selected, result.original);
        } else {
            assert!(!result.is_mutation);
            assert_eq!(result.selected, result.original);
        }
    }
}
