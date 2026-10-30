//! KineContext Engine — Core Library
//!
//! This crate provides the shared types, traits, and error types used by all
//! other KCE modules.  It is the foundation layer — every other crate depends
//! on `kce-core`, but `kce-core` depends on nothing internal.
//!
//! # Modules
//!
//! - [`error`] — System-wide error taxonomy (`KceError`, `StorageError`, `MetricError`).
//! - [`types`] — Shared data structures (context I/O, health, search results, mRNA, etc.).
//! - [`traits`] — Inter-module contracts (`ContextEngine`, `StorageBackend`, `DistanceMetric`).

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]

pub mod error;
pub mod traits;
pub mod types;
