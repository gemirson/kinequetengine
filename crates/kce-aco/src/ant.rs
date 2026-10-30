//! Ant explorer — parallel path discovery via probabilistic exploration.
//!
//! Each ant traverses the graph choosing edges based on pheromone level
//! and a heuristic (edge weight). The probability of choosing edge (i,j):
//!
//! P(i,j) = τ(i,j)^α * η(i,j)^β / Σ [τ(i,k)^α * η(i,k)^β]
//!
//! where τ = pheromone, η = heuristic (1/distance), α/β = tuning params.

use std::collections::HashSet;

use crate::pheromone::PheromoneMap;

/// An ant that explores the graph from a starting node.
pub struct Ant {
    /// Current path: sequence of node IDs.
    path: Vec<u64>,
    /// Set of visited nodes (for dedup).
    visited: HashSet<u64>,
    /// Total path cost (sum of inverse weights).
    cost: f64,
}

/// Configuration for ant behavior.
#[derive(Debug, Clone)]
pub struct AntConfig {
    /// Pheromone influence (α).
    pub alpha: f64,
    /// Heuristic influence (β).
    pub beta: f64,
    /// Maximum path length before stopping.
    pub max_steps: usize,
    /// Per-ant timeout in milliseconds (0 = no timeout).
    pub timeout_ms: u64,
}

impl Default for AntConfig {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            beta: 2.0,
            max_steps: 20,
            timeout_ms: 50,
        }
    }
}

/// A candidate edge for ant exploration.
#[derive(Debug, Clone)]
pub struct CandidateEdge {
    /// Destination node.
    pub to: u64,
    /// Edge weight/heuristic value.
    pub weight: f64,
}

/// Result from a single ant's exploration.
#[derive(Debug, Clone)]
pub struct AntResult {
    /// Ant identifier.
    pub ant_id: usize,
    /// Path found.
    pub path: Vec<u64>,
    /// Total cost of the path.
    pub cost: f64,
    /// Latency of this ant's exploration in milliseconds.
    pub latency_ms: f64,
    /// The alpha/beta config used.
    pub alpha: f64,
    pub beta: f64,
}

impl Ant {
    /// Create a new ant starting at the given node.
    pub fn new(start: u64) -> Self {
        let mut visited = HashSet::new();
        visited.insert(start);
        Self {
            path: vec![start],
            visited,
            cost: 0.0,
        }
    }

    /// Get the ant's current path.
    pub fn path(&self) -> &[u64] {
        &self.path
    }

    /// Get the ant's total path cost.
    pub fn cost(&self) -> f64 {
        self.cost
    }

    /// Get the ant's current (last) node.
    pub fn current_node(&self) -> u64 {
        // SAFETY: path is never empty — always starts with the origin node.
        #[allow(clippy::unwrap_used)]
        *self.path.last().unwrap()
    }

    /// Check if a node has been visited.
    pub fn has_visited(&self, node: u64) -> bool {
        self.visited.contains(&node)
    }

    /// Explore one step: choose the next edge based on pheromone + heuristic.
    ///
    /// Returns `true` if a step was taken, `false` if no valid candidates.
    pub fn step(
        &mut self,
        candidates: &[CandidateEdge],
        pheromone: &PheromoneMap,
        config: &AntConfig,
        rng: &mut dyn RngSource,
    ) -> bool {
        if self.path.len() >= config.max_steps {
            return false;
        }

        let current = self.current_node();

        // Filter to unvisited candidates
        let valid: Vec<_> = candidates
            .iter()
            .filter(|c| !self.visited.contains(&c.to))
            .collect();

        if valid.is_empty() {
            return false;
        }

        // Compute probabilities: P = τ^α * η^β
        let scores: Vec<f64> = valid
            .iter()
            .map(|c| {
                let tau = pheromone.get(current, c.to);
                let eta = if c.weight > 0.0 { 1.0 / c.weight } else { 1.0 };
                tau.powf(config.alpha) * eta.powf(config.beta)
            })
            .collect();

        let total: f64 = scores.iter().sum();
        if total == 0.0 {
            return false;
        }

        // Roulette wheel selection
        let r = rng.gen_f64() * total;
        let mut cumulative = 0.0;
        for (i, score) in scores.iter().enumerate() {
            cumulative += score;
            if cumulative >= r {
                let chosen = valid[i];
                self.path.push(chosen.to);
                self.visited.insert(chosen.to);
                self.cost += chosen.weight;
                return true;
            }
        }

        // Fallback: choose last (valid is non-empty — checked above).
        if let Some(chosen) = valid.last() {
            self.path.push(chosen.to);
            self.visited.insert(chosen.to);
            self.cost += chosen.weight;
            true
        } else {
            false
        }
    }
}

