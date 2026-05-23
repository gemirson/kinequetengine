//! KineContext Engine — Pipeline Orchestrator.
//!
//! Coordinates all KCE modules into a deterministic cognitive pipeline.
//! Each stage returns `Result<T, KceError>` — nothing assumes success.

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]

pub mod cache;
pub mod orchestrator;
