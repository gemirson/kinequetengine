//! Pheromone evaporation — temporal decay of pheromone trails.
//!
//! Evaporation prevents "toxic memory" where old paths dominate forever.
//! Formula: `tau(t+1) = max(tau_min, (1 - rho) * tau(t))`

use crate::pheromone::PheromoneMap;
use serde::{Deserialize, Serialize};

/// Configuration for evaporation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaporationConfig {
    /// Evaporation rate (ρ): fraction of pheromone removed per cycle.
    /// Must be in (0.0, 1.0).  Default 0.1.
    pub rho: f64,
}

impl Default for EvaporationConfig {
    fn default() -> Self {
        Self { rho: 0.1 }
    }
}

/// Metrics from an evaporation cycle.
#[derive(Debug, Clone, Default)]
pub struct EvaporationMetrics {
    /// Number of edges processed.
    pub edges_processed: usize,
    /// Total decay across all edges.
    pub total_decay: f64,
    /// Number of edges that hit tau_min floor.
    pub edges_at_tau_min: usize,
}

/// Evaporation engine.
pub struct EvaporationEngine {
    config: EvaporationConfig,
}

impl EvaporationEngine {
    /// Create a new evaporation engine.
    ///
    /// Returns `Err` if rate is not in (0.0, 1.0).
    pub fn new(config: EvaporationConfig) -> Result<Self, String> {
        if config.rho <= 0.0 || config.rho >= 1.0 {
            return Err(format!(
                "evaporation rate must be in (0.0, 1.0), got {}",
                config.rho
            ));
        }
        Ok(Self { config })
    }

    /// Apply evaporation to all edges in the pheromone map.
    ///
    /// Returns metrics about the evaporation cycle.
    pub fn evaporate_all(&self, map: &mut PheromoneMap) -> EvaporationMetrics {
        let rho = self.config.rho;
        let tau_min = map.config().tau_min;
        let mut metrics = EvaporationMetrics::default();

        // Iterate over all edge keys and apply evaporation
        let keys: Vec<(u64, u64)> = map.edge_keys();
        for (from, to) in keys {
            let old_level = map.get(from, to);
            let new_level = (old_level * (1.0 - rho)).max(tau_min);
            let decay = old_level - new_level;
            if decay.abs() > 1e-10 {
                // Apply as a negative deposit
                map.deposit(from, to, -decay);
            }
            metrics.total_decay += decay;
            metrics.edges_processed += 1;
            if (new_level - tau_min).abs() < 1e-10 {
                metrics.edges_at_tau_min += 1;
            }
        }

        metrics
    }

    /// Apply evaporation to a single edge.
    pub fn evaporate_edge(&self, map: &mut PheromoneMap, from: u64, to: u64) {
        let rho = self.config.rho;
        let tau_min = map.config().tau_min;
        let current = map.get(from, to);
        let new_level = (current * (1.0 - rho)).max(tau_min);
        let delta = new_level - current;
        if delta.abs() > 1e-10 {
            map.deposit(from, to, delta);
        }
    }

    /// Get the evaporation rate.
    pub fn rho(&self) -> f64 {
        self.config.rho
    }
}

impl Default for EvaporationEngine {
    fn default() -> Self {
        // SAFETY: 0.1 is in (0.0, 1.0)
        Self::new(EvaporationConfig::default()).expect("default rho is valid")
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::pheromone::PheromoneConfig;

    #[test]
    fn evaporation_reduces_level() {
        let mut map = PheromoneMap::new(PheromoneConfig::default());
        map.deposit(1, 2, 2.0);
        let before = map.get(1, 2);
        let evap = EvaporationEngine::default();
        evap.evaporate_all(&mut map);
        assert!(map.get(1, 2) < before);
    }

    #[test]
    fn evaporation_respects_tau_min() {
        let config = PheromoneConfig {
            tau_min: 0.5,
            initial_tau: 0.6,
            ..Default::default()
        };
        let mut map = PheromoneMap::new(config);
        let evap = EvaporationEngine::default();
        for _ in 0..100 {
            evap.evaporate_all(&mut map);
        }
        assert!(map.get(1, 2) >= 0.5);
    }

    #[test]
    fn evaporation_preserves_ordering() {
        let mut map = PheromoneMap::new(PheromoneConfig::default());
        map.deposit(1, 2, 5.0);
        map.deposit(3, 4, 1.0);
        let evap = EvaporationEngine::default();
        evap.evaporate_all(&mut map);
        assert!(map.get(1, 2) > map.get(3, 4));
    }

    #[test]
    fn evaporation_returns_metrics() {
        let mut map = PheromoneMap::new(PheromoneConfig::default());
        map.deposit(1, 2, 1.0);
        map.deposit(3, 4, 2.0);
        let evap = EvaporationEngine::default();
        let metrics = evap.evaporate_all(&mut map);
        assert_eq!(metrics.edges_processed, 2);
        assert!(metrics.total_decay > 0.0);
    }

    #[test]
    fn invalid_rho_rejected() {
        let result = EvaporationEngine::new(EvaporationConfig { rho: 0.0 });
        assert!(result.is_err());
        let result = EvaporationEngine::new(EvaporationConfig { rho: 1.0 });
        assert!(result.is_err());
    }

    #[test]
    fn evaporate_edge_single() {
        let mut map = PheromoneMap::new(PheromoneConfig::default());
        map.deposit(1, 2, 2.0);
        let before = map.get(1, 2);
        let evap = EvaporationEngine::default();
        evap.evaporate_edge(&mut map, 1, 2);
        assert!(map.get(1, 2) < before);
    }
}
