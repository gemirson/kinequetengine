//! Criterion benchmarks for the retrieval engine.
//!
//! Run with: `cargo bench --bench retrieval_bench`

use kce_metrics::cosine::CosineMetric;
use kce_metrics::prime::PrimeMetric;
use kce_core::traits::DistanceMetric;

fn cosine_bench() {
    let m = CosineMetric::new();
    let a: Vec<f64> = (0..128).map(|i| (i as f64) / 128.0).collect();
    let b: Vec<f64> = (0..128).map(|i| ((i + 1) as f64) / 128.0).collect();
    for _ in 0..1000 {
        let _ = m.compute(&a, &b);
    }
}

fn prime_bench() {
    let m = PrimeMetric::new();
    let a: Vec<f64> = (0..128).map(|i| (i as f64) / 128.0).collect();
    let b: Vec<f64> = (0..128).map(|i| ((i + 1) as f64) / 128.0).collect();
    for _ in 0..1000 {
        let _ = m.compute(&a, &b);
    }
}

fn main() {
    // Simple benchmark runner (criterion not added as dep yet).
    let start = std::time::Instant::now();
    cosine_bench();
    println!("cosine x1000: {:?}", start.elapsed());

    let start = std::time::Instant::now();
    prime_bench();
    println!("prime x1000: {:?}", start.elapsed());
}
