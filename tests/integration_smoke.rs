//! Integration smoke test -- verifies that the pipeline works end-to-end.
//!
//! Also includes concurrent stress tests (FT-010) and resilience checks (FT-009).

use kce_core::types::ContextInput;
use kce_pipeline::orchestrator::{PipelineConfig, PipelineOrchestrator};
use kce_retrieval::engine::Dataset;

fn make_pipeline() -> PipelineOrchestrator {
    let mut dataset = Dataset::new(4);
    dataset.push(1, vec![1.0, 0.0, 0.0, 0.0]).expect("push");
    dataset.push(2, vec![0.0, 1.0, 0.0, 0.0]).expect("push");
    dataset.push(3, vec![0.0, 0.0, 1.0, 0.0]).expect("push");
    dataset.push(4, vec![0.0, 0.0, 0.0, 1.0]).expect("push");
    PipelineOrchestrator::new(PipelineConfig::default(), dataset)
}

#[test]
fn smoke_pipeline_returns_results() {
    let pipeline = make_pipeline();
    let input = ContextInput {
        query_vector: vec![1.0, 0.0, 0.0, 0.0],
        top_k: 2,
        context: Some("smoke_test".into()),
        tenant_id: None,
    };
    let output = pipeline.execute(&input).expect("execute");
    assert_eq!(output.action, "retrieve");
    assert!(output.pipeline_ms >= 0.0);
}

#[test]
fn smoke_pipeline_deterministic() {
    let pipeline = make_pipeline();
    let input = ContextInput {
        query_vector: vec![0.5, 0.5, 0.0, 0.0],
        top_k: 3,
        context: None,
        tenant_id: None,
    };
    let r1 = pipeline.execute(&input).expect("execute 1");
    let r2 = pipeline.execute(&input).expect("execute 2");
    assert_eq!(r1.result, r2.result);
}

#[test]
fn smoke_pipeline_rejects_empty_query() {
    let pipeline = make_pipeline();
    let input = ContextInput {
        query_vector: vec![],
        top_k: 1,
        context: None,
        tenant_id: None,
    };
    assert!(pipeline.execute(&input).is_err());
}

#[test]
fn smoke_metrics_work() {
    use kce_core::traits::DistanceMetric;
    use kce_metrics::cosine::CosineMetric;
    use kce_metrics::sinkhorn::SinkhornMetric;
    use kce_metrics::wasserstein::WassersteinMetric;

    let cosine = CosineMetric::new();
    let sinkhorn = SinkhornMetric::new(kce_metrics::sinkhorn::SinkhornConfig::default());
    let wasserstein = WassersteinMetric::new();

    let a = [1.0, 0.0];
    let b = [0.0, 1.0];

    assert!(cosine.compute(&a, &b).is_ok());
    assert!(sinkhorn.compute(&a, &b).is_ok());
    assert!(wasserstein.compute(&a, &b).is_ok());
}

#[test]
fn smoke_graph_expand() {
    use kce_graph::graph::SemanticGraph;

    let mut g = SemanticGraph::new();
    g.add_edge(1, 2, 0.8);
    g.add_edge(1, 3, 0.6);
    g.add_edge(2, 4, 0.5);

    let result = g.expand(&[1], 2, 10);
    assert!(result.nodes.contains(&1));
    assert!(result.nodes.contains(&2));
    assert!(result.nodes.contains(&3));
}

#[test]
fn smoke_ecma_lifecycle() {
    use kce_core::types::{EcmaNode, NodeState};
    use kce_ecma::engine::EcmaEngine;

    let engine = EcmaEngine::with_defaults();
    let mut node = EcmaNode {
        id: 1,
        state: NodeState::Stem,
        usage_count: 15,
        entropy: 0.2,
        connections: 8,
        maturity: 0.0,
    };
    engine.process_node(&mut node).expect("process");
    assert_eq!(node.state, NodeState::Specialized);
    assert!(node.maturity > 0.0);
}

#[test]
fn smoke_mce_encode_execute() {
    use kce_core::types::{EcmaNode, MrnaIntent, NodeState};
    use kce_mce::engine::MceEngine;

    let engine = MceEngine::with_defaults();
    let nodes = vec![EcmaNode {
        id: 42,
        state: NodeState::Specialized,
        usage_count: 15,
        entropy: 0.2,
        connections: 8,
        maturity: 0.87,
    }];
    let mrna = engine.encode(&nodes, MrnaIntent::RiskEval);
    let result = engine.execute(&mrna).expect("execute");
    assert!(result.feedback.success);
}

#[test]
fn smoke_aco_pheromone() {
    use kce_aco::pheromone::PheromoneMap;

    let mut map = PheromoneMap::new();
    map.deposit(1, 2, 0.5);
    assert!((map.get(1, 2) - 1.5).abs() < 1e-10);
}

