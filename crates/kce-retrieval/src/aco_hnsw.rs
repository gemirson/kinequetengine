//! ACO-HNSW — Bio-inspired vector indexing (FT-027).
//!
//! Combines Hierarchical Navigable Small World (HNSW) graphs with
//! Ant Colony Optimization (ACO) for dynamic, usage-driven routing.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use parking_lot::RwLock;
use rand::Rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use kce_core::error::KceError;
use kce_core::traits::DistanceMetric;
use kce_metrics::cosine::CosineMetric;

/// Configuration for the ACO-HNSW index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcoHnswConfig {
    /// ACO alpha parameter (pheromone weight).
    pub alpha: f64,
    /// ACO beta parameter (heuristic weight).
    pub beta: f64,
    /// Pheromone evaporation rate (rho).
    pub evaporation_rate: f64,
    /// Minimum pheromone level (tau_min).
    pub min_pheromone: f64,
    /// Initial pheromone level (tau_0).
    pub initial_pheromone: f64,
    /// Pheromone deposit amount (Q).
    pub deposit_amount: f64,
    /// Number of ants per query.
    pub num_ants: usize,
    /// Maximum search depth per ant.
    pub max_hops: usize,
}

impl Default for AcoHnswConfig {
    fn default() -> Self {
        Self {
            alpha: 1.2,
            beta: 2.0,
            evaporation_rate: 0.1,
            min_pheromone: 0.01,
            initial_pheromone: 1.0,
            num_ants: 8,
            max_hops: 15,
            deposit_amount: 0.5,
        }
    }
}

/// An edge in the ACO-HNSW graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcoEdge {
    /// Target node ID.
    pub to: u64,
    /// Static distance (computed at index time).
    pub distance: f64,
    /// Dynamic pheromone level (updated by queries).
    pub pheromone: f64,
}

/// A node in the ACO-HNSW graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcoNode {
    pub id: u64,
    pub vector: Vec<f64>,
    pub neighbors: Vec<AcoEdge>,
}

/// The ACO-HNSW Index.
pub struct AcoHnswIndex {
    nodes: RwLock<HashMap<u64, AcoNode>>,
    config: RwLock<AcoHnswConfig>,
    metric: Arc<dyn DistanceMetric>,
}