/// Explore with multiple ants, each using a different heuristic.
///
/// Returns results from all ants. The caller selects the best.
/// Each ant gets a slightly different alpha/beta for diversity.
pub fn explore(
    start: u64,
    num_ants: usize,
    neighbors_fn: &dyn Fn(u64) -> Vec<CandidateEdge>,
    pheromone: &PheromoneMap,
    base_config: &AntConfig,
    seed: u64,
) -> Vec<AntResult> {
    let mut results = Vec::with_capacity(num_ants);

    for ant_id in 0..num_ants {
        let start_time = std::time::Instant::now();

        // Vary alpha/beta for diversity (AC-011)
        let alpha = base_config.alpha * (0.8 + 0.4 * (ant_id as f64 / num_ants.max(1) as f64));
        let beta = base_config.beta * (0.8 + 0.4 * (ant_id as f64 / num_ants.max(1) as f64));

        let config = AntConfig {
            alpha,
            beta,
            max_steps: base_config.max_steps,
            timeout_ms: base_config.timeout_ms,
        };

        let mut ant = Ant::new(start);
        let mut rng = DeterministicRng::new(seed.wrapping_add(ant_id as u64));

        loop {
            // Check timeout
            if config.timeout_ms > 0
                && start_time.elapsed().as_millis() as u64 >= config.timeout_ms
            {
                break;
            }

            let candidates = neighbors_fn(ant.current_node());
            if candidates.is_empty()
                || !ant.step(&candidates, pheromone, &config, &mut rng)
            {
                break;
            }
        }

        results.push(AntResult {
            ant_id,
            path: ant.path().to_vec(),
            cost: ant.cost(),
            latency_ms: start_time.elapsed().as_secs_f64() * 1000.0,
            alpha,
            beta,
        });
    }

    results
}

/// Trait abstracting random number generation for testability.
pub trait RngSource {
    /// Generate a random f64 in [0.0, 1.0).
    fn gen_f64(&mut self) -> f64;
}

/// Simple deterministic RNG for testing.
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    /// Create a new deterministic RNG with the given seed.
    /// Applies splitmix64 to decorrelate sequential seeds.
    pub fn new(seed: u64) -> Self {
        // splitmix64 finalizer to decorrelate sequential seeds
        let mut z = seed.wrapping_add(0x9e3779b97f4a7c15);
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^= z >> 31;
        // Ensure non-zero state (xorshift degenerates on 0)
        Self { state: if z == 0 { 1 } else { z } }
    }
}

impl RngSource for DeterministicRng {
    fn gen_f64(&mut self) -> f64 {
        // xorshift64
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        (self.state as f64) / (u64::MAX as f64)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::pheromone::PheromoneConfig;

    #[test]
    fn ant_starts_at_origin() {
        let ant = Ant::new(42);
        assert_eq!(ant.current_node(), 42);
        assert_eq!(ant.path(), &[42]);
    }

    #[test]
    fn ant_does_not_revisit() {
        let mut ant = Ant::new(1);
        let config = AntConfig::default();
        let pheromone = PheromoneMap::new(PheromoneConfig::default());
        let mut rng = DeterministicRng::new(12345);

        let candidates = vec![CandidateEdge { to: 1, weight: 0.5 }];
        assert!(!ant.step(&candidates, &pheromone, &config, &mut rng));
        assert_eq!(ant.path().len(), 1);
    }

    #[test]
    fn ant_takes_valid_step() {
        let mut ant = Ant::new(1);
        let config = AntConfig::default();
        let pheromone = PheromoneMap::new(PheromoneConfig::default());
        let mut rng = DeterministicRng::new(42);

        let candidates = vec![
            CandidateEdge { to: 2, weight: 0.5 },
            CandidateEdge { to: 3, weight: 0.8 },
        ];
        assert!(ant.step(&candidates, &pheromone, &config, &mut rng));
        assert_eq!(ant.path().len(), 2);
        assert!(ant.has_visited(ant.current_node()));
    }

    #[test]
    fn ant_respects_max_steps() {
        let mut ant = Ant::new(1);
        let config = AntConfig {
            max_steps: 2,
            ..Default::default()
        };
        let pheromone = PheromoneMap::new(PheromoneConfig::default());
        let mut rng = DeterministicRng::new(42);

        let candidates = vec![CandidateEdge { to: 2, weight: 0.5 }];
        assert!(ant.step(&candidates, &pheromone, &config, &mut rng));
        let candidates2 = vec![CandidateEdge { to: 3, weight: 0.5 }];
        assert!(!ant.step(&candidates2, &pheromone, &config, &mut rng));
    }

    #[test]
    fn explore_returns_results() {
        let pheromone = PheromoneMap::new(PheromoneConfig::default());
        let neighbors = |node: u64| -> Vec<CandidateEdge> {
            match node {
                1 => vec![
                    CandidateEdge { to: 2, weight: 1.0 },
                    CandidateEdge { to: 3, weight: 2.0 },
                ],
                _ => vec![],
            }
        };
        let config = AntConfig::default();
        let results = explore(1, 5, &neighbors, &pheromone, &config, 42);
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn explore_ants_have_different_heuristics() {
        let pheromone = PheromoneMap::new(PheromoneConfig::default());
        let neighbors = |node: u64| -> Vec<CandidateEdge> {
            match node {
                1 => vec![
                    CandidateEdge { to: 2, weight: 1.0 },
                    CandidateEdge { to: 3, weight: 2.0 },
                ],
                _ => vec![],
            }
        };
        let config = AntConfig::default();
        let results = explore(1, 5, &neighbors, &pheromone, &config, 42);
        // At least 2 ants should have different alpha values
        let unique_alphas: std::collections::HashSet<u64> = results
            .iter()
            .map(|r| (r.alpha * 1000.0) as u64)
            .collect();
        assert!(unique_alphas.len() >= 2);
    }
}
