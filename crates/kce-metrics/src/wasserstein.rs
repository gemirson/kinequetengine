//! Wasserstein distance — exact 1D optimal transport.
//!
//! This is a stub for the Wasserstein implementation specified in FT-023.
//! Full implementation will compute exact Earth Mover's Distance for 1D distributions.

use kce_core::error::MetricError;
use kce_core::traits::DistanceMetric;

/// Wasserstein (Earth Mover's) distance metric.
#[derive(Debug, Default)]
pub struct WassersteinMetric;

impl WassersteinMetric {
    /// Create a new Wasserstein metric.
    pub fn new() -> Self {
        Self
    }
}

impl DistanceMetric for WassersteinMetric {
    fn compute(&self, a: &[f64], b: &[f64]) -> Result<f64, MetricError> {
        if a.len() != b.len() {
            return Err(MetricError::DimensionMismatch(a.len(), b.len()));
        }
        if a.is_empty() {
            return Ok(0.0);
        }

        // Validate inputs are non-negative.
        if a.iter().any(|v| *v < 0.0) || b.iter().any(|v| *v < 0.0) {
            return Err(MetricError::InvalidDistribution(
                "distributions must be non-negative".into(),
            ));
        }

        // 1D Wasserstein-1 distance: L1 distance between CDFs.
        let sum_a: f64 = a.iter().sum();
        let sum_b: f64 = b.iter().sum();
        if sum_a == 0.0 && sum_b == 0.0 {
            return Ok(0.0);
        }

        let inv_a = if sum_a > 0.0 { 1.0 / sum_a } else { 0.0 };
        let inv_b = if sum_b > 0.0 { 1.0 / sum_b } else { 0.0 };

        let mut cum_a = 0.0;
        let mut cum_b = 0.0;
        let mut distance = 0.0;
        for (va, vb) in a.iter().zip(b.iter()) {
            cum_a += va * inv_a;
            cum_b += vb * inv_b;
            distance += (cum_a - cum_b).abs();
        }

        Ok(distance)
    }

    fn name(&self) -> &'static str {
        "wasserstein"
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn identical_distributions() {
        let m = WassersteinMetric::new();
        let a = vec![0.25, 0.25, 0.25, 0.25];
        let result = m.compute(&a, &a).expect("compute");
        assert!((result).abs() < 1e-10);
    }

    #[test]
    fn different_distributions() {
        let m = WassersteinMetric::new();
        let result = m.compute(&[0.1, 0.9], &[0.9, 0.1]).expect("compute");
        assert!(result > 0.0);
    }

    #[test]
    fn dimension_mismatch() {
        let m = WassersteinMetric::new();
        assert!(m.compute(&[0.5, 0.5], &[1.0]).is_err());
    }

    #[test]
    fn negative_values_rejected() {
        let m = WassersteinMetric::new();
        assert!(m.compute(&[-1.0, 2.0], &[1.0, 1.0]).is_err());
    }

    #[test]
    fn empty_distributions() {
        let m = WassersteinMetric::new();
        assert_eq!(m.compute(&[], &[]).expect("compute"), 0.0);
    }
}
