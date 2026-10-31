//! Criterion benchmarks for the retrieval engine.
//!
//! Run with: `cargo bench --bench retrieval_bench`

use kce_core::traits::DistanceMetric;
use kce_metrics::cosine::CosineMetric;

fn cosine_bench() {
    let m = CosineMetric::new();
    let a: Vec<f64> = (0..128).map(|i| (i as f64) / 128.0).collect();
    let b: Vec<f64> = (0..128).map(|i| ((i + 1) as f64) / 128.0).collect();
    for _ in 0..1000 {
        let _ = m.compute(&a, &b);
    }
}

fn main() {
    let start = std::time::Instant::now();
    cosine_bench();
    println!("cosine x1000: {:?}", start.elapsed());
}
