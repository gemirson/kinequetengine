//! Criterion benchmarks for KCE metrics.
//!
//! Run with: `cargo bench --bench metrics_bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use kce_core::traits::DistanceMetric;
use kce_metrics::cosine::CosineMetric;
use kce_metrics::latency::LatencyHistogram;

// ── LatencyHistogram::record_us ──────────────────────────────────────────────

fn bench_histogram_record(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency_histogram");

    group.bench_function("record_us", |b| {
        let h = LatencyHistogram::new();
        b.iter(|| {
            h.record_us(black_box(1500));
        });
    });

    group.bench_function("percentiles", |b| {
        let h = LatencyHistogram::new();
        for i in 0..1000 {
            h.record_us(i * 100);
        }
        b.iter(|| {
            let snap = h.percentiles();
            black_box(&snap);
        });
    });

    group.finish();
}

// ── Cosine similarity ────────────────────────────────────────────────────────

fn bench_cosine(c: &mut Criterion) {
    let mut group = c.benchmark_group("cosine");
    let m = CosineMetric::new();

    group.bench_function("dim=2", |b| {
        let a = vec![0.5, 0.8];
        let b_vec = vec![0.3, 0.9];
        b.iter(|| {
            let result = m.compute(black_box(&a), black_box(&b_vec));
            black_box(&result);
        });
    });

    group.bench_function("dim=128", |b| {
        let a: Vec<f64> = (0..128).map(|i| (i as f64) / 128.0).collect();
        let b_vec: Vec<f64> = (0..128).map(|i| ((i + 1) as f64) / 128.0).collect();
        b.iter(|| {
            let result = m.compute(black_box(&a), black_box(&b_vec));
            black_box(&result);
        });
    });

    group.bench_function("dim=256", |b| {
        let a: Vec<f64> = (0..256).map(|i| (i as f64).sin() / 256.0).collect();
        let b_vec: Vec<f64> = (0..256).map(|i| (i as f64).cos() / 256.0).collect();
        b.iter(|| {
            let result = m.compute(black_box(&a), black_box(&b_vec));
            black_box(&result);
        });
    });

    group.finish();
}

// ── Criterion groups ─────────────────────────────────────────────────────────

criterion_group!(metrics_benches, bench_histogram_record, bench_cosine,);
criterion_main!(metrics_benches);
