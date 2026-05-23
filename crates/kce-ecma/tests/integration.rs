//! Integration tests for ECMA: lifecycle evolution, MCE feedback, and persistence.

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]

use kce_core::types::{ActionFeedback, EcmaNode, NodeState};
use kce_ecma::engine::EcmaEngine;
use kce_ecma::state_machine::EcmaConfig;
use kce_storage::KineSQL;

fn make_db() -> KineSQL {
    let dir = std::env::temp_dir().join(format!("kce_ecma_integration_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let wal_path = dir.join(format!(
        "integration_{}.wal",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    KineSQL::open(&wal_path).expect("open")
}

#[test]
fn full_lifecycle_with_feedback_and_persistence() {
    // 1. Create a node in Stem state
    let mut node = EcmaNode {
        id: 100,
        state: NodeState::Stem,
        usage_count: 0,
        entropy: 0.3,
        connections: 0,
        maturity: 0.0,
    };

    let config = EcmaConfig::default();
    let engine = EcmaEngine::new(config);

    // 2. Advance usage and evolve: Stem -> Progenitor
    node.usage_count = 6;
    node.connections = 2;
    engine.process_node(&mut node).expect("process stem");
    assert_eq!(node.state, NodeState::Progenitor, "should evolve to Progenitor");
    let progenitor_maturity = node.maturity;
    assert!(progenitor_maturity > 0.0, "maturity should be positive after evolution");

    // 3. Further advance: Progenitor -> Specialized
    node.usage_count = 12;
    node.connections = 5;
    node.entropy = 0.1;
    engine.process_node(&mut node).expect("process progenitor");
    assert_eq!(node.state, NodeState::Specialized, "should evolve to Specialized");
    assert!(node.maturity > progenitor_maturity, "maturity should increase");

    // 4. Apply MCE feedback (simulating successful execution)
    let feedback = ActionFeedback {
        success: true,
        maturity_delta: 0.05,
    };
    let maturity_before = node.maturity;
    EcmaEngine::apply_feedback(&mut node, &feedback);
    assert!(
        node.maturity > maturity_before,
        "maturity should increase from feedback"
    );
    assert!(
        node.maturity <= 1.0,
        "maturity should be clamped to 1.0"
    );

    // 5. Persist to KineSQL and reload
    let mut db = make_db();
    EcmaEngine::persist_node(&node, &mut db).expect("persist");

    let loaded = EcmaEngine::load_node(100, &db).expect("load");
    let loaded = loaded.expect("node should exist");
    assert_eq!(loaded.id, node.id);
    assert_eq!(loaded.state, NodeState::Specialized);
    assert!((loaded.maturity - node.maturity).abs() < f64::EPSILON);
    assert_eq!(loaded.usage_count, node.usage_count);
    assert_eq!(loaded.connections, node.connections);

    // 6. Non-existent node returns None
    let missing = EcmaEngine::load_node(999, &db).expect("load missing");
    assert!(missing.is_none());
}

#[test]
fn connections_gate_progenitor_to_specialized() {
    let config = EcmaConfig::default();
    let engine = EcmaEngine::new(config);

    // Enough usage and low entropy, but connections < 3
    let mut node = EcmaNode {
        id: 200,
        state: NodeState::Progenitor,
        usage_count: 15,
        entropy: 0.1,
        connections: 2, // below the default threshold of 3
        maturity: 0.0,
    };

    engine.process_node(&mut node).expect("process");
    assert_eq!(
        node.state,
        NodeState::Progenitor,
        "should stay Progenitor without enough connections"
    );

    // Now add enough connections
    node.connections = 4;
    engine.process_node(&mut node).expect("process");
    assert_eq!(
        node.state,
        NodeState::Specialized,
        "should evolve to Specialized with enough connections"
    );
}

#[test]
fn feedback_clamped_at_boundaries() {
    let mut node = EcmaNode {
        id: 300,
        state: NodeState::Specialized,
        usage_count: 20,
        entropy: 0.1,
        connections: 10,
        maturity: 0.98,
    };

    // Feedback that would push above 1.0
    let feedback = ActionFeedback {
        success: true,
        maturity_delta: 0.5,
    };
    EcmaEngine::apply_feedback(&mut node, &feedback);
    assert!((node.maturity - 1.0).abs() < f64::EPSILON, "should clamp to 1.0");

    // Feedback that would push below 0.0
    let negative_feedback = ActionFeedback {
        success: false,
        maturity_delta: -2.0,
    };
    EcmaEngine::apply_feedback(&mut node, &negative_feedback);
    assert!((node.maturity - 0.0).abs() < f64::EPSILON, "should clamp to 0.0");
}

#[test]
fn persisted_node_survives_full_ecma_mce_roundtrip() {
    let mut db = make_db();
    let mut node = EcmaNode {
        id: 400,
        state: NodeState::Stem,
        usage_count: 6,
        entropy: 0.2,
        connections: 4,
        maturity: 0.0,
    };

    let engine = EcmaEngine::with_defaults();
    // Evolve to Progenitor
    engine.process_node(&mut node).expect("process");
    assert_eq!(node.state, NodeState::Progenitor);

    // Persist
    EcmaEngine::persist_node(&node, &mut db).expect("persist");

    // Reload, apply feedback, re-persist
    let mut reloaded = EcmaEngine::load_node(400, &db)
        .expect("load")
        .expect("exists");

    // Simulate MCE feedback
    EcmaEngine::apply_feedback(
        &mut reloaded,
        &ActionFeedback {
            success: true,
            maturity_delta: 0.1,
        },
    );
    EcmaEngine::persist_node(&reloaded, &mut db).expect("re-persist");

    // Final load
    let final_node = EcmaEngine::load_node(400, &db)
        .expect("final load")
        .expect("exists");
    assert_eq!(final_node.state, NodeState::Progenitor);
    assert!(final_node.maturity > 0.0, "maturity should reflect feedback");
}
