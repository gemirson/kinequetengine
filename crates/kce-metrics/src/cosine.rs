//! Cosine similarity metric with SIMD (AVX2) acceleration.
//!
//! Computes the cosine of the angle between two vectors.
//! Returns `1.0` for identical vectors, `0.0` for orthogonal, `-1.0` for opposite.
//!
//! On x86_64 targets, this module uses AVX2 intrinsics when available at runtime
//! to compute the dot product and squared-norm accumulations 4 elements at a time.
//! A scalar fallback handles non-AVX2 systems and trailing elements.

use kce_core::error::MetricError;
use kce_core::traits::DistanceMetric;

/// Cosine similarity metric with optional AVX2 SIMD acceleration.
#[derive(Debug, Default)]
pub struct CosineMetric;

impl CosineMetric {
    /// Create a new cosine metric.
    pub fn new() -> Self {
        Self
    }
}

/// Validate that both slices are finite, returning `Err(NonFiniteValues)` if not.
fn validate_finite(a: &[f64], b: &[f64]) -> Result<(), MetricError> {
    for (va, vb) in a.iter().zip(b.iter()) {
        if !va.is_finite() || !vb.is_finite() {
            return Err(MetricError::NonFiniteValues);
        }
    }
    Ok(())
}

/// Scalar fallback: compute (dot, norm_a, norm_b) with a plain loop.
fn scalar_dot_norms(a: &[f64], b: &[f64]) -> (f64, f64, f64) {
    let mut dot = 0.0_f64;
    let mut norm_a = 0.0_f64;
    let mut norm_b = 0.0_f64;
    for (va, vb) in a.iter().zip(b.iter()) {
        dot += va * vb;
        norm_a += va * va;
        norm_b += vb * vb;
    }
    (dot, norm_a, norm_b)
}

/// AVX2-accelerated computation of (dot, norm_a, norm_b).
///
/// # Safety
///
/// Caller must ensure AVX2 is available on the current CPU. This is guarded
/// by `is_x86_feature_detected!("avx2")` in the only call site.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
unsafe fn avx2_dot_norms(a: &[f64], b: &[f64]) -> (f64, f64, f64) {
    use std::arch::x86_64::*;

    let len = a.len();
    let mut dot = _mm256_setzero_pd();
    let mut norm_a = _mm256_setzero_pd();
    let mut norm_b = _mm256_setzero_pd();

    let chunks = len / 4;
    let pa = a.as_ptr();
    let pb = b.as_ptr();

    for i in 0..chunks {
        let offset = i * 4;
        // Safety: offset..offset+4 is within bounds because i < len/4.
        let va = _mm256_loadu_pd(pa.add(offset));
        let vb = _mm256_loadu_pd(pb.add(offset));
        dot = _mm256_fmadd_pd(va, vb, dot);
        norm_a = _mm256_fmadd_pd(va, va, norm_a);
        norm_b = _mm256_fmadd_pd(vb, vb, norm_b);
    }

    // Horizontal sum of the 4-wide accumulators.
    let dot_sum = hsum_avx2(dot);
    let norm_a_sum = hsum_avx2(norm_a);
    let norm_b_sum = hsum_avx2(norm_b);

    // Handle trailing elements with scalar code.
    let remainder_start = chunks * 4;
    let (tail_dot, tail_na, tail_nb) = scalar_dot_norms(&a[remainder_start..], &b[remainder_start..]);

    (dot_sum + tail_dot, norm_a_sum + tail_na, norm_b_sum + tail_nb)
}

/// Horizontal sum of a 256-bit register containing 4 f64 values.
///
/// # Safety
///
/// Caller must ensure AVX2 is available.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
unsafe fn hsum_avx2(v: std::arch::x86_64::__m256d) -> f64 {
    use std::arch::x86_64::*;
    // v = [a, b, c, d]
    // lo = [a, b], hi = [c, d]
    let lo = _mm256_castpd256_pd128(v);
    let hi = _mm256_extractf128_pd::<1>(v);
    // sum128 = [a+c, b+d]
    let sum128 = _mm_add_pd(lo, hi);
    // sum64 = [a+c+b+d, ...]
    let sum64 = _mm_hadd_pd(sum128, sum128);
    _mm_cvtsd_f64(sum64)
}

