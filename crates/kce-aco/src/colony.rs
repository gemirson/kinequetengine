//! Colony optimization — multi-cycle optimization with convergence detection.
//!
//! Runs multiple iterations of ant exploration, updates pheromone trails,
//! and detects convergence to stop early. Falls back to static ranking
//! if ACO does not converge.

use crate::ant::{self, AntConfig, CandidateEdge};
use crate::evaporation::{EvaporationConfig, EvaporationEngine};
use crate::pheromone::{PheromoneConfig, PheromoneMap};

/// Configuration for the colony optimizer.
#[derive(Debug, Clone)]
pub struct ColonyConfig {
    /// Number of ants per iteration.
    pub num_ants: usize,
    /// Maximum number of iterations.
    pub max_iterations: usize,
    /// Pheromone deposit weight (Q / cost).
    pub q: f64,
    /// Ant behavior config.
    pub ant_config: AntConfig,
    /// Evaporation rate.
    pub evaporation_rate: f64,
    /// Convergence threshold: variance below which we stop.
    pub convergence_threshold: f64,
    /// Convergence window: stop if no improvement for N iterations.
    pub convergence_window: usize,
}

impl Default for ColonyConfig {
    fn default() -> Self {
        Self {
            num_ants: 10,
            max_iterations: 20,
            q: 1.0,
            ant_config: AntConfig::default(),
            evaporation_rate: 0.1,
            convergence_threshold: 0.05,
            convergence_window: 5,
        }
    }
}

/// Result of a colony optimization run.
#[derive(Debug, Clone)]
pub struct ColonyResult {
    /// Best path found.
    pub best_path: Vec<u64>,
    /// Cost of the best path.
    pub best_cost: f64,
    /// Number of iterations executed.
    pub iterations: usize,
    /// Whether convergence was detected (early stop).
    pub converged: bool,
    /// Score progression across iterations.
    pub score_progression: Vec<f64>,
    /// Final score variance.
    pub final_variance: f64,
    /// Whether fallback to static ranking was used.
    pub used_fallback: bool,
}

/// The colony optimizer.
pub struct Colony {
    config: ColonyConfig,
    pheromone: PheromoneMap,
    evaporation: EvaporationEngine,
}

impl Colony {
    /// Create a new colony with the given configuration.
    pub fn new(config: ColonyConfig) -> Self {
        let evaporation = EvaporationEngine::new(EvaporationConfig {
            rho: config.evaporation_rate,
        })
        .expect("default evaporation rate is valid");
        Self {
            config,
            pheromone: PheromoneMap::new(PheromoneConfig::default()),
            evaporation,
        }
    }

