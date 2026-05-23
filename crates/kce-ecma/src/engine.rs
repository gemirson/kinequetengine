//! ECMA Engine — processes context nodes through the lifecycle state machine.

use kce_core::error::KceError;
use kce_core::traits::StorageBackend;
use kce_core::types::{ActionFeedback, EcmaNode};

use crate::state_machine::{self, EcmaConfig};

/// ECMA engine that manages node lifecycle evolution.
#[derive(Debug)]
pub struct EcmaEngine {
    config: EcmaConfig,
}

impl EcmaEngine {
    /// Create a new ECMA engine with the given configuration.
    pub fn new(config: EcmaConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(EcmaConfig::default())
    }

    /// Process a node: apply state transition and update maturity.
    pub fn process_node(&self, node: &mut EcmaNode) -> Result<(), KceError> {
        state_machine::transition(node, &self.config)?;
        node.maturity = state_machine::maturity(node);
        Ok(())
    }

    /// Process a batch of nodes.
    pub fn process_batch(&self, nodes: &mut [EcmaNode]) -> Result<(), KceError> {
        for node in nodes.iter_mut() {
            self.process_node(node)?;
        }
        Ok(())
    }

    /// Apply execution feedback from MCE to an ECMA node.
    ///
    /// Adjusts the node's maturity by the feedback delta, clamped to [0.0, 1.0].
    pub fn apply_feedback(node: &mut EcmaNode, feedback: &ActionFeedback) {
        node.maturity = (node.maturity + feedback.maturity_delta).clamp(0.0, 1.0);
    }

    /// Serialize and persist an ECMA node to a storage backend.
    ///
    /// The node is stored under a key derived from its id: `ecma:node:{id}`.
    ///
    /// # Errors
    ///
    /// Returns [`KceError::Validation`] if serialization fails.
    /// Returns [`KceError::Storage`] if the write fails.
    pub fn persist_node(node: &EcmaNode, db: &mut impl StorageBackend) -> Result<(), KceError> {
        let serialized = serde_json::to_vec(node).map_err(|e| {
            KceError::Validation(format!("failed to serialize node {}: {}", node.id, e))
        })?;
        let key = format!("ecma:node:{}", node.id);
        db.write(key.as_bytes(), &serialized)?;
        Ok(())
    }

    /// Load an ECMA node from a storage backend by id.
    ///
    /// Returns `None` if the node does not exist in storage.
    ///
    /// # Errors
    ///
    /// Returns [`KceError::Validation`] if deserialization fails.
    /// Returns [`KceError::Storage`] if the read fails.
    pub fn load_node(id: u64, db: &impl StorageBackend) -> Result<Option<EcmaNode>, KceError> {
        let key = format!("ecma:node:{}", id);
        match db.read(key.as_bytes())? {
            Some(bytes) => {
                let node: EcmaNode = serde_json::from_slice(&bytes).map_err(|e| {
                    KceError::Validation(format!("failed to deserialize node {}: {}", id, e))
                })?;
                Ok(Some(node))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use kce_core::types::NodeState;

    #[test]
    fn process_node_updates_maturity() {
        let engine = EcmaEngine::with_defaults();
        let mut node = EcmaNode {
            id: 1,
            state: NodeState::Stem,
            usage_count: 0,
            entropy: 0.3,
            connections: 2,
            maturity: 0.0,
        };
        engine.process_node(&mut node).expect("process");
        assert!(node.maturity > 0.0);
    }

    #[test]
    fn process_batch() {
        let engine = EcmaEngine::with_defaults();
        let mut nodes = vec![
            EcmaNode {
                id: 1,
                state: NodeState::Stem,
                usage_count: 6,
                entropy: 0.3,
                connections: 3,
                maturity: 0.0,
            },
            EcmaNode {
                id: 2,
                state: NodeState::Progenitor,
                usage_count: 15,
                entropy: 0.1,
                connections: 8,
                maturity: 0.0,
            },
        ];
        engine.process_batch(&mut nodes).expect("batch");
        assert_eq!(nodes[0].state, NodeState::Progenitor);
        assert_eq!(nodes[1].state, NodeState::Specialized);
    }
}
