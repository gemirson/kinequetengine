//! mRNA encoding and validation.

use kce_core::error::KceError;
use kce_core::types::{EcmaNode, Mrna, MrnaIntent};
use serde::{Deserialize, Serialize};

/// Configuration for mRNA encoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MceConfig {
    /// Default TTL in milliseconds.
    pub default_ttl_ms: u64,
    /// Minimum priority (1-10).
    pub min_priority: u8,
    /// Maximum priority (1-10).
    pub max_priority: u8,
}

impl Default for MceConfig {
    fn default() -> Self {
        Self {
            default_ttl_ms: 5000,
            min_priority: 1,
            max_priority: 10,
        }
    }
}

/// Encode ECMA nodes into an mRNA instruction.
///
/// Each node is encoded as 48 bytes in the payload:
/// - id (u64), state_ordinal (u64), usage_count (u64),
///   entropy (f64), connections (u64), maturity (f64).
///
/// The `original_size_bytes` is computed as the JSON byte length of the
/// nodes, so `compression_ratio` reflects real compression vs JSON.
pub fn encode(nodes: &[EcmaNode], intent: MrnaIntent, config: &MceConfig) -> Mrna {
    let mut payload = Vec::with_capacity(nodes.len() * 48);
    for node in nodes {
        payload.extend_from_slice(&node.id.to_le_bytes());
        let state_ord: u64 = match node.state {
            kce_core::types::NodeState::Stem => 0,
            kce_core::types::NodeState::Progenitor => 1,
            kce_core::types::NodeState::Specialized => 2,
            kce_core::types::NodeState::Apoptosis => 3,
        };
        payload.extend_from_slice(&state_ord.to_le_bytes());
        payload.extend_from_slice(&node.usage_count.to_le_bytes());
        payload.extend_from_slice(&node.entropy.to_le_bytes());
        payload.extend_from_slice(&(node.connections as u64).to_le_bytes());
        payload.extend_from_slice(&node.maturity.to_le_bytes());
    }

    let avg_maturity: f64 = if nodes.is_empty() {
        0.0
    } else {
        nodes.iter().map(|n| n.maturity).sum::<f64>() / nodes.len() as f64
    };
    let priority = (avg_maturity * config.max_priority as f64).round() as u8;
    let priority = priority.clamp(config.min_priority, config.max_priority);

    // Compute original size as JSON byte length (the unencoded form).
    let original_size = serde_json::to_vec(nodes)
        .map(|v| v.len())
        .unwrap_or(nodes.len() * 128);

    let payload_size = payload.len();
    let compression_ratio = if original_size > 0 {
        payload_size as f64 / original_size as f64
    } else {
        0.0
    };

    Mrna {
        intent,
        priority,
        ttl_ms: config.default_ttl_ms,
        payload,
        payload_size_bytes: payload_size,
        original_size_bytes: original_size,
        compression_ratio,
    }
}

/// Validate an mRNA instruction.
pub fn validate(mrna: &Mrna) -> Result<(), KceError> {
    if mrna.payload.is_empty() {
        return Err(KceError::Validation("mRNA payload is empty".into()));
    }
    if mrna.ttl_ms == 0 {
        return Err(KceError::Validation("mRNA TTL is zero".into()));
    }
    Ok(())
}

/// Check if an mRNA has expired given its creation timestamp.
pub fn is_expired(mrna: &Mrna, created_at_ms: u64, now_ms: u64) -> bool {
    now_ms.saturating_sub(created_at_ms) > mrna.ttl_ms
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use kce_core::types::NodeState;

    fn sample_nodes() -> Vec<EcmaNode> {
        vec![
            EcmaNode {
                id: 42,
                state: NodeState::Specialized,
                usage_count: 15,
                entropy: 0.2,
                connections: 8,
                maturity: 0.87,
            },
            EcmaNode {
                id: 17,
                state: NodeState::Progenitor,
                usage_count: 8,
                entropy: 0.4,
                connections: 4,
                maturity: 0.55,
            },
        ]
    }

    #[test]
    fn encode_produces_valid_mrna() {
        let config = MceConfig::default();
        let mrna = encode(&sample_nodes(), MrnaIntent::RiskEval, &config);
        assert_eq!(mrna.intent, MrnaIntent::RiskEval);
        assert!(mrna.priority >= 1 && mrna.priority <= 10);
        assert!(mrna.ttl_ms > 0);
        assert!(!mrna.payload.is_empty());
    }

    #[test]
    fn encode_compression_ratio() {
        let config = MceConfig::default();
        let mrna = encode(&sample_nodes(), MrnaIntent::Classify, &config);
        assert!(mrna.compression_ratio > 0.0);
    }

    #[test]
    fn encode_empty_nodes() {
        let config = MceConfig::default();
        let mrna = encode(&[], MrnaIntent::Alert, &config);
        assert_eq!(mrna.payload.len(), 0);
    }

    #[test]
    fn validate_empty_payload_errors() {
        let mrna = Mrna {
            intent: MrnaIntent::RiskEval,
            priority: 5,
            ttl_ms: 1000,
            payload: vec![],
            payload_size_bytes: 0,
            original_size_bytes: 0,
            compression_ratio: 0.0,
        };
        assert!(validate(&mrna).is_err());
    }

    #[test]
    fn validate_zero_ttl_errors() {
        let mrna = Mrna {
            intent: MrnaIntent::RiskEval,
            priority: 5,
            ttl_ms: 0,
            payload: vec![0x01],
            payload_size_bytes: 1,
            original_size_bytes: 1,
            compression_ratio: 1.0,
        };
        assert!(validate(&mrna).is_err());
    }

    #[test]
    fn expired_detection_works() {
        let mrna = Mrna {
            intent: MrnaIntent::RiskEval,
            priority: 5,
            ttl_ms: 100,
            payload: vec![0x01],
            payload_size_bytes: 1,
            original_size_bytes: 1,
            compression_ratio: 1.0,
        };
        assert!(is_expired(&mrna, 1000, 2000));
        assert!(!is_expired(&mrna, 1000, 1050));
    }
}
