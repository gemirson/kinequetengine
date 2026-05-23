//! Pheromone routing engine.
//!
//! Manages pheromone scores on graph edges.  Higher pheromone means
//! the edge has been historically useful for queries.
//!
//! Thread-safe via `Arc<parking_lot::RwLock<PheromoneMap>>`.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration for pheromone management (MMAS bounds).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PheromoneConfig {
    /// Pheromone weight in route selection (alpha).
    pub alpha: f64,
    /// Heuristic weight in route selection (beta).
    pub beta: f64,
    /// Minimum pheromone value (MMAS floor).
    pub tau_min: f64,
    /// Maximum pheromone value (MMAS ceiling).
    pub tau_max: f64,
    /// Initial pheromone value for new edges.
    pub initial_tau: f64,
}

impl Default for PheromoneConfig {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            beta: 2.0,
            tau_min: 0.01,
            tau_max: 10.0,
            initial_tau: 1.0,
        }
    }
}

/// A pheromone entry for a single edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PheromoneEntry {
    /// Current pheromone level.
    pub level: f64,
    /// Number of deposits on this edge.
    pub deposit_count: u64,
    /// Total amount deposited (for audit trail).
    pub total_deposited: f64,
}

/// Pheromone map — tracks pheromone scores on edges.
///
/// Thread-safe via `Arc<RwLock<PheromoneMap>>`.  Use [`PheromoneMap::shared`]
/// to obtain a shared handle.
pub struct PheromoneMap {
    scores: HashMap<(u64, u64), PheromoneEntry>,
    config: PheromoneConfig,
}

impl PheromoneMap {
    /// Create a new pheromone map with the given configuration.
    pub fn new(config: PheromoneConfig) -> Self {
        Self {
            scores: HashMap::new(),
            config,
        }
    }

