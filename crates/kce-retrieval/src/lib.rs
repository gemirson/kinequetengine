//! KineContext Engine — Hybrid Retrieval Engine.
//!
//! Performs hybrid searches combining cosine similarity and prime (GCD) similarity.
//! Supports Rayon parallelism, early pruning, and deterministic ordering.
//!
//! # Example
//!
//! ```
//! use kce_retrieval::engine::{RetrievalEngine, Dataset};
//!
//! let engine = RetrievalEngine::with_defaults();
//! let mut ds = Dataset::new(2);
//! ds.push(1, vec![1.0, 0.0]).unwrap();
//! ds.push(2, vec![0.0, 1.0]).unwrap();
//! let results = engine.search(&[1.0, 0.0], &ds, 1).unwrap();
//! assert_eq!(results[0].id, 1);
//! ```

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]

pub mod engine;
