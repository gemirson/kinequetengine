//! Sinkhorn distance — entropic-regularized optimal transport.
//!
//! Implements the Sinkhorn-Knopp iterative scaling algorithm for computing
//! an approximation to the optimal transport (Wasserstein) distance between
//! two discrete distributions.  Entropic regularization with parameter ε
//! makes the problem strictly convex and solvable in O(n²/ε) time.
//!
//! Reference: Cuturi (2013) "Sinkhorn Distances: Lightspeed Computation of
//! Optimal Transport"

use kce_core::error::MetricError;
use kce_core::traits::DistanceMetric;

/// Configuration for the Sinkhorn metric.
#[derive(Debug, Clone)]
pub struct SinkhornConfig {
    /// Entropic regularization parameter (ε).  Smaller = more precise, slower.
    pub epsilon: f64,
    /// Maximum number of Sinkhorn iterations.
    pub max_iter: usize,
    /// Convergence tolerance on marginal residuals.
    pub tolerance: f64,
}

impl Default for SinkhornConfig {
    fn default() -> Self {
        Self {
            epsilon: 0.01,
            max_iter: 100,
            tolerance: 1e-6,
        }
    }
}

/// Detailed result from a Sinkhorn computation.
#[derive(Debug, Clone)]
pub struct SinkhornResult {
    /// The computed transport distance.
    pub distance: f64,
    /// Number of Sinkhorn iterations performed.
    pub iterations: usize,
    /// Whether the algorithm converged within `max_iter`.
    pub converged: bool,
    /// Wall-clock computation time in milliseconds.
    pub computation_ms: f64,
}

/// Sinkhorn optimal transport distance.
#[derive(Debug)]
pub struct SinkhornMetric {
    config: SinkhornConfig,
}

impl SinkhornMetric {
    /// Create a new Sinkhorn metric with the given configuration.
    pub fn new(config: SinkhornConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(SinkhornConfig::default())
    }
}

impl DistanceMetric for SinkhornMetric {
    fn compute(&self, a: &[f64], b: &[f64]) -> Result<f64, MetricError> {
        if a.len() != b.len() {
            return Err(MetricError::DimensionMismatch(a.len(), b.len()));
        }
        if a.is_empty() {
            return Ok(0.0);
        }

        // Validate non-negative
        if a.iter().any(|v| *v < 0.0) || b.iter().any(|v| *v < 0.0) {
            return Err(MetricError::InvalidDistribution(
                "distributions must be non-negative".into(),
            ));
        }

        let sum_a: f64 = a.iter().sum();
        let sum_b: f64 = b.iter().sum();
        if sum_a <= 0.0 || sum_b <= 0.0 {
            return Err(MetricError::InvalidDistribution(
                "distributions must have positive sum".into(),
            ));
        }

        // Normalize to probability distributions
        let a_norm: Vec<f64> = a.iter().map(|v| v / sum_a).collect();
        let b_norm: Vec<f64> = b.iter().map(|v| v / sum_b).collect();

        // Check if identical (fast path)
        let is_identical = a_norm
            .iter()
            .zip(b_norm.iter())
            .all(|(x, y)| (x - y).abs() < 1e-12);
        if is_identical {
            return Ok(0.0);
        }

        let result = sinkhorn_core(
            &a_norm,
            &b_norm,
            self.config.epsilon,
            self.config.max_iter,
            self.config.tolerance,
        );

        if !result.converged {
            // Fallback to L1 distance
            let l1: f64 = a_norm
                .iter()
                .zip(b_norm.iter())
                .map(|(va, vb)| (va - vb).abs())
                .sum();
            return Ok(l1);
        }

        Ok(result.distance)
    }

    fn name(&self) -> &'static str {
        "sinkhorn"
    }
}