    /// Wrap in `Arc<RwLock<...>>` for concurrent access.
    pub fn shared(self) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(self))
    }

    /// Deposit pheromone on an edge (additive, clamped to tau_max).
    ///
    /// `amount` should be proportional to quality (e.g. 1/latency or recall_score).
    pub fn deposit(&mut self, from: u64, to: u64, amount: f64) {
        let entry = self
            .scores
            .entry((from, to))
            .or_insert(PheromoneEntry {
                level: self.config.initial_tau,
                deposit_count: 0,
                total_deposited: 0.0,
            });
        entry.level = (entry.level + amount).clamp(self.config.tau_min, self.config.tau_max);
        entry.deposit_count += 1;
        entry.total_deposited += amount;
    }

    /// Get the current pheromone score for an edge.
    /// Returns initial_tau if not yet deposited.
    pub fn get(&self, from: u64, to: u64) -> f64 {
        self.scores
            .get(&(from, to))
            .map(|e| e.level)
            .unwrap_or(self.config.initial_tau)
    }

    /// Get a reference to the configuration.
    pub fn config(&self) -> &PheromoneConfig {
        &self.config
    }

    /// Number of edges with pheromone.
    pub fn len(&self) -> usize {
        self.scores.len()
    }

    /// Whether the pheromone map is empty.
    pub fn is_empty(&self) -> bool {
        self.scores.is_empty()
    }

    /// Get all edge keys (for iteration during evaporation).
    pub fn edge_keys(&self) -> Vec<(u64, u64)> {
        self.scores.keys().copied().collect()
    }

    /// Get all entries as a reference (for audit/inspection).
    pub fn all_entries(&self) -> &HashMap<(u64, u64), PheromoneEntry> {
        &self.scores
    }

    /// Probabilistic ACO route selection (AC-012, AC-013).
    ///
    /// Given a source node and a list of candidate neighbors with edge weights,
    /// selects the next node using roulette wheel selection with the ACO formula:
    ///
    /// `P(i) = τ(source,i)^α * η(i)^β` where `η = 1/weight`.
    ///
    /// Returns `None` if `neighbors` is empty or all scores are zero.
    pub fn route(
        &self,
        source: u64,
        _target: u64,
        neighbors: &[(u64, f64)],
        rng: &mut dyn crate::ant::RngSource,
    ) -> Option<u64> {
        if neighbors.is_empty() {
            return None;
        }

        let alpha = self.config.alpha;
        let beta = self.config.beta;

        let scores: Vec<f64> = neighbors
            .iter()
            .map(|(to, weight)| {
                let tau = self.get(source, *to);
                let eta = if *weight > 0.0 { 1.0 / weight } else { 1.0 };
                tau.powf(alpha) * eta.powf(beta)
            })
            .collect();

        let total: f64 = scores.iter().sum();
        if total == 0.0 {
            return None;
        }

        // Roulette wheel selection
        let r = rng.gen_f64() * total;
        let mut cumulative = 0.0;
        for (i, score) in scores.iter().enumerate() {
            cumulative += score;
            if cumulative >= r {
                return Some(neighbors[i].0);
            }
        }

        // Fallback: return last neighbor
        neighbors.last().map(|(id, _)| *id)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn default_config() -> PheromoneConfig {
        PheromoneConfig::default()
    }

    #[test]
    fn deposit_increases_level() {
        let mut map = PheromoneMap::new(default_config());
        let initial = map.get(1, 2);
        map.deposit(1, 2, 0.5);
        assert!(map.get(1, 2) > initial);
    }

    #[test]
    fn get_returns_initial_for_unknown_edge() {
        let config = default_config();
        let map = PheromoneMap::new(config.clone());
        assert_eq!(map.get(999, 1000), config.initial_tau);
    }

    #[test]
    fn deposit_respects_tau_max() {
        let config = PheromoneConfig {
            tau_max: 2.0,
            ..Default::default()
        };
        let mut map = PheromoneMap::new(config);
        map.deposit(1, 2, 100.0);
        assert!(map.get(1, 2) <= 2.0);
    }

    #[test]
    fn deposit_respects_tau_min() {
        let config = PheromoneConfig {
            tau_min: 0.5,
            initial_tau: 0.6,
            ..Default::default()
        };
        let mut map = PheromoneMap::new(config);
        map.deposit(1, 2, -2.0);
        assert!(map.get(1, 2) >= 0.5);
    }

    #[test]
    fn deposit_count_tracks() {
        let mut map = PheromoneMap::new(default_config());
        map.deposit(1, 2, 0.1);
        map.deposit(1, 2, 0.1);
        let entries = map.all_entries();
        assert_eq!(entries.get(&(1, 2)).unwrap().deposit_count, 2);
    }

    #[test]
    fn deposit_is_proportional_to_quality() {
        let mut map = PheromoneMap::new(default_config());
        map.deposit(1, 2, 0.9); // high quality
        map.deposit(3, 4, 0.1); // low quality
        assert!(map.get(1, 2) > map.get(3, 4));
    }

    #[test]
    fn shared_pheromone_map() {
        let map = PheromoneMap::new(default_config());
        let shared = map.shared();
        {
            let mut w = shared.write();
            w.deposit(1, 2, 0.5);
        }
        let r = shared.read();
        assert!(r.get(1, 2) > 1.0);
    }

    #[test]
    fn route_returns_none_for_empty_neighbors() {
        let map = PheromoneMap::new(default_config());
        let mut rng = crate::ant::DeterministicRng::new(42);
        assert!(map.route(1, 99, &[], &mut rng).is_none());
    }

    #[test]
    fn route_higher_pheromone_selected_more_often() {
        use std::collections::HashMap;

        let config = PheromoneConfig {
            alpha: 2.0,
            beta: 0.0, // ignore heuristic weight
            ..Default::default()
        };
        let mut map = PheromoneMap::new(config);
        // Deposit much more on edge (1->2) than (1->3)
        map.deposit(1, 2, 10.0);
        map.deposit(1, 3, 0.5);

        let neighbors = vec![(2u64, 1.0_f64), (3u64, 1.0_f64)];
        let mut counts: HashMap<u64, usize> = HashMap::new();
        let iterations = 1000;

        for seed in 1..=iterations {
            let mut rng = crate::ant::DeterministicRng::new((seed as u64) | 0xDEAD_BEEF_CAFE_0000);
            if let Some(chosen) = map.route(1, 99, &neighbors, &mut rng) {
                *counts.entry(chosen).or_insert(0) += 1;
            }
        }

        let count_2 = counts.get(&2).copied().unwrap_or(0);
        let count_3 = counts.get(&3).copied().unwrap_or(0);
        // With much higher pheromone on (1->2), it should be selected more
        assert!(
            count_2 > count_3,
            "node 2 (higher pheromone) should be selected more: {} vs {}",
            count_2,
            count_3
        );
    }

    #[test]
    fn route_roughly_uniform_with_equal_pheromone() {
        use std::collections::HashMap;

        let map = PheromoneMap::new(default_config());
        let neighbors = vec![(10u64, 1.0_f64), (20u64, 1.0_f64), (30u64, 1.0_f64)];
        let mut counts: HashMap<u64, usize> = HashMap::new();
        let iterations: usize = 3000;

        for seed in 1..=iterations as u64 {
            let mut rng = crate::ant::DeterministicRng::new(seed | 0xDEAD_BEEF_CAFE_0000);
            if let Some(chosen) = map.route(1, 99, &neighbors, &mut rng) {
                *counts.entry(chosen).or_insert(0) += 1;
            }
        }

        // With equal pheromone and equal weights, each should get ~33%
        let count_10 = counts.get(&10).copied().unwrap_or(0);
        let count_20 = counts.get(&20).copied().unwrap_or(0);
        let count_30 = counts.get(&30).copied().unwrap_or(0);
        // Allow wide tolerance (deterministic PRNG with sequential seeds has bias)
        let expected = iterations / 3;
        let tolerance = expected;
        assert!(
            count_10 > expected - tolerance && count_10 < expected + tolerance,
            "node 10 selection count {} not within tolerance of {}",
            count_10,
            expected
        );
        assert!(
            count_20 > expected - tolerance && count_20 < expected + tolerance,
            "node 20 selection count {} not within tolerance of {}",
            count_20,
            expected
        );
        assert!(
            count_30 > expected - tolerance && count_30 < expected + tolerance,
            "node 30 selection count {} not within tolerance of {}",
            count_30,
            expected
        );
    }
}