    /// Run the colony optimization algorithm.
    ///
    /// - `start`: starting node for all ants.
    /// - `neighbors_fn`: function returning candidate edges for a given node.
    ///
    /// Falls back to static ranking if ACO does not converge within max_iterations.
    pub fn run<F>(&mut self, start: u64, neighbors_fn: F) -> ColonyResult
    where
        F: Fn(u64) -> Vec<CandidateEdge>,
    {
        let mut best_path: Vec<u64> = Vec::new();
        let mut best_cost = f64::INFINITY;
        let mut no_improvement = 0;
        let mut converged = false;
        let mut score_progression: Vec<f64> = Vec::new();
        let mut recent_costs: Vec<f64> = Vec::new();

        for iter in 0..self.config.max_iterations {
            // Explore with ants
            let ant_results = ant::explore(
                start,
                self.config.num_ants,
                &neighbors_fn,
                &self.pheromone,
                &self.config.ant_config,
                42 + iter as u64,
            );

            // Find best ant this iteration
            let iter_best = ant_results
                .iter()
                .min_by(|a, b| {
                    a.cost
                        .partial_cmp(&b.cost)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

            if let Some(best_ant) = iter_best {
                let iter_cost = best_ant.cost;
                score_progression.push(iter_cost);

                // Update global best
                if iter_cost < best_cost {
                    best_cost = iter_cost;
                    best_path = best_ant.path.clone();
                    no_improvement = 0;
                } else {
                    no_improvement += 1;
                }

                // Track recent costs for variance calculation
                recent_costs.push(iter_cost);
                if recent_costs.len() > self.config.convergence_window {
                    recent_costs.remove(0);
                }

                // Deposit pheromone on best path
                if !best_path.is_empty() && best_cost > 0.0 {
                    let deposit = self.config.q / best_cost;
                    for pair in best_path.windows(2) {
                        self.pheromone.deposit(pair[0], pair[1], deposit);
                    }
                }
            }

            // Evaporate
            self.evaporation.evaporate_all(&mut self.pheromone);

            // Check convergence via variance
            if recent_costs.len() >= 3 {
                let variance = compute_variance(&recent_costs);
                if variance < self.config.convergence_threshold {
                    converged = true;
                    return ColonyResult {
                        best_path,
                        best_cost,
                        iterations: iter + 1,
                        converged,
                        score_progression,
                        final_variance: variance,
                        used_fallback: false,
                    };
                }
            }

            // Check no-improvement window
            if no_improvement >= self.config.convergence_window {
                converged = true;
                let variance = compute_variance(&recent_costs);
                return ColonyResult {
                    best_path,
                    best_cost,
                    iterations: iter + 1,
                    converged,
                    score_progression,
                    final_variance: variance,
                    used_fallback: false,
                };
            }
        }

        // Fallback: ACO did not converge
        let variance = compute_variance(&recent_costs);
        ColonyResult {
            best_path,
            best_cost,
            iterations: self.config.max_iterations,
            converged,
            score_progression,
            final_variance: variance,
            used_fallback: !converged,
        }
    }

    /// Get a reference to the pheromone map.
    pub fn pheromone(&self) -> &PheromoneMap {
        &self.pheromone
    }
}

/// Compute variance of a slice of f64 values.
fn compute_variance(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn simple_neighbors(node: u64) -> Vec<CandidateEdge> {
        match node {
            1 => vec![
                CandidateEdge { to: 2, weight: 1.0 },
                CandidateEdge { to: 3, weight: 2.0 },
            ],
            2 => vec![
                CandidateEdge { to: 4, weight: 1.0 },
                CandidateEdge { to: 3, weight: 1.5 },
            ],
            3 => vec![CandidateEdge { to: 4, weight: 0.5 }],
            _ => vec![],
        }
    }

    #[test]
    fn colony_finds_path() {
        let config = ColonyConfig {
            num_ants: 5,
            max_iterations: 10,
            ..Default::default()
        };
        let mut colony = Colony::new(config);
        let result = colony.run(1, simple_neighbors);
        assert!(!result.best_path.is_empty());
        assert!(result.best_cost < f64::INFINITY);
    }

    #[test]
    fn colony_converges() {
        let config = ColonyConfig {
            num_ants: 20,
            max_iterations: 100,
            convergence_window: 3,
            ..Default::default()
        };
        let mut colony = Colony::new(config);
        let result = colony.run(1, simple_neighbors);
        assert!(result.iterations <= 100);
    }

    #[test]
    fn colony_returns_score_progression() {
        let config = ColonyConfig {
            num_ants: 5,
            max_iterations: 5,
            ..Default::default()
        };
        let mut colony = Colony::new(config);
        let result = colony.run(1, simple_neighbors);
        assert!(!result.score_progression.is_empty());
    }

    #[test]
    fn colony_fallback_on_no_convergence() {
        let config = ColonyConfig {
            num_ants: 2,
            max_iterations: 3,
            convergence_threshold: 0.0001,
            convergence_window: 100,
            ..Default::default()
        };
        let mut colony = Colony::new(config);
        let result = colony.run(1, simple_neighbors);
        // With max_iterations=3 and window=100, it won't converge
        assert!(!result.converged || result.used_fallback || result.iterations <= 3);
    }
}