#[test]
fn smoke_ais_detection() {
    use kce_ais::detection::ImmuneDetector;

    let mut detector = ImmuneDetector::default();
    detector.add_pattern(vec![1.0, 0.0]);
    detector.add_pattern(vec![0.0, 1.0]);
    let result = detector.detect(&[1.0, 0.1]).expect("detect");
    assert!(!result.is_anomaly);
}

#[test]
fn smoke_ais_classifier() {
    use kce_ais::classifier::{Classification, SelfNonSelfClassifier};

    let mut cls = SelfNonSelfClassifier::default();
    cls.register_self(vec![1.0, 0.0]);
    cls.register_self(vec![0.0, 1.0]);
    cls.register_self(vec![0.5, 0.5]);
    assert_eq!(cls.classify(&[1.0, 0.05]), Classification::Self_);
}

// ── FT-010: Concurrent stress test ──────────────────────────────────────────

/// Spawn 50 threads doing concurrent `pipeline.execute()` calls and
/// verify every result is Ok.
#[test]
fn stress_concurrent_pipeline_execute() {
    use std::sync::Arc;
    use std::thread;

    let pipeline = Arc::new(make_pipeline());
    let mut handles = Vec::with_capacity(50);

    for i in 0..50 {
        let p = Arc::clone(&pipeline);
        handles.push(thread::spawn(move || {
            let input = ContextInput {
                query_vector: vec![0.1 * i as f64, 0.9, 0.0, 0.0],
                top_k: 2,
                context: Some(format!("stress_{}", i)),
                tenant_id: "t1".into(),
            };
            let result = p.execute(&input);
            assert!(result.is_ok(), "thread {} failed: {:?}", i, result.err());
        }));
    }

    for (i, handle) in handles.into_iter().enumerate() {
        handle
            .join()
            .unwrap_or_else(|_| panic!("thread {} panicked", i));
    }
}

// ── FT-009: Resilience integration test ─────────────────────────────────────

/// Verify that `execute_with_resilience` works end-to-end.
#[test]
fn resilience_execute_with_resilience_works() {
    let pipeline = make_pipeline();
    let input = ContextInput {
        query_vector: vec![1.0, 0.0, 0.0, 0.0],
        top_k: 2,
        context: Some("resilience_test".into()),
        tenant_id: None,
    };
    let output = pipeline
        .execute_with_resilience(&input, "req_resilience_1")
        .expect("resilience execute");
    assert!(output.pipeline_ms > 0.0);
    assert_eq!(output.stages.len(), 10);
}

/// Verify that the circuit breaker starts in Closed state.
#[test]
fn resilience_circuit_breaker_closed() {
    use kce_pipeline::orchestrator::CircuitState;

    let pipeline = make_pipeline();
    let cb = pipeline.circuit_breaker.lock();
    assert_eq!(cb.state(), CircuitState::Closed);
}

/// Verify that metrics are updated after resilience execution.
#[test]
fn resilience_metrics_updated() {
    let pipeline = make_pipeline();
    let input = ContextInput {
        query_vector: vec![1.0, 0.0, 0.0, 0.0],
        top_k: 1,
        context: None,
        tenant_id: None,
    };
    let _ = pipeline.execute_with_resilience(&input, "req_metrics_1");

    use std::sync::atomic::Ordering;
    let total = pipeline.metrics.total_count.load(Ordering::Relaxed);
    assert!(total >= 1, "expected total_count >= 1, got {}", total);
}

// ── FT-011: Stage identification in errors ──────────────────────────────────

/// Verify that pipeline errors carry the stage name.
#[test]
fn stage_error_contains_stage_name() {
    let pipeline = make_pipeline();
    let input = ContextInput {
        query_vector: vec![],
        top_k: 1,
        context: None,
        tenant_id: None,
    };
    let err = pipeline.execute(&input).expect_err("should fail");
    let msg = err.to_string();
    assert!(
        msg.contains("validate"),
        "expected stage name 'validate' in error, got: {}",
        msg
    );
}

// ── FT-010: Concurrent stress test via resilience path ──────────────────────

/// 50 threads doing concurrent `execute_with_resilience()` calls.
#[test]
fn stress_concurrent_resilience() {
    use std::sync::Arc;
    use std::thread;

    let pipeline = Arc::new(make_pipeline());
    let mut handles = Vec::with_capacity(50);

    for i in 0..50 {
        let p = Arc::clone(&pipeline);
        handles.push(thread::spawn(move || {
            let input = ContextInput {
                query_vector: vec![0.5, 0.5, 0.0, 0.0],
                top_k: 2,
                context: None,
                tenant_id: "t1".into(),
            };
            let request_id = format!("req_stress_{}", i);
            let result = p.execute_with_resilience(&input, &request_id);
            assert!(
                result.is_ok(),
                "thread {} failed: {:?}",
                i,
                result.err()
            );
        }));
    }

    for (i, handle) in handles.into_iter().enumerate() {
        handle
            .join()
            .unwrap_or_else(|_| panic!("thread {} panicked", i));
    }
}
