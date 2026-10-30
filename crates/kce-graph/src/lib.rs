//! KineContext Engine — Semantic Graph Engine.
//!
//! Provides an adjacency-list graph with bidirectional weighted edges
//! and contextual expansion (BFS up to configurable depth).
//!
//! # Example
//!
//! ```
//! use kce_graph::graph::SemanticGraph;
//!
//! let mut graph = SemanticGraph::new();
//! graph.add_edge(1, 2, 0.85);
//! graph.add_edge(1, 3, 0.72);
//! let neighbors = graph.neighbors(1);
//! assert_eq!(neighbors.len(), 2);
//! ```

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]

pub mod graph;
