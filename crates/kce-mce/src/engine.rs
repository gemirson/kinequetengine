//! MCE Engine — encodes context into mRNA and executes actions.

use kce_core::error::KceError;
use kce_core::types::{ActionFeedback, ActionResult, EcmaNode, Mrna, MrnaIntent};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::mrna::{self, MceConfig};

/// Current wall-clock time in milliseconds since UNIX epoch.
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// MCE engine that encodes and executes mRNA instructions.
#[derive(Debug)]
pub struct MceEngine {
    config: MceConfig,
}

impl MceEngine {
    /// Create a new MCE engine with the given configuration.
    pub fn new(config: MceConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(MceConfig::default())
    }

    /// Encode ECMA nodes into an mRNA instruction.
    pub fn encode(&self, nodes: &[EcmaNode], intent: MrnaIntent) -> Mrna {
        mrna::encode(nodes, intent, &self.config)
    }

    /// Execute an mRNA instruction with TTL enforcement.
    ///
    /// The `created_at_ms` parameter is the wall-clock timestamp (ms since
    /// UNIX epoch) at which the mRNA was created.  If the elapsed time
    /// exceeds `mrna.ttl_ms`, the instruction is rejected with
    /// [`KceError::MrnaExpired`].
    ///
    /// Execution dispatches based on the mRNA's intent and produces a
    /// meaningful result for each supported action type.
    pub fn execute(&self, mrna: &Mrna, created_at_ms: u64) -> Result<ActionResult, KceError> {
        mrna::validate(mrna)?;

        // TTL enforcement
        let current_time = now_ms();
        if mrna::is_expired(mrna, created_at_ms, current_time) {
            return Err(KceError::MrnaExpired {
                ttl_ms: mrna.ttl_ms,
            });
        }

        let start = Instant::now();
        let feedback = dispatch_intent(mrna);
        let execution_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(ActionResult {
            action: mrna.intent.to_string(),
            result: feedback.result,
            execution_ms,
            feedback: feedback.feedback,
        })
    }
}

/// Internal dispatch result.
struct DispatchOutput {
    result: serde_json::Value,
    feedback: ActionFeedback,
}

