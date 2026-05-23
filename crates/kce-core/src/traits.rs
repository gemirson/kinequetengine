//! Core trait contracts for the KineContext Engine.
//!
//! Boundaries are traits. Not structs. Not concrete types. Traits.
//! Every module implements one of these contracts to participate in the pipeline.

use crate::error::{MetricError, StorageError};
use crate::types::{ContextInput, ContextOutput, HealthStatus};

/// The primary contract for all cognitive engines (Retrieval, ECMA, MCE, etc.).
///
/// Each engine processes a [`ContextInput`] and produces a [`ContextOutput`].
/// Engines are composable — the pipeline wires them together via this trait.
pub trait ContextEngine: Send + Sync {
    /// Engine-specific configuration type.
    type Config: Default + serde::Deserialize<'static>;

    /// Initialize the engine with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::KceError`] if configuration is invalid.
    fn init(config: Self::Config) -> Result<Self, crate::error::KceError>
    where
        Self: Sized;

    /// Process a context input and produce an output.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::KceError`] if processing fails at any stage.
    fn process(&self, ctx: &ContextInput) -> Result<ContextOutput, crate::error::KceError>;

    /// Report the engine's current health status.
    fn health(&self) -> HealthStatus;
}

/// Contract for the storage layer (KineSQL).
///
/// All storage access goes through this trait — never directly through
/// KineSQL internals.
pub trait StorageBackend: Send + Sync {
    /// Read a value by key.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::NotFound`] if the key does not exist.
    /// Returns [`StorageError::ChecksumFailure`] on data corruption.
    fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;

    /// Write a key-value pair.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::DiskFull`] if no space is available.
    fn write(&mut self, key: &[u8], value: &[u8]) -> Result<(), StorageError>;

    /// Flush all pending writes to durable storage (fsync).
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Io`] on fsync failure.
    fn flush(&mut self) -> Result<(), StorageError>;

    /// Recover state from WAL after a crash.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::WalCorrupted`] if the WAL cannot be replayed.
    fn recover(&mut self) -> Result<RecoveryReport, StorageError>;
}

/// Report produced after a storage recovery operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecoveryReport {
    /// Number of WAL entries replayed.
    pub wal_entries_replayed: usize,
    /// Number of pages recovered.
    pub pages_recovered: usize,
    /// Number of checksum failures encountered.
    pub checksum_failures: usize,
    /// Whether data integrity was verified.
    pub data_integrity_valid: bool,
}

/// Contract for distance/similarity metrics.
///
/// Pure functions — no state. Implementations include cosine, prime (GCD),
/// Wasserstein, and Sinkhorn.
pub trait DistanceMetric: Send + Sync {
    /// Compute the distance (or similarity) between two vectors.
    ///
    /// # Errors
    ///
    /// Returns [`MetricError::DimensionMismatch`] if inputs have different lengths.
    fn compute(&self, a: &[f64], b: &[f64]) -> Result<f64, MetricError>;

    /// Human-readable name of this metric (e.g. "cosine", "sinkhorn").
    fn name(&self) -> &'static str;
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn recovery_report_serializes() {
        let report = RecoveryReport {
            wal_entries_replayed: 10,
            pages_recovered: 3,
            checksum_failures: 0,
            data_integrity_valid: true,
        };
        let json = serde_json::to_string(&report).expect("serialize");
        assert!(json.contains("10"));
        assert!(json.contains("true"));
    }
}
