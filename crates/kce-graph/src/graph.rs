//! Semantic graph engine.
//!
//! Manages an adjacency-list graph where nodes are connected by weighted edges.
//! Supports `add_edge`, `remove_edge`, `neighbors`, and `expand` for contextual
//! expansion up to a configurable depth.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// A weighted edge in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Source node.
    pub from: u64,
    /// Target node.
    pub to: u64,
    /// Edge weight (0.0 .. 1.0).
    pub weight: f64,
}

/// Result of a graph expansion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpansionResult {
    /// Expanded node ids (including seed nodes, deduplicated).
    pub nodes: Vec<u64>,
    /// Edges traversed during expansion.
    pub edges: Vec<Edge>,
    /// Maximum depth reached.
    pub depth_reached: u32,
}

/// Adjacency-list semantic graph.
///
/// Thread-safe for concurrent reads via `Arc<parking_lot::RwLock<SemanticGraph>>`.
#[derive(Debug, Default)]
pub struct SemanticGraph {
    /// Adjacency list: node_id -> Vec<(neighbor_id, weight)>.
    adjacency: BTreeMap<u64, Vec<(u64, f64)>>,
}

impl SemanticGraph {
    /// Create a new empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a bidirectional edge between two nodes.
    ///
    /// Weight must be in 0.0..=1.0.  Idempotent: duplicate edges are ignored.
    pub fn add_edge(&mut self, a: u64, b: u64, weight: f64) {
        self.add_directed(a, b, weight);
        self.add_directed(b, a, weight);
    }

    /// Remove a bidirectional edge.
    ///
    /// Returns silently if the edge does not exist (idempotent).
    pub fn remove_edge(&mut self, a: u64, b: u64) {
        if let Some(neighbors) = self.adjacency.get_mut(&a) {
            neighbors.retain(|(id, _)| *id != b);
        }
        if let Some(neighbors) = self.adjacency.get_mut(&b) {
            neighbors.retain(|(id, _)| *id != a);
        }
    }

    /// Get neighbors of a node with their edge weights.
    pub fn neighbors(&self, node_id: u64) -> Vec<(u64, f64)> {
        self.adjacency.get(&node_id).cloned().unwrap_or_default()
    }

    /// Expand from seed nodes up to `depth` hops.
    ///
    /// Returns at most `2x` the number of seed nodes (per spec AC-013),
    /// deduplicated, plus the edges traversed.
    /// Deterministic: same input always produces same output.
    pub fn expand(&self, seeds: &[u64], depth: u32) -> ExpansionResult {
        // AC-013: auto limit to 2x seed count
        let max_nodes = seeds.len() * 2;
        self.expand_with_limit(seeds, depth, max_nodes)
    }

    /// Expand from seed nodes with an explicit max_nodes limit.
    pub fn expand_with_limit(
        &self,
        seeds: &[u64],
        depth: u32,
        max_nodes: usize,
    ) -> ExpansionResult {
        let mut visited: BTreeSet<u64> = BTreeSet::new();
        let mut edges: Vec<Edge> = Vec::new();
        let mut queue: VecDeque<(u64, u32)> = VecDeque::new();

        for &seed in seeds {
            if visited.insert(seed) {
                queue.push_back((seed, 0));
            }
        }

        let mut max_depth_reached = 0u32;

        while let Some((node, d)) = queue.pop_front() {
            if d >= depth || visited.len() >= max_nodes {
                continue;
            }

            for (neighbor, weight) in self.neighbors(node) {
                max_depth_reached = max_depth_reached.max(d + 1);
                if visited.len() < max_nodes && visited.insert(neighbor) {
                    edges.push(Edge {
                        from: node,
                        to: neighbor,
                        weight,
                    });
                    queue.push_back((neighbor, d + 1));
                }
            }
        }

        ExpansionResult {
            nodes: visited.into_iter().collect(),
            edges,
            depth_reached: max_depth_reached,
        }
    }

