//! KCE High-Concurrency Stress Benchmark.
//!
//! This benchmark simulates a heavy real-world workload:
//! 1. Large dataset (10,000 vectors).
//! 2. High-concurrency parallel queries using Rayon.
//! 3. Diverse search patterns (narrow vs broad top_k).
//! 4. Continuous ACO pheromone updates during retrieval.
//!
//! Run with: `cargo bench --bench kce_stress_bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rayon::prelude::*;
use std::sync::Arc;

use kce_core::types::ContextInput;
use kce_pipeline::orchestrator::{PipelineConfig, PipelineOrchestrator};
use kce_retrieval::engine::Dataset;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_large_dataset(n: usize, dim: usize) -> Dataset {
    let mut ds = Dataset::new(dim);
    for i in 0..n {
        let v: Vec<f64> = (0..dim)
            .map(|j| ((i * dim + j) as f64) / (n * dim) as f64)
            .collect();
        ds.push(i as u64 + 1, v).expect("push");
    }
    ds
}

fn make_queries(count: usize, dim: usize) -> Vec<Vec<f64>> {
    (0..count)
        .map(|i| {
            (0..dim)
                .map(|j| ((i + j) as f64 + 0.5) / (count + dim) as f64)
                .collect()
        })
        .collect()
}

// ── Stress Test: High-Concurrency Pipeline Execution ────────────────────────

fn bench_stress_parallel_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("kce_stress_parallel");
    
    // Setup: 10,000 vectors of 128 dimensions
    let n_records = 10_000;
    let dim = 128;
    let dataset = make_large_dataset(n_records, dim);
    let pipeline = Arc::new(PipelineOrchestrator::new(PipelineConfig::default(), dataset));
    
    // Batch of 100 queries to be executed in parallel
    let queries = make_queries(100, dim);
    
    group.sample_size(10); // High-concurrency tests are slow, reduce sample size
    
    group.bench_function(format!("parallel_batch_100_queries_n={n_records}"), |b| {
        b.iter(|| {
            queries.par_iter().for_each(|q| {
                let input = ContextInput {
                    query_vector: q.clone(),
                    top_k: 10,
                    context: None,
                    tenant_id: "stress_test".into(),
                };
                let _ = pipeline.execute(black_box(&input));
            });
        });
    });

    group.finish();
}

// ── Stress Test: Throughput vs Top-K ────────────────────────────────────────

fn bench_stress_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("kce_throughput");
    
    let dataset = make_large_dataset(1000, 128);
    let pipeline = PipelineOrchestrator::new(PipelineConfig::default(), dataset);
    let query = make_queries(1, 128).remove(0);

    for &top_k in &[10, 100, 500] {
        let input = ContextInput {
            query_vector: query.clone(),
            top_k,
            context: None,
            tenant_id: "throughput_test".into(),
        };

        group.bench_function(format!("throughput_top_k={top_k}"), |b| {
            b.iter(|| {
                let result = pipeline.execute(black_box(&input));
                black_box(&result);
            });
        });
    }

    group.finish();
}

// ── Criterion groups ─────────────────────────────────────────────────────────

criterion_group!(
    stress_benches,
    bench_stress_parallel_queries,
    bench_stress_throughput,
);
criterion_main!(stress_benches);
