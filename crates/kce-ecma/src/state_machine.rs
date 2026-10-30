//! ECMA state machine — node lifecycle management.
//!
//! Nodes evolve through: `Stem -> Progenitor -> Specialized` based on usage
//! and entropy.  Obsolete nodes degrade to `Apoptosis`.

use kce_core::caputo::CaputoState;
use kce_core::error::KceError;
use kce_core::types::{EcmaNode, NodeState};
use serde::{Deserialize, Serialize};

/// Configuration thresholds for state transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcmaConfig {
    /// Usage count to transition Stem -> Progenitor.
    pub stem_to_progenitor_usage: u64,
    /// Usage count to transition Progenitor -> Specialized.
    pub progenitor_to_specialized_usage: u64,
    /// Minimum connections required for Progenitor -> Specialized.
    pub progenitor_to_specialized_connections: usize,
    /// Entropy threshold above which a node degrades to Apoptosis.
    pub apoptosis_entropy_threshold: f64,
    /// Usage count below which a node degrades to Apoptosis (combined with entropy).
    pub apoptosis_usage_threshold: u64,
}

impl Default for EcmaConfig {
    fn default() -> Self {
        Self {
            stem_to_progenitor_usage: 5,
            progenitor_to_specialized_usage: 10,
            progenitor_to_specialized_connections: 3,
            apoptosis_entropy_threshold: 0.9,
            apoptosis_usage_threshold: 2,
        }
    }
}

/// Validate and apply a state transition.
pub fn transition(node: &mut EcmaNode, config: &EcmaConfig) -> Result<(), KceError> {
    let new_state = match node.state {
        NodeState::Stem => {
            if node.usage_count >= config.stem_to_progenitor_usage {
                NodeState::Progenitor
            } else {
                NodeState::Stem
            }
        }
        NodeState::Progenitor => {
            if node.usage_count >= config.progenitor_to_specialized_usage
                && node.entropy < config.apoptosis_entropy_threshold
                && node.connections >= config.progenitor_to_specialized_connections
            {
                NodeState::Specialized
            } else if node.entropy >= config.apoptosis_entropy_threshold
                && node.usage_count <= config.apoptosis_usage_threshold
            {
                NodeState::Apoptosis
            } else {
                NodeState::Progenitor
            }
        }
        NodeState::Specialized => {
            if node.entropy >= config.apoptosis_entropy_threshold
                && node.usage_count <= config.apoptosis_usage_threshold
            {
                NodeState::Apoptosis
            } else {
                NodeState::Specialized
            }
        }
        NodeState::Apoptosis => {
            return Err(KceError::InvalidTransition {
                from: node.state.to_string(),
                to: "any".into(),
            });
        }
    };

    if new_state != node.state {
        node.state = new_state;
    }
    Ok(())
}

/// Compute maturity score for a node (0.0 .. 1.0).
pub fn maturity(node: &EcmaNode) -> f64 {
    let state_factor = match node.state {
        NodeState::Stem => 0.1,
        NodeState::Progenitor => 0.4,
        NodeState::Specialized => 0.9,
        NodeState::Apoptosis => 0.0,
    };

    let usage_factor = (node.usage_count as f64 / 20.0).min(1.0);
    let entropy_factor = 1.0 - node.entropy;
    let connection_factor = (node.connections as f64 / 10.0).min(1.0);

    let score =
        0.3 * state_factor + 0.3 * usage_factor + 0.2 * entropy_factor + 0.2 * connection_factor;

    score.clamp(0.0, 1.0)
}

