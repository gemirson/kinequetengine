//! Distance and similarity metrics for the KineContext Engine.
//!
//! Provides implementations of [`DistanceMetric`](kce_core::traits::DistanceMetric)
//! for cosine, prime, Wasserstein, and Sinkhorn distances.

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]

pub mod cosine;
pub mod latency;