impl AcoHnswIndex {
    /// Create a new empty index.
    pub fn new(config: AcoHnswConfig) -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            config: RwLock::new(config),
            metric: Arc::new(CosineMetric::new()),
        }
    }

    /// Add a node to the index.
    /// (Simplified: adds to a flat graph for MVP, full HNSW layers in Iteration 1).
    pub fn add_node(&self, id: u64, vector: Vec<f64>, neighbor_ids: Vec<u64>) -> Result<(), KceError> {
        let mut nodes = self.nodes.write();
        let config = self.config.read();

        let mut neighbors = Vec::new();
        for nid in neighbor_ids {
            if let Some(target) = nodes.get(&nid) {
                let dist = self.metric.compute(&vector, &target.vector).map_err(|e| KceError::Config(e.to_string()))?;
                neighbors.push(AcoEdge {
                    to: nid,
                    distance: dist,
                    pheromone: config.initial_pheromone,
                });
            }
        }

        nodes.insert(id, AcoNode { id, vector, neighbors });
        Ok(())
    }

    /// Perform a search using ACO-driven traversal.
    pub fn search(&self, query_vector: &[f64], top_k: usize) -> Result<Vec<AcoSearchResult>, KceError> {
        let nodes = self.nodes.read();
        let config = self.config.read();
        
        if nodes.is_empty() {
            return Ok(vec![]);
        }

        // Start from a random node (or entry point in full HNSW)
        let entry_ids: Vec<u64> = nodes.keys().take(config.num_ants).copied().collect();
        
        // Launch ants in parallel
        let ant_results: Vec<AntPath> = (0..config.num_ants)
            .into_par_iter()
            .map(|i| {
                let start_id = entry_ids[i % entry_ids.len()];
                self.run_ant(start_id, query_vector, &nodes, &config)
            })
            .collect();

        // Aggregate results
        let mut unique_results: HashMap<u64, (f64, usize)> = HashMap::new();
        for path in &ant_results {
            if let Some(best_node_id) = path.best_node {
                let score = path.best_score;
                let entry = unique_results.entry(best_node_id).or_insert((score, 0));
                entry.1 += 1; // track visit count
            }
        }

        let mut results: Vec<AcoSearchResult> = unique_results
            .into_iter()
            .map(|(id, (score, _))| AcoSearchResult { id, score, hops: 0 }) // hops tracking per node can be added
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);

        // Reinforce paths for top results (FT-012)
        drop(nodes); // Release read lock before write
        self.reinforce_paths(&ant_results, &results, &config);

        Ok(results)
    }

    /// Single ant traversal logic.
    fn run_ant(&self, start_id: u64, query_vector: &[f64], nodes: &HashMap<u64, AcoNode>, config: &AcoHnswConfig) -> AntPath {
        let mut current_id = start_id;
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        let mut best_node = None;
        let mut best_score = -1.0;

        let mut rng = rand::thread_rng();

        for _ in 0..config.max_hops {
            path.push(current_id);
            visited.insert(current_id);

            let node = match nodes.get(&current_id) {
                Some(n) => n,
                None => break,
            };

            // Update best found
            let score = self.metric.compute(query_vector, &node.vector).unwrap_or(0.0);
            if score > best_score {
                best_score = score;
                best_node = Some(current_id);
            }

            // Probabilistic selection of next neighbor
            if node.neighbors.is_empty() { break; }

            let mut candidates = Vec::new();
            let mut total_prob = 0.0;

            for edge in &node.neighbors {
                if visited.contains(&edge.to) { continue; }
                
                // ηij = inverse distance similarity (approx score)
                let eta = 1.0 / (1.0 + edge.distance); 
                let prob = edge.pheromone.powf(config.alpha) * eta.powf(config.beta);
                candidates.push((edge.to, prob));
                total_prob += prob;
            }

            if candidates.is_empty() || total_prob == 0.0 { break; }

            // Roulette wheel
            let r = rng.gen::<f64>() * total_prob;
            let mut cumulative = 0.0;
            let mut next_id = candidates.last().unwrap().0;
            for (id, prob) in candidates {
                cumulative += prob;
                if cumulative >= r {
                    next_id = id;
                    break;
                }
            }
            current_id = next_id;
        }

        AntPath {
            nodes: path,
            best_node,
            best_score,
        }
    }

    /// Reinforce paths taken by successful ants.
    fn reinforce_paths(&self, ant_paths: &[AntPath], top_results: &[AcoSearchResult], config: &AcoHnswConfig) {
        let top_ids: HashSet<u64> = top_results.iter().map(|r| r.id).collect();
        let mut nodes = self.nodes.write();

        for path in ant_paths {
            if let Some(best_id) = path.best_node {
                if top_ids.contains(&best_id) {
                    // This ant found a good result, reinforce its path
                    for i in 0..path.nodes.len() - 1 {
                        let from = path.nodes[i];
                        let to = path.nodes[i+1];
                        if let Some(node) = nodes.get_mut(&from) {
                            for edge in &mut node.neighbors {
                                if edge.to == to {
                                    edge.pheromone = (edge.pheromone + config.deposit_amount).min(10.0);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Evaporate pheromones across the entire graph.
    pub fn evaporate(&self) {
        let mut nodes = self.nodes.write();
        let config = self.config.read();
        let rho = config.evaporation_rate;
        let tau_min = config.min_pheromone;

        for node in nodes.values_mut() {
            for edge in &mut node.neighbors {
                edge.pheromone = (edge.pheromone * (1.0 - rho)).max(tau_min);
            }
        }
    }
}

/// Result of an ACO search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcoSearchResult {
    pub id: u64,
    pub score: f64,
    pub hops: usize,
}

/// Path taken by an ant.
struct AntPath {
    nodes: Vec<u64>,
    best_node: Option<u64>,
    best_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aco_hnsw_basic_search() {
        let config = AcoHnswConfig::default();
        let index = AcoHnswIndex::new(config);

        // Create a simple chain 1 -> 2 -> 3
        index.add_node(1, vec![1.0, 0.0], vec![2]).unwrap();
        index.add_node(2, vec![0.5, 0.5], vec![1, 3]).unwrap();
        index.add_node(3, vec![0.0, 1.0], vec![2]).unwrap();

        let results = index.search(&[0.1, 0.9], 1).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].id, 3);
    }

    #[test]
    fn test_pheromone_reinforcement() {
        let config = AcoHnswConfig {
            num_ants: 1,
            max_hops: 5,
            deposit_amount: 1.0,
            ..Default::default()
        };
        let index = AcoHnswIndex::new(config);

        index.add_node(1, vec![1.0], vec![2]).unwrap();
        index.add_node(2, vec![1.1], vec![]).unwrap();

        // Initial pheromone is 1.0
        {
            let n1 = index.nodes.read();
            assert_eq!(n1.get(&1).unwrap().neighbors[0].pheromone, 1.0);
        }

        // Search reinforces path
        index.search(&[1.1], 1).unwrap();

        {
            let n1 = index.nodes.read();
            assert!(n1.get(&1).unwrap().neighbors[0].pheromone > 1.0);
        }
    }
}