    /// Number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.adjacency.len()
    }

    /// Total number of edges (directed count).
    pub fn edge_count(&self) -> usize {
        self.adjacency.values().map(|v| v.len()).sum()
    }

    /// Add a directed edge (internal helper).
    fn add_directed(&mut self, from: u64, to: u64, weight: f64) {
        let neighbors = self.adjacency.entry(from).or_default();
        if !neighbors.iter().any(|(id, _)| *id == to) {
            neighbors.push((to, weight));
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn add_edge_bidirectional() {
        let mut g = SemanticGraph::new();
        g.add_edge(1, 2, 0.5);
        let n1 = g.neighbors(1);
        let n2 = g.neighbors(2);
        assert_eq!(n1.len(), 1);
        assert_eq!(n1[0].0, 2);
        assert_eq!(n2.len(), 1);
        assert_eq!(n2[0].0, 1);
    }

    #[test]
    fn neighbors_empty_node() {
        let g = SemanticGraph::new();
        assert!(g.neighbors(99).is_empty());
    }

    #[test]
    fn expand_depth_1() {
        let mut g = SemanticGraph::new();
        g.add_edge(1, 2, 0.5);
        g.add_edge(1, 3, 0.8);
        // With 1 seed, auto 2x limit = 2 nodes
        let result = g.expand(&[1], 1);
        assert!(result.nodes.contains(&1));
        // Use expand_with_limit for full expansion
        let result = g.expand_with_limit(&[1], 1, 100);
        assert!(result.nodes.contains(&1));
        assert!(result.nodes.contains(&2));
        assert!(result.nodes.contains(&3));
    }

    #[test]
    fn expand_no_duplicates() {
        let mut g = SemanticGraph::new();
        g.add_edge(1, 2, 0.5);
        g.add_edge(2, 3, 0.5);
        g.add_edge(3, 1, 0.5);
        let result = g.expand(&[1], 10);
        let unique: BTreeSet<u64> = result.nodes.iter().copied().collect();
        assert_eq!(unique.len(), result.nodes.len());
    }

    #[test]
    fn expand_auto_2x_limit() {
        // AC-013: expand returns <= 2x seed count
        let mut g = SemanticGraph::new();
        for i in 0..100u64 {
            g.add_edge(0, i + 1, 0.5);
        }
        let result = g.expand(&[0], 2);
        assert!(result.nodes.len() <= 2); // 1 seed * 2 = 2
    }

    #[test]
    fn expand_with_limit_respects_max() {
        let mut g = SemanticGraph::new();
        for i in 0..100u64 {
            g.add_edge(0, i + 1, 0.5);
        }
        let result = g.expand_with_limit(&[0], 2, 5);
        assert!(result.nodes.len() <= 5);
    }

    #[test]
    fn remove_edge() {
        let mut g = SemanticGraph::new();
        g.add_edge(1, 2, 0.5);
        g.remove_edge(1, 2);
        assert!(g.neighbors(1).is_empty());
        assert!(g.neighbors(2).is_empty());
    }

    #[test]
    fn expand_deterministic() {
        let mut g = SemanticGraph::new();
        g.add_edge(1, 2, 0.5);
        g.add_edge(1, 3, 0.8);
        g.add_edge(2, 4, 0.3);
        let r1 = g.expand(&[1], 2);
        let r2 = g.expand(&[1], 2);
        assert_eq!(r1.nodes, r2.nodes);
    }

    #[test]
    fn node_count() {
        let mut g = SemanticGraph::new();
        g.add_edge(1, 2, 0.5);
        g.add_edge(3, 4, 0.5);
        assert_eq!(g.node_count(), 4);
    }

    #[test]
    fn edge_count() {
        let mut g = SemanticGraph::new();
        g.add_edge(1, 2, 0.5);
        // Bidirectional = 2 directed edges
        assert_eq!(g.edge_count(), 2);
    }

    #[test]
    fn stress_100k_nodes() {
        let mut g = SemanticGraph::new();
        for i in 0..100_000u64 {
            g.add_edge(i, i + 1, 0.5);
        }
        assert_eq!(g.node_count(), 100_001);
        let result = g.expand(&[0], 3);
        assert!(!result.nodes.is_empty());
    }

    #[test]
    fn expand_multiple_seeds() {
        let mut g = SemanticGraph::new();
        g.add_edge(1, 2, 0.5);
        g.add_edge(10, 20, 0.8);
        let result = g.expand(&[1, 10], 1);
        // 2 seeds * 2 = 4 max, but we have 4 unique nodes + 2 seeds = 6
        // Actually: seeds = [1, 10], max_nodes = 4
        // BFS: 1 -> neighbors(1)=[2], 10 -> neighbors(10)=[20]
        // visited = {1, 10, 2, 20} = 4 nodes
        assert!(result.nodes.contains(&1));
        assert!(result.nodes.contains(&10));
        assert!(result.nodes.contains(&2));
        assert!(result.nodes.contains(&20));
    }
}
