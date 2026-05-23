//! KineContext Engine — AIS (Artificial Immune System).
//!
//! Bio-inspired anomaly detection, antigen memory, response amplification,
//! self/non-self classification, adaptive mutation, and network regulation.
//!
//! # Modules
//!
//! - [`detection`] — Anomaly detection via threshold-based antigen matching.
//! - [`memory`] — Persistent antigen memory for known threat patterns.
//! - [`amplifier`] — Response amplification for critical contexts.
//! - [`classifier`] — Self/non-self classification of contexts.
//! - [`mutation`] — Adaptive mutation of embeddings for exploration.
//! - [`regulation`] — Network regulation (homeostasis) controller.

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]

pub mod amplifier;
pub mod classifier;
pub mod detection;
pub mod memory;
pub mod mutation;
pub mod regulation;
