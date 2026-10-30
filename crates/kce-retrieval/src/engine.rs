//! Hybrid Retrieval Engine for KCE.
//!
//! Combines cosine similarity and prime (GCD) similarity with configurable
//! weights.  Supports early pruning and deterministic ordering.

use kce_core::error::KceError;
use kce_core::traits::DistanceMetric;
use kce_core::types::SearchResult;
use kce_metrics::cosine::CosineMetric;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Configuration for the retrieval engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalConfig {
    /// Weight for cosine similarity (0.0 .. 1.0).
    pub cosine_weight: f64,
    /// Minimum score threshold for early pruning.
    pub threshold: f64,
    /// Maximum number of threads for parallel search.
    pub max_threads: usize,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            cosine_weight: 1.0,
            threshold: 0.1,
            max_threads: 4,
        }
    }
}

/// A dataset of vectors with associated ids.
#[derive(Debug, Clone)]
pub struct Dataset {
    /// Unique ids for each vector.
    pub ids: Vec<u64>,
    /// Vectors stored as flat f64 slices (each row is `dimension` elements).
    pub vectors: Vec<Vec<f64>>,
    /// Dimension of each vector.
    pub dimension: usize,
}

impl Dataset {
    /// Create a new empty dataset with the given dimension.
    pub fn new(dimension: usize) -> Self {
        Self {
            ids: Vec::new(),
            vectors: Vec::new(),
            dimension,
        }
    }

    /// Add a vector to the dataset.
    pub fn push(&mut self, id: u64, vector: Vec<f64>) -> Result<(), KceError> {
        if vector.is_empty() {
            return Err(KceError::EmptyVector);
        }
        if vector.len() != self.dimension {
            return Err(KceError::DimensionMismatch {
                query: vector.len(),
                dataset: self.dimension,
            });
        }
        self.ids.push(id);
        self.vectors.push(vector);
        Ok(())
    }

    /// Number of vectors in the dataset.
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Whether the dataset is empty.
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

/// Hybrid retrieval engine.
#[derive(Debug)]
pub struct RetrievalEngine {
    config: RetrievalConfig,
    cosine: CosineMetric,
}

impl RetrievalEngine {
    /// Create a new retrieval engine with the given configuration.
    pub fn new(config: RetrievalConfig) -> Self {
        Self {
            config,
            cosine: CosineMetric::new(),
        }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(RetrievalConfig::default())
    }

    /// Search the dataset for the top-k most similar vectors to the query.
    pub fn search(
        &self,
        query: &[f64],
        dataset: &Dataset,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, KceError> {
        if query.is_empty() {
            return Err(KceError::EmptyVector);
        }
        if query.len() != dataset.dimension {
            return Err(KceError::DimensionMismatch {
                query: query.len(),
                dataset: dataset.dimension,
            });
        }
        if top_k == 0 {
            return Err(KceError::Validation("top_k must be > 0".into()));
        }

        if query.iter().any(|v| !v.is_finite()) {
            return Err(KceError::Validation("query contains NaN/Inf".into()));
        }

        // Score all vectors in parallel via Rayon (FT-001 AC-016).
        let threshold = self.config.threshold;
        let cw = self.config.cosine_weight;
        let cosine = &self.cosine;

        let mut results: Vec<SearchResult> = dataset
            .ids
            .par_iter()
            .zip(dataset.vectors.par_iter())
            .enumerate()
            .filter_map(|(_i, (id, vector))| {
                let cosine_score = cosine.compute(query, vector).ok()?;
                let combined = cw * cosine_score;
                if combined < threshold {
                    return None;
                }
                Some(SearchResult {
                    id: *id,
                    score: combined,
                    cosine_score,
                })
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.id.cmp(&b.id))
        });

        results.truncate(top_k);
        Ok(results)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn sample_dataset() -> Dataset {
        let mut ds = Dataset::new(2);
        ds.push(1, vec![1.0, 0.0]).expect("push");
        ds.push(2, vec![0.0, 1.0]).expect("push");
        ds.push(3, vec![1.0, 1.0]).expect("push");
        ds
    }

    #[test]
    fn search_returns_top_k() {
        let engine = RetrievalEngine::with_defaults();
        let ds = sample_dataset();
        let results = engine.search(&[1.0, 0.0], &ds, 2).expect("search");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 1);
    }

    #[test]
    fn search_deterministic_order() {
        let engine = RetrievalEngine::with_defaults();
        let ds = sample_dataset();
        let r1 = engine.search(&[0.5, 0.5], &ds, 10).expect("search");
        let r2 = engine.search(&[0.5, 0.5], &ds, 10).expect("search");
        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a.id, b.id);
        }
    }

    #[test]
    fn search_dimension_mismatch() {
        let engine = RetrievalEngine::with_defaults();
        let ds = sample_dataset();
        assert!(engine.search(&[1.0], &ds, 5).is_err());
    }

    #[test]
    fn search_empty_query() {
        let engine = RetrievalEngine::with_defaults();
        let ds = sample_dataset();
        assert!(engine.search(&[], &ds, 5).is_err());
    }

    #[test]
    fn search_zero_k() {
        let engine = RetrievalEngine::with_defaults();
        let ds = sample_dataset();
        assert!(engine.search(&[1.0, 0.0], &ds, 0).is_err());
    }

    #[test]
    fn search_top_k_greater_than_dataset() {
        let engine = RetrievalEngine::with_defaults();
        let ds = sample_dataset();
        let results = engine.search(&[1.0, 0.0], &ds, 100).expect("search");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn search_prunes_low_scores() {
        let config = RetrievalConfig {
            threshold: 0.99,
            ..Default::default()
        };
        let engine = RetrievalEngine::new(config);
        let ds = sample_dataset();
        let results = engine.search(&[1.0, 0.0], &ds, 10).expect("search");
        assert!(!results.is_empty());
        assert!(results.iter().all(|r| r.score >= 0.99));
    }
}