/// FT-037: Evaluates entity lifecycle stage driven by Caputo fractional memory intensity and order alpha.
///
/// Modulates alpha dynamically:
/// - `Stem`: alpha = 0.90 (high volatility, rapid forgetting if unreinforced)
/// - `Progenitor`: alpha = 0.55 (intermediate retention)
/// - `Specialized`: alpha = 0.15 (cortical consolidation, almost immune to apoptosis)
/// - `Apoptosis`: vitality fell below survival threshold (< 0.05)
pub fn evaluate_caputo_lifecycle(state: &mut CaputoState, now_ms: u64) -> NodeState {
    let vitality = state.peek_intensity(now_ms);

    if vitality < 0.05 {
        NodeState::Apoptosis
    } else if vitality > 50.0 && state.alpha > 0.20 {
        state.modulate_alpha(0.15, now_ms);
        NodeState::Specialized
    } else if vitality > 15.0 && state.alpha > 0.60 {
        state.modulate_alpha(0.55, now_ms);
        NodeState::Progenitor
    } else {
        NodeState::Stem
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn make_node(state: NodeState, usage: u64, entropy: f64, connections: usize) -> EcmaNode {
        EcmaNode {
            id: 1,
            state,
            usage_count: usage,
            entropy,
            connections,
            maturity: 0.0,
        }
    }

    #[test]
    fn stem_to_progenitor() {
        let config = EcmaConfig::default();
        let mut node = make_node(NodeState::Stem, 5, 0.3, 3);
        transition(&mut node, &config).expect("transition");
        assert_eq!(node.state, NodeState::Progenitor);
    }

    #[test]
    fn progenitor_to_specialized() {
        let config = EcmaConfig::default();
        let mut node = make_node(NodeState::Progenitor, 10, 0.2, 5);
        transition(&mut node, &config).expect("transition");
        assert_eq!(node.state, NodeState::Specialized);
    }

    #[test]
    fn progenitor_to_specialized_requires_connections() {
        let config = EcmaConfig::default();
        // Has enough usage but not enough connections
        let mut node = make_node(NodeState::Progenitor, 10, 0.2, 2);
        transition(&mut node, &config).expect("transition");
        assert_eq!(node.state, NodeState::Progenitor);
    }

    #[test]
    fn degrade_to_apoptosis() {
        let config = EcmaConfig::default();
        let mut node = make_node(NodeState::Progenitor, 1, 0.95, 0);
        transition(&mut node, &config).expect("transition");
        assert_eq!(node.state, NodeState::Apoptosis);
    }

    #[test]
    fn apoptosis_is_terminal() {
        let config = EcmaConfig::default();
        let mut node = make_node(NodeState::Apoptosis, 0, 0.5, 0);
        let result = transition(&mut node, &config);
        assert!(result.is_err());
    }

    #[test]
    fn stem_stays_stem_below_threshold() {
        let config = EcmaConfig::default();
        let mut node = make_node(NodeState::Stem, 2, 0.3, 1);
        transition(&mut node, &config).expect("transition");
        assert_eq!(node.state, NodeState::Stem);
    }

    #[test]
    fn maturity_range() {
        let states = [
            NodeState::Stem,
            NodeState::Progenitor,
            NodeState::Specialized,
            NodeState::Apoptosis,
        ];
        for state in states {
            let node = make_node(state, 10, 0.5, 5);
            let m = maturity(&node);
            assert!(
                (0.0..=1.0).contains(&m),
                "maturity {} out of range for {:?}",
                m,
                state
            );
        }
    }

    #[test]
    fn specialized_has_higher_maturity_than_stem() {
        let stem = make_node(NodeState::Stem, 0, 0.9, 0);
        let spec = make_node(NodeState::Specialized, 20, 0.1, 10);
        assert!(maturity(&spec) > maturity(&stem));
    }

    #[test]
    fn test_caputo_lifecycle_transitions() {
        let mut state = CaputoState::new(0.90, 80.0, 1000);

        // High vitality promotes to Specialized and reduces alpha to 0.15
        let stage = evaluate_caputo_lifecycle(&mut state, 1000);
        assert_eq!(stage, NodeState::Specialized);
        assert_eq!(state.alpha, 0.15);

        // An unreinforced volatile state (alpha = 0.95) decaying after 100 days
        let mut decaying = CaputoState::new(0.95, 2.0, 1000);
        let hundred_days_ms = 1000 + 100 * 24 * 3600 * 1000;
        let stage_decayed = evaluate_caputo_lifecycle(&mut decaying, hundred_days_ms);
        assert_eq!(stage_decayed, NodeState::Apoptosis);
    }
}
