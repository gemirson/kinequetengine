//! KineContext Engine — ACO (Ant Colony Optimization).
//!
//! Implements pheromone-based routing for context exploration.
//!
//! # Modules
//!
//! - [`pheromone`] — Pheromone map with deposit and decay.
//! - [`evaporation`] — Temporal evaporation of pheromone trails.
//! - [`ant`] — Ant explorer for parallel path discovery.
//! - [`colony`] — Colony optimization with convergence detection.

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]

pub mod ant;
pub mod colony;
pub mod evaporation;
pub mod pheromone;