impl DistanceMetric for CosineMetric {
    fn compute(&self, a: &[f64], b: &[f64]) -> Result<f64, MetricError> {
        if a.len() != b.len() {
            return Err(MetricError::DimensionMismatch(a.len(), b.len()));
        }
        if a.is_empty() {
            return Ok(0.0);
        }

        // Validate before any SIMD path to avoid branching on non-finite inside intrinsics.
        validate_finite(a, b)?;

        #[cfg(target_arch = "x86_64")]
        let (dot, norm_a, norm_b) = if is_x86_feature_detected!("avx2") && a.len() >= 4 {
            // Safety: runtime feature detection confirmed AVX2 is available.
            #[allow(unsafe_code)]
            let result = unsafe { avx2_dot_norms(a, b) };
            result
        } else {
            scalar_dot_norms(a, b)
        };

        #[cfg(not(target_arch = "x86_64"))]
        let (dot, norm_a, norm_b) = scalar_dot_norms(a, b);

        let denom = norm_a.sqrt() * norm_b.sqrt();
        if denom == 0.0 {
            return Ok(0.0);
        }

        Ok(dot / denom)
    }

    fn name(&self) -> &'static str {
        "cosine"
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn identical_vectors() {
        let m = CosineMetric::new();
        let result = m.compute(&[1.0, 0.0], &[1.0, 0.0]).expect("compute");
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn orthogonal_vectors() {
        let m = CosineMetric::new();
        let result = m.compute(&[1.0, 0.0], &[0.0, 1.0]).expect("compute");
        assert!((result - 0.0).abs() < 1e-10);
    }

    #[test]
    fn opposite_vectors() {
        let m = CosineMetric::new();
        let result = m.compute(&[1.0, 0.0], &[-1.0, 0.0]).expect("compute");
        assert!((result - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn zero_vector_returns_zero() {
        let m = CosineMetric::new();
        let result = m.compute(&[0.0, 0.0], &[1.0, 0.0]).expect("compute");
        assert_eq!(result, 0.0);
    }

    #[test]
    fn empty_vectors() {
        let m = CosineMetric::new();
        let result = m.compute(&[], &[]).expect("compute");
        assert_eq!(result, 0.0);
    }

    #[test]
    fn dimension_mismatch() {
        let m = CosineMetric::new();
        let result = m.compute(&[1.0, 2.0], &[1.0]);
        assert!(result.is_err());
    }

    #[test]
    fn nan_returns_error() {
        let m = CosineMetric::new();
        let result = m.compute(&[f64::NAN, 0.0], &[1.0, 0.0]);
        assert!(result.is_err());
    }

    #[test]
    fn name_is_cosine() {
        let m = CosineMetric::new();
        assert_eq!(m.name(), "cosine");
    }

    /// SIMD path: vectors with 128 elements (divisible by 4).
    #[test]
    fn simd_path_128_elements() {
        let m = CosineMetric::new();
        let a: Vec<f64> = (0..128).map(|i| (i as f64) / 128.0).collect();
        let b: Vec<f64> = (0..128).map(|i| ((i + 1) as f64) / 128.0).collect();
        let result = m.compute(&a, &b).expect("compute");
        // Manually compute expected cosine.
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let na: f64 = a.iter().map(|x| x * x).sum();
        let nb: f64 = b.iter().map(|x| x * x).sum();
        let expected = dot / (na.sqrt() * nb.sqrt());
        assert!(
            (result - expected).abs() < 1e-12,
            "SIMD result {result} != expected {expected}"
        );
    }

    /// SIMD path with trailing scalar elements: 130 elements (130 = 32*4 + 2).
    #[test]
    fn simd_path_with_remainder() {
        let m = CosineMetric::new();
        let a: Vec<f64> = (0..130).map(|i| (i as f64) * 0.01).collect();
        let b: Vec<f64> = (0..130).map(|i| ((i + 3) as f64) * 0.01).collect();
        let result = m.compute(&a, &b).expect("compute");
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let na: f64 = a.iter().map(|x| x * x).sum();
        let nb: f64 = b.iter().map(|x| x * x).sum();
        let expected = dot / (na.sqrt() * nb.sqrt());
        assert!(
            (result - expected).abs() < 1e-12,
            "SIMD+tail result {result} != expected {expected}"
        );
    }

    /// Identity-ish test: large identical vectors must return ~1.0.
    #[test]
    fn simd_identical_large() {
        let m = CosineMetric::new();
        let a: Vec<f64> = (0..256).map(|i| (i as f64).sin()).collect();
        let result = m.compute(&a, &a).expect("compute");
        assert!(
            (result - 1.0).abs() < 1e-10,
            "identical large vectors should yield ~1.0, got {result}"
        );
    }
}
