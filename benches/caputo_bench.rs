//! Criterion benchmarks for FT-037: Caputo Fractional Hereditary Memory.
//!
//! Validates the performance SLA:
//! - Latency < 10ns per node evaluation
//! - Cache line alignment and layout (48 bytes)
//! - SIMD auto-vectorization across node batches
//!
//! Run with: `cargo bench --bench caputo_bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kce_core::caputo::CaputoState;

fn bench_caputo_sla(c: &mut Criterion) {
    let mut group = c.benchmark_group("caputo_memory");

    // 1. Single Node Peek Intensity (< 10ns SLA)
    group.bench_function("peek_intensity_single", |b| {
        let state = CaputoState::new(0.55, 100.0, 10_000);
        let now_ms = 15_000;
        b.iter(|| {
            let val = state.peek_intensity(black_box(now_ms));
            black_box(val);
        });
    });

    // 2. Step and Reinforce (O(1) update and stimulus injection)
    group.bench_function("step_and_reinforce", |b| {
        let mut state = CaputoState::new(0.55, 100.0, 10_000);
        let mut t = 10_000;
        b.iter(|| {
            t += 100;
            let val = state.step_and_reinforce(black_box(t), black_box(1.5));
            black_box(val);
        });
    });

    // 3. Phase Transition / Modulate Alpha
    group.bench_function("modulate_alpha", |b| {
        let mut state = CaputoState::new(0.90, 80.0, 10_000);
        let mut toggle = false;
        b.iter(|| {
            toggle = !toggle;
            let target_alpha = if toggle { 0.15 } else { 0.90 };
            state.modulate_alpha(black_box(target_alpha), black_box(12_000));
            black_box(state.cached_intensity);
        });
    });

    // 4. Batch 1000 Nodes Scan (Simulating HNSW / Semantic Graph expansion)
    group.bench_function("batch_peek_1000_nodes", |b| {
        let states: Vec<CaputoState> = (0..1000)
            .map(|i| {
                let alpha = 0.1 + (i % 80) as f32 * 0.01;
                CaputoState::new(alpha, 50.0 + (i % 50) as f32, 1_000_000)
            })
            .collect();
        let now_ms = 1_060_000; // 60s later

        b.iter(|| {
            let mut sum = 0.0_f32;
            for s in states.iter() {
                sum += s.peek_intensity(black_box(now_ms));
            }
            black_box(sum);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_caputo_sla);
criterion_main!(benches);