/// Compute Sinkhorn distance with detailed result (for tests and diagnostics).
pub fn sinkhorn_detailed(
    a: &[f64],
    b: &[f64],
    epsilon: f64,
    max_iter: usize,
) -> Result<SinkhornResult, MetricError> {
    if a.len() != b.len() {
        return Err(MetricError::DimensionMismatch(a.len(), b.len()));
    }
    if a.is_empty() {
        return Ok(SinkhornResult {
            distance: 0.0,
            iterations: 0,
            converged: true,
            computation_ms: 0.0,
        });
    }

    if a.iter().any(|v| *v < 0.0) || b.iter().any(|v| *v < 0.0) {
        return Err(MetricError::InvalidDistribution(
            "distributions must be non-negative".into(),
        ));
    }

    let sum_a: f64 = a.iter().sum();
    let sum_b: f64 = b.iter().sum();
    if sum_a <= 0.0 || sum_b <= 0.0 {
        return Err(MetricError::InvalidDistribution(
            "distributions must have positive sum".into(),
        ));
    }

    if epsilon <= 0.0 {
        return Err(MetricError::InvalidDistribution(
            "epsilon must be positive".into(),
        ));
    }

    let a_norm: Vec<f64> = a.iter().map(|v| v / sum_a).collect();
    let b_norm: Vec<f64> = b.iter().map(|v| v / sum_b).collect();

    Ok(sinkhorn_core(&a_norm, &b_norm, epsilon, max_iter, 1e-6))
}

