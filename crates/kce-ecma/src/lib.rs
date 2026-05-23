//! KineContext Engine — ECMA (Embryological Cognitive Memory Architecture).
//!
//! Implements knowledge node lifecycle evolution: Stem → Progenitor → Specialized → Apoptosis.
//! Nodes evolve based on usage, entropy, and connections.
//!
//! # Example
//!
//! ```
//! use kce_ecma::state_machine::{EcmaConfig, transition, maturity};
//! use kce_core::types::{EcmaNode, NodeState};
//!
//! let config = EcmaConfig::default();
//! let mut node = EcmaNode {
//!     id: 42,
//!     state: NodeState::Stem,
//!     usage_count: 5,
//!     entropy: 0.3,
//!     connections: 3,
//!     maturity: 0.0,
//! };
//! transition(&mut node, &config).unwrap();
//! assert_eq!(node.state, NodeState::Progenitor);
//! ```

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]

pub mod engine;
pub mod state_machine;
