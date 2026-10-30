//! Criterion benchmarks for the full KCE pipeline.
//!
//! Run with: `cargo bench --bench pipeline_bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use kce_aco::ant::{AntConfig, CandidateEdge};
use kce_aco::colony::{Colony, ColonyConfig};
use kce_aco::pheromone::{PheromoneConfig, PheromoneMap};
use kce_core::traits::DistanceMetric;
use kce_core::types::ContextInput;
use kce_pipeline::orchestrator::{PipelineConfig, PipelineOrchestrator};
use kce_retrieval::engine::{Dataset, RetrievalEngine};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_dataset(n: usize, dim: usize) -> Dataset {
    let mut ds = Dataset::new(dim);
    for i in 0..n {
        let v: Vec<f64> = (0..dim)
            .map(|j| ((i * dim + j) as f64) / (n * dim) as f64)
            .collect();
        ds.push(i as u64 + 1, v).expect("push");
    }
    ds
}

fn make_query(dim: usize) -> Vec<f64> {
    (0..dim).map(|i| (i as f64 + 0.5) / dim as f64).collect()
}

// ── Pipeline execute() ───────────────────────────────────────────────────────

fn bench_pipeline_execute(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_execute");

    for &n in &[10, 100] {
        let dataset = make_dataset(n, 128);
        let pipeline = PipelineOrchestrator::new(PipelineConfig::default(), dataset);
        let input = ContextInput {
            query_vector: make_query(128),
            top_k: 5,
            context: None,
            tenant_id: "bench".into(),
        };

        group.bench_function(format!("n={n}"), |b| {
            b.iter(|| {
                let result = pipeline.execute(black_box(&input));
                black_box(&result);
            });
        });
    }

    group.finish();
}

// ── Retrieval search with varying top_k ──────────────────────────────────────

fn bench_retrieval_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("retrieval_search");

    let dataset = make_dataset(200, 128);
    let engine = RetrievalEngine::with_defaults();
    let query = make_query(128);

    for &top_k in &[1, 5, 10, 50] {
        group.bench_function(format!("top_k={top_k}"), |b| {
            b.iter(|| {
                let results = engine.search(black_box(&query), black_box(&dataset), black_box(top_k));
                black_box(&results);
            });
        });
    }

    group.finish();
}

// ── ACO pheromone routing ────────────────────────────────────────────────────

fn aco_neighbors(node: u64) -> Vec<CandidateEdge> {
    match node {
        1 => vec![
            CandidateEdge { to: 2, weight: 1.0 },
            CandidateEdge { to: 3, weight: 2.0 },
            CandidateEdge { to: 4, weight: 1.5 },
        ],
        2 => vec![
            CandidateEdge { to: 5, weight: 1.0 },
            CandidateEdge { to: 3, weight: 1.5 },
        ],
        3 => vec![
            CandidateEdge { to: 5, weight: 0.5 },
            CandidateEdge { to: 6, weight: 1.0 },
        ],
        4 => vec![
            CandidateEdge { to: 6, weight: 2.0 },
            CandidateEdge { to: 7, weight: 1.0 },
        ],
        5 => vec![CandidateEdge { to: 7, weight: 0.8 }],
        6 => vec![CandidateEdge { to: 7, weight: 0.5 }],
        _ => vec![],
    }
}

fn bench_aco_colony(c: &mut Criterion) {
    let mut group = c.benchmark_group("aco_colony");

    for &num_ants in &[5, 20] {
        group.bench_function(format!("ants={num_ants}"), |b| {
            b.iter_batched(
                || {
                    Colony::new(ColonyConfig {
                        num_ants,
                        max_iterations: 10,
                        ..Default::default()
                    })
                },
                |mut colony| {
                    let result = colony.run(black_box(1), aco_neighbors);
                    black_box(&result);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_pheromone_deposit(c: &mut Criterion) {
    let mut group = c.benchmark_group("pheromone_deposit");

    group.bench_function("deposit_1000_edges", |b| {
        b.iter_batched(
            || PheromoneMap::new(PheromoneConfig::default()),
            |mut map| {
                for i in 0..1000u64 {
                    map.deposit(black_box(i), black_box(i + 1), black_box(0.5));
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ── Criterion groups ─────────────────────────────────────────────────────────

criterion_group!(
    pipeline_benches,
    bench_pipeline_execute,
    bench_retrieval_search,
    bench_aco_colony,
    bench_pheromone_deposit,
);
criterion_main!(pipeline_benches);
