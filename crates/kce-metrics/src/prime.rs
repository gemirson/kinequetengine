//! Prime similarity metric.
//!
//! Computes similarity between two integer-encoded vectors using GCD.
//! `1.0` means maximum shared factors, `0.0` means coprime.

use kce_core::error::MetricError;
use kce_core::traits::DistanceMetric;

/// Prime / GCD-based similarity metric.
#[derive(Debug, Default)]
pub struct PrimeMetric;

impl PrimeMetric {
    /// Create a new prime metric.
    pub fn new() -> Self {
        Self
    }

    /// Compute GCD of two u64 values.
    fn gcd(mut a: u64, mut b: u64) -> u64 {
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        a
    }
}

impl DistanceMetric for PrimeMetric {
    fn compute(&self, a: &[f64], b: &[f64]) -> Result<f64, MetricError> {
        if a.len() != b.len() {
            return Err(MetricError::DimensionMismatch(a.len(), b.len()));
        }
        if a.is_empty() {
            return Ok(0.0);
        }

        // Convert floats to scaled integers for GCD computation.
        let scale = 1000.0_f64;
        let mut total_gcd = 0_u64;
        let mut total_max = 0_u64;

        for (va, vb) in a.iter().zip(b.iter()) {
            if !va.is_finite() || !vb.is_finite() {
                return Err(MetricError::NonFiniteValues);
            }
            let ia = (va.abs() * scale).round() as u64;
            let ib = (vb.abs() * scale).round() as u64;
            total_gcd += Self::gcd(ia, ib);
            total_max += ia.max(ib);
        }

        if total_max == 0 {
            return Ok(0.0);
        }

        Ok(total_gcd as f64 / total_max as f64)
    }

    fn name(&self) -> &'static str {
        "prime"
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn identical_values() {
        let m = PrimeMetric::new();
        let result = m.compute(&[12.0, 12.0], &[12.0, 12.0]).expect("compute");
        assert!((result - 1.0).abs() < 1e-6);
    }

    #[test]
    fn coprime_values() {
        let m = PrimeMetric::new();
        let result = m.compute(&[3.0], &[7.0]).expect("compute");
        // GCD(3,7) = 1, max = 7, so 1/7 ≈ 0.143
        assert!(result < 0.2);
        assert!(result > 0.0);
    }

    #[test]
    fn empty_vectors() {
        let m = PrimeMetric::new();
        let result = m.compute(&[], &[]).expect("compute");
        assert_eq!(result, 0.0);
    }

    #[test]
    fn dimension_mismatch() {
        let m = PrimeMetric::new();
        assert!(m.compute(&[1.0, 2.0], &[1.0]).is_err());
    }

    #[test]
    fn name_is_prime() {
        assert_eq!(PrimeMetric::new().name(), "prime");
    }
}
