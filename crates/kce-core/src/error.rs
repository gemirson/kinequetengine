//! Error types for the KineContext Engine.
//!
//! All modules converge on [`KceError`] as the single error type.
//! Sub-errors ([`StorageError`], [`MetricError`]) are converted via `From`.

use thiserror::Error;

/// Top-level error type for the KCE system.
#[derive(Debug, Error)]
pub enum KceError {
    /// Storage subsystem error.
    #[error("storage: {0}")]
    Storage(#[from] StorageError),

    /// Query vector dimension does not match dataset dimension.
    #[error("retrieval: dimension mismatch (query={query}, dataset={dataset})")]
    DimensionMismatch { query: usize, dataset: usize },

    /// An empty vector was provided where a non-empty one is required.
    #[error("retrieval: empty vector")]
    EmptyVector,

    /// Metric computation error.
    #[error("metric: {0}")]
    Metric(#[from] MetricError),

    /// A pipeline stage failed.
    #[error("pipeline failed at stage '{stage}': {source}")]
    PipelineFailure {
        /// The stage where the failure occurred.
        stage: &'static str,
        /// The underlying error.
        #[source]
        source: Box<dyn std::error::Error + Send>,
    },

    /// Generic validation error.
    #[error("validation: {0}")]
    Validation(String),

    /// An operation exceeded its time budget.
    #[error("operation timed out after {ms}ms")]
    Timeout {
        /// Timeout threshold in milliseconds.
        ms: u64,
    },

    /// Configuration loading or parsing error.
    #[error("config: {0}")]
    Config(String),

    /// A graph node was not found.
    #[error("node {id} not found")]
    NodeNotFound {
        /// The missing node id.
        id: u64,
    },

    /// An invalid ECMA state transition was attempted.
    #[error("invalid transition: {from} -> {to}")]
    InvalidTransition {
        /// Source state.
        from: String,
        /// Target state.
        to: String,
    },

    /// An mRNA instruction exceeded its TTL.
    #[error("mRNA expired (ttl={ttl_ms}ms)")]
    MrnaExpired {
        /// The TTL that was exceeded.
        ttl_ms: u64,
    },

    /// Rate limit exceeded.
    #[error("rate limit exceeded")]
    RateLimitExceeded,

    /// Circuit breaker is open.
    #[error("circuit breaker open")]
    CircuitOpen,
}

/// Errors specific to the storage layer.
#[derive(Debug, Error)]
pub enum StorageError {
    /// A page checksum did not match the expected value.
    #[error("checksum failure on page {page_id}: expected {expected:#010x}, got {actual:#010x}")]
    ChecksumFailure {
        /// Page identifier.
        page_id: u64,
        /// Expected CRC32.
        expected: u32,
        /// Actual CRC32.
        actual: u32,
    },

    /// The WAL file is corrupted and cannot be replayed.
    #[error("WAL corrupted at byte offset {offset}: {reason}")]
    WalCorrupted {
        /// Byte offset of corruption.
        offset: u64,
        /// Human-readable reason.
        reason: String,
    },

    /// A requested key was not found in storage.
    #[error("key not found")]
    NotFound,

    /// The disk is full and a write cannot complete.
    #[error("disk full")]
    DiskFull,

    /// An underlying I/O error.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors specific to metric computation.
#[derive(Debug, Error)]
pub enum MetricError {
    /// Inputs are not valid probability distributions.
    #[error("invalid distribution: {0}")]
    InvalidDistribution(String),

    /// Input vectors have mismatched dimensions.
    #[error("dimension mismatch: {0} vs {1}")]
    DimensionMismatch(usize, usize),

    /// A vector contains NaN or infinite values.
    #[error("vector contains non-finite values")]
    NonFiniteValues,

    /// Sinkhorn iteration did not converge within the iteration budget.
    #[error("sinkhorn did not converge in {iterations} iterations")]
    SinkhornNotConverged {
        /// Number of iterations attempted.
        iterations: usize,
    },
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn kce_error_display_storage() {
        let err = KceError::Storage(StorageError::NotFound);
        assert_eq!(err.to_string(), "storage: key not found");
    }

    #[test]
    fn kce_error_display_dimension_mismatch() {
        let err = KceError::DimensionMismatch {
            query: 3,
            dataset: 4,
        };
        assert!(err.to_string().contains("3"));
        assert!(err.to_string().contains("4"));
    }

    #[test]
    fn storage_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let err: StorageError = io_err.into();
        assert!(err.to_string().contains("gone"));
    }

    #[test]
    fn metric_error_display() {
        let err = MetricError::SinkhornNotConverged { iterations: 100 };
        assert!(err.to_string().contains("100"));
    }
}