/// Core Sinkhorn iterative scaling algorithm.
///
/// Given two probability distributions `a` and `b` of length `n`:
/// 1. Build cost matrix `C[i][j] = |a_pos[i] - b_pos[j]|` (1D euclidean on indices)
/// 2. Compute Gibbs kernel `K[i][j] = exp(-C[i][j] / epsilon)`
/// 3. Iterate Sinkhorn scaling:
///    - `u = a ./ (K @ v)`
///    - `v = b ./ (K^T @ u)`
/// 4. Transport plan `T = diag(u) @ K @ diag(v)`
/// 5. Distance = `sum(T * C)`
fn sinkhorn_core(
    a: &[f64],
    b: &[f64],
    epsilon: f64,
    max_iter: usize,
    tolerance: f64,
) -> SinkhornResult {
    let start = std::time::Instant::now();
    let n = a.len();

    // Build cost matrix: euclidean distance between positions (1D embedding)
    // For 1D distributions, position i has coordinate i/n
    let mut cost = vec![0.0f64; n * n];
    for i in 0..n {
        for j in 0..n {
            let d = (i as f64 - j as f64).abs() / n as f64;
            cost[i * n + j] = d;
        }
    }

    // Gibbs kernel K[i][j] = exp(-C[i][j] / epsilon)
    let mut k = vec![0.0f64; n * n];
    for i in 0..n * n {
        k[i] = (-cost[i] / epsilon).exp();
    }

    // Sinkhorn scaling
    let mut u = vec![1.0f64; n];
    let mut v = vec![1.0f64; n];
    let mut converged = false;
    let mut iterations = 0;

    for iter in 0..max_iter {
        iterations = iter + 1;

        // u = a / (K @ v)
        let mut max_residual = 0.0f64;
        for i in 0..n {
            let kv: f64 = (0..n).map(|j| k[i * n + j] * v[j]).sum();
            if kv > 1e-300 {
                let new_u = a[i] / kv;
                max_residual = max_residual.max((new_u - u[i]).abs());
                u[i] = new_u;
            }
        }

        // v = b / (K^T @ u)
        for j in 0..n {
            let ktu: f64 = (0..n).map(|i| k[i * n + j] * u[i]).sum();
            if ktu > 1e-300 {
                let new_v = b[j] / ktu;
                max_residual = max_residual.max((new_v - v[j]).abs());
                v[j] = new_v;
            }
        }

        if max_residual < tolerance {
            converged = true;
            break;
        }
    }

    // Compute transport distance: sum(T * C) where T = diag(u) * K * diag(v)
    let mut distance = 0.0f64;
    for i in 0..n {
        for j in 0..n {
            distance += u[i] * k[i * n + j] * v[j] * cost[i * n + j];
        }
    }

    SinkhornResult {
        distance,
        iterations,
        converged,
        computation_ms: start.elapsed().as_secs_f64() * 1000.0,
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn identical_distributions() {
        let m = SinkhornMetric::with_defaults();
        let a = vec![0.5, 0.5];
        let result = m.compute(&a, &a).expect("compute");
        assert!(
            result.abs() < 1e-6,
            "identical distributions should have distance ~0, got {}",
            result
        );
    }

    #[test]
    fn different_distributions() {
        let m = SinkhornMetric::with_defaults();
        let result = m.compute(&[0.2, 0.8], &[0.6, 0.4]).expect("compute");
        assert!(
            result > 0.0,
            "different distributions should have distance > 0"
        );
    }

    #[test]
    fn symmetry() {
        let m = SinkhornMetric::with_defaults();
        let a = vec![0.2, 0.3, 0.5];
        let b = vec![0.4, 0.4, 0.2];
        let ab = m.compute(&a, &b).expect("compute a,b");
        let ba = m.compute(&b, &a).expect("compute b,a");
        assert!(
            (ab - ba).abs() < 1e-6,
            "W(a,b) should equal W(b,a), got {} vs {}",
            ab,
            ba
        );
    }

    #[test]
    fn non_negativity() {
        let m = SinkhornMetric::with_defaults();
        let pairs: Vec<(Vec<f64>, Vec<f64>)> = vec![
            (vec![0.1, 0.9], vec![0.9, 0.1]),
            (vec![0.25, 0.25, 0.25, 0.25], vec![0.1, 0.2, 0.3, 0.4]),
            (vec![1.0, 0.0, 0.0], vec![0.0, 0.0, 1.0]),
        ];
        for (a, b) in pairs {
            let d = m.compute(&a, &b).expect("compute");
            assert!(d >= 0.0, "distance should be non-negative, got {}", d);
        }
    }

    #[test]
    fn convergence_within_100_iters() {
        let result = sinkhorn_detailed(&[0.2, 0.3, 0.5], &[0.4, 0.4, 0.2], 0.01, 100)
            .expect("detailed");
        assert!(
            result.converged,
            "should converge within 100 iterations, did {} iters",
            result.iterations
        );
        assert!(
            result.iterations <= 100,
            "iterations {} should be <= 100",
            result.iterations
        );
    }

    #[test]
    fn triangle_inequality_approximate() {
        let m = SinkhornMetric::with_defaults();
        let a = vec![0.1, 0.9];
        let b = vec![0.5, 0.5];
        let c = vec![0.9, 0.1];
        let ab = m.compute(&a, &b).expect("ab");
        let bc = m.compute(&b, &c).expect("bc");
        let ac = m.compute(&a, &c).expect("ac");
        // Sinkhorn with regularization is approximate, allow small violation
        assert!(
            ac <= ab + bc + 0.05,
            "triangle inequality approx: W(a,c)={} <= W(a,b)+W(b,c)={}+{}={}",
            ac,
            ab,
            bc,
            ab + bc
        );
    }

    #[test]
    fn dimension_mismatch() {
        let m = SinkhornMetric::with_defaults();
        assert!(m.compute(&[0.5, 0.5], &[1.0]).is_err());
    }

    #[test]
    fn invalid_distribution() {
        let m = SinkhornMetric::with_defaults();
        assert!(m.compute(&[0.0, 0.0], &[0.5, 0.5]).is_err());
    }

    #[test]
    fn negative_values_rejected() {
        let m = SinkhornMetric::with_defaults();
        assert!(m.compute(&[-1.0, 2.0], &[1.0, 1.0]).is_err());
    }

    #[test]
    fn empty_distributions() {
        let m = SinkhornMetric::with_defaults();
        assert_eq!(m.compute(&[], &[]).expect("compute"), 0.0);
    }

    #[test]
    fn dirac_distributions_have_max_distance() {
        let m = SinkhornMetric::with_defaults();
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 0.0, 1.0];
        let d = m.compute(&a, &b).expect("compute");
        assert!(
            d > 0.1,
            "dirac at opposite ends should have significant distance, got {}",
            d
        );
    }

    #[test]
    fn detailed_result_fields() {
        let result = sinkhorn_detailed(&[0.3, 0.7], &[0.7, 0.3], 0.01, 100).expect("detailed");
        assert!(result.distance > 0.0);
        assert!(result.iterations > 0);
        assert!(result.converged);
        assert!(result.computation_ms >= 0.0);
    }

    #[test]
    fn name_is_sinkhorn() {
        let m = SinkhornMetric::with_defaults();
        assert_eq!(m.name(), "sinkhorn");
    }
}