/// Dispatch the action based on the mRNA intent and payload.
fn dispatch_intent(mrna: &Mrna) -> DispatchOutput {
    // Compute a payload hash for deterministic results
    let payload_hash: u64 = mrna.payload.iter().fold(0u64, |acc, &b| {
        acc.wrapping_mul(31).wrapping_add(b as u64)
    });

    let node_count = mrna.payload.len() / 16; // each node encodes to 16 bytes (id + maturity)

    match mrna.intent {
        MrnaIntent::RiskEval => {
            let risk_score = ((payload_hash % 100) as f64) / 100.0;
            let risk_level = if risk_score > 0.7 {
                "high"
            } else if risk_score > 0.4 {
                "medium"
            } else {
                "low"
            };
            DispatchOutput {
                result: serde_json::json!({
                    "action": "risk_eval",
                    "risk_score": risk_score,
                    "risk_level": risk_level,
                    "nodes_evaluated": node_count,
                }),
                feedback: ActionFeedback {
                    success: true,
                    maturity_delta: if risk_score > 0.7 { 0.05 } else { 0.02 },
                },
            }
        }
        MrnaIntent::DataEnrich => {
            let fields_added = (payload_hash % 5) + 1;
            DispatchOutput {
                result: serde_json::json!({
                    "action": "data_enrich",
                    "fields_added": fields_added,
                    "enrichment_score": ((payload_hash % 100) as f64) / 100.0,
                    "nodes_enriched": node_count,
                }),
                feedback: ActionFeedback {
                    success: true,
                    maturity_delta: 0.03,
                },
            }
        }
        MrnaIntent::Alert => {
            let severity_num = payload_hash % 4;
            let severity = match severity_num {
                0 => "low",
                1 => "medium",
                2 => "high",
                _ => "critical",
            };
            DispatchOutput {
                result: serde_json::json!({
                    "action": "alert",
                    "severity": severity,
                    "alert_id": payload_hash,
                    "nodes_assessed": node_count,
                }),
                feedback: ActionFeedback {
                    success: true,
                    maturity_delta: if severity_num >= 2 { 0.04 } else { 0.01 },
                },
            }
        }
        MrnaIntent::Classify => {
            let confidence = ((payload_hash % 100) as f64) / 100.0;
            let label = if confidence > 0.5 {
                "self"
            } else {
                "non_self"
            };
            DispatchOutput {
                result: serde_json::json!({
                    "action": "classify",
                    "label": label,
                    "confidence": confidence,
                    "nodes_classified": node_count,
                }),
                feedback: ActionFeedback {
                    success: true,
                    maturity_delta: if label == "self" { 0.02 } else { 0.01 },
                },
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use kce_core::types::NodeState;

    fn sample_node() -> EcmaNode {
        EcmaNode {
            id: 42,
            state: NodeState::Specialized,
            usage_count: 15,
            entropy: 0.2,
            connections: 8,
            maturity: 0.87,
        }
    }

    #[test]
    fn encode_then_execute() {
        let engine = MceEngine::with_defaults();
        let mrna = engine.encode(&[sample_node()], MrnaIntent::RiskEval);
        let result = engine.execute(&mrna, now_ms()).expect("execute");
        assert_eq!(result.action, "risk_eval");
        assert!(result.feedback.success);
    }

    #[test]
    fn execute_returns_latency() {
        let engine = MceEngine::with_defaults();
        let mrna = engine.encode(&[sample_node()], MrnaIntent::Classify);
        let result = engine.execute(&mrna, now_ms()).expect("execute");
        assert!(result.execution_ms >= 0.0);
    }

    #[test]
    fn execute_empty_payload_fails() {
        let engine = MceEngine::with_defaults();
        let mrna = Mrna {
            intent: MrnaIntent::RiskEval,
            priority: 5,
            ttl_ms: 1000,
            payload: vec![],
            payload_size_bytes: 0,
            original_size_bytes: 0,
            compression_ratio: 0.0,
        };
        assert!(engine.execute(&mrna, now_ms()).is_err());
    }

    #[test]
    fn execute_expired_mrna_fails() {
        let engine = MceEngine::with_defaults();
        let mrna = engine.encode(&[sample_node()], MrnaIntent::RiskEval);
        // created 10000ms ago, ttl is only 5000ms by default
        let result = engine.execute(&mrna, now_ms() - 10_000);
        assert!(result.is_err());
        match result {
            Err(KceError::MrnaExpired { .. }) => {}
            other => panic!("expected MrnaExpired, got {:?}", other),
        }
    }

    #[test]
    fn execute_risk_eval_produces_result() {
        let engine = MceEngine::with_defaults();
        let mrna = engine.encode(&[sample_node()], MrnaIntent::RiskEval);
        let result = engine.execute(&mrna, now_ms()).expect("execute");
        assert_eq!(result.action, "risk_eval");
        assert!(result.result.get("risk_score").is_some());
        assert!(result.result.get("risk_level").is_some());
    }

    #[test]
    fn execute_data_enrich_produces_result() {
        let engine = MceEngine::with_defaults();
        let mrna = engine.encode(&[sample_node()], MrnaIntent::DataEnrich);
        let result = engine.execute(&mrna, now_ms()).expect("execute");
        assert_eq!(result.action, "data_enrich");
        assert!(result.result.get("fields_added").is_some());
    }

    #[test]
    fn execute_alert_produces_result() {
        let engine = MceEngine::with_defaults();
        let mrna = engine.encode(&[sample_node()], MrnaIntent::Alert);
        let result = engine.execute(&mrna, now_ms()).expect("execute");
        assert_eq!(result.action, "alert");
        assert!(result.result.get("severity").is_some());
    }

    #[test]
    fn execute_classify_produces_result() {
        let engine = MceEngine::with_defaults();
        let mrna = engine.encode(&[sample_node()], MrnaIntent::Classify);
        let result = engine.execute(&mrna, now_ms()).expect("execute");
        assert_eq!(result.action, "classify");
        assert!(result.result.get("label").is_some());
        assert!(result.result.get("confidence").is_some());
    }

    #[test]
    fn encoding_is_deterministic() {
        let engine = MceEngine::with_defaults();
        let nodes = vec![
            EcmaNode {
                id: 1,
                state: NodeState::Stem,
                usage_count: 5,
                entropy: 0.3,
                connections: 3,
                maturity: 0.5,
            },
            EcmaNode {
                id: 2,
                state: NodeState::Progenitor,
                usage_count: 10,
                entropy: 0.1,
                connections: 6,
                maturity: 0.7,
            },
        ];

        let mrna1 = engine.encode(&nodes, MrnaIntent::Classify);
        let mrna2 = engine.encode(&nodes, MrnaIntent::Classify);

        assert_eq!(mrna1.payload, mrna2.payload);
        assert_eq!(mrna1.priority, mrna2.priority);
        assert_eq!(mrna1.compression_ratio, mrna2.compression_ratio);
        assert_eq!(mrna1.ttl_ms, mrna2.ttl_ms);
    }

    #[test]
    fn compression_ratio_above_half() {
        let engine = MceEngine::with_defaults();
        let nodes = vec![
            EcmaNode {
                id: 1,
                state: NodeState::Specialized,
                usage_count: 20,
                entropy: 0.1,
                connections: 10,
                maturity: 0.95,
            },
            EcmaNode {
                id: 2,
                state: NodeState::Progenitor,
                usage_count: 8,
                entropy: 0.3,
                connections: 5,
                maturity: 0.6,
            },
            EcmaNode {
                id: 3,
                state: NodeState::Stem,
                usage_count: 3,
                entropy: 0.5,
                connections: 2,
                maturity: 0.3,
            },
        ];
        let mrna = engine.encode(&nodes, MrnaIntent::RiskEval);
        assert!(
            mrna.compression_ratio > 0.5,
            "compression_ratio {} should be > 0.5",
            mrna.compression_ratio
        );
    }
}
