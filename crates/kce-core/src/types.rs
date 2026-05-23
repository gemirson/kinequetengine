//! Shared data types for the KineContext Engine.

use serde::{Deserialize, Serialize};

/// Input to the cognitive pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInput {
    /// The query vector for similarity search.
    pub query_vector: Vec<f64>,
    /// Number of top results to return.
    pub top_k: usize,
    /// Optional context label (e.g. "loan_evaluation").
    pub context: Option<String>,
    /// Tenant identifier for multi-tenant isolation.
    pub tenant_id: String,
}

/// Output from the cognitive pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextOutput {
    /// The action that was executed.
    pub action: String,
    /// The result payload.
    pub result: serde_json::Value,
    /// Total pipeline latency in milliseconds.
    pub pipeline_ms: f64,
    /// Per-stage latency breakdown.
    pub stages: std::collections::BTreeMap<String, f64>,
}

/// Health status of the system or a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// All components are healthy.
    Healthy,
    /// Some components are degraded but the system is operational.
    Degraded,
    /// The system is not operational.
    Unhealthy,
}

/// Component-level health check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Overall status.
    pub status: HealthStatus,
    /// WAL health.
    pub wal_ok: bool,
    /// Database / storage health.
    pub db_ok: bool,
    /// Latency within acceptable bounds.
    pub latency_ok: bool,
    /// Application version.
    pub version: String,
    /// Uptime in seconds.
    pub uptime_seconds: u64,
}

/// A single search result with score breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The node / document id.
    pub id: u64,
    /// Combined relevance score.
    pub score: f64,
    /// Cosine similarity component.
    pub cosine_score: f64,
    /// Prime similarity component.
    pub prime_score: f64,
}

/// ECMA node lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeState {
    /// Newly created, immature knowledge.
    Stem,
    /// Growing, gaining connections.
    Progenitor,
    /// Mature, highly relevant knowledge.
    Specialized,
    /// Marked for removal (obsolescence).
    Apoptosis,
}

impl std::fmt::Display for NodeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeState::Stem => write!(f, "Stem"),
            NodeState::Progenitor => write!(f, "Progenitor"),
            NodeState::Specialized => write!(f, "Specialized"),
            NodeState::Apoptosis => write!(f, "Apoptosis"),
        }
    }
}

/// An ECMA node with its lifecycle metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcmaNode {
    /// Node identifier.
    pub id: u64,
    /// Current lifecycle state.
    pub state: NodeState,
    /// Number of times this node has been used.
    pub usage_count: u64,
    /// Entropy measure (0.0 = stable, 1.0 = chaotic).
    pub entropy: f64,
    /// Number of graph connections.
    pub connections: usize,
    /// Maturity score (0.0 .. 1.0).
    pub maturity: f64,
}

/// Supported mRNA intent types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MrnaIntent {
    /// Risk evaluation action.
    RiskEval,
    /// Data enrichment action.
    DataEnrich,
    /// Alert / notification action.
    Alert,
    /// Classification action.
    Classify,
}

impl std::fmt::Display for MrnaIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MrnaIntent::RiskEval => write!(f, "risk_eval"),
            MrnaIntent::DataEnrich => write!(f, "data_enrich"),
            MrnaIntent::Alert => write!(f, "alert"),
            MrnaIntent::Classify => write!(f, "classify"),
        }
    }
}

/// An mRNA instruction — compact executable payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mrna {
    /// The intent / action type.
    pub intent: MrnaIntent,
    /// Execution priority (1-10).
    pub priority: u8,
    /// Time-to-live in milliseconds.
    pub ttl_ms: u64,
    /// Encoded payload bytes.
    pub payload: Vec<u8>,
    /// Size of the encoded payload in bytes.
    pub payload_size_bytes: usize,
    /// Size of the original input in bytes.
    pub original_size_bytes: usize,
    /// Compression ratio (payload / original).
    pub compression_ratio: f64,
}

/// Result of executing an mRNA instruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    /// The action that was executed.
    pub action: String,
    /// The result value.
    pub result: serde_json::Value,
    /// Execution time in milliseconds.
    pub execution_ms: f64,
    /// Feedback for ECMA maturity update.
    pub feedback: ActionFeedback,
}

/// Feedback from action execution to feed back into ECMA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionFeedback {
    /// Whether the action succeeded.
    pub success: bool,
    /// Change in maturity to apply.
    pub maturity_delta: f64,
}

/// A pheromone entry on a graph edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PheromoneEntry {
    /// Edge identifier.
    pub edge_id: u64,
    /// Current pheromone score.
    pub score: f64,
}

/// An antigen stored in the immune memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Antigen {
    /// The anomalous pattern hash.
    pub pattern: String,
    /// Severity level.
    pub severity: Severity,
    /// First time this antigen was seen.
    pub first_seen: u64,
    /// Last time this antigen was seen.
    pub last_seen: u64,
    /// Number of times matched.
    pub match_count: u64,
    /// Expiry in days (0 = permanent).
    pub expiry_days: u32,
    /// Current status.
    pub status: AntigenStatus,
}

/// Severity level for anomalies and antigens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Low severity.
    Low,
    /// Medium severity.
    Medium,
    /// High severity.
    High,
    /// Critical severity.
    Critical,
}

/// Status of a stored antigen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AntigenStatus {
    /// Active and being monitored.
    Active,
    /// Expired, no longer relevant.
    Expired,
    /// Manually resolved.
    Resolved,
}

/// Result of an anomaly detection scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyResult {
    /// Whether the input is considered anomalous.
    pub is_anomaly: bool,
    /// Anomaly score (0.0 = normal, 1.0 = anomalous).
    pub score: f64,
    /// Per-dimension deviation details.
    pub deviations: Vec<Deviation>,
}

/// A single deviation detected in an anomaly scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deviation {
    /// The dimension / feature that deviated.
    pub dimension: String,
    /// The deviation magnitude.
    pub deviation: f64,
    /// Human-readable detail.
    pub detail: String,
}

/// Classification label for self/non-self.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClassificationLabel {
    /// Trusted, known-good context.
    SelfContext,
    /// Unknown, potentially anomalous context.
    NonSelf,
}

/// Result of a self/non-self classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    /// The classification label.
    pub label: ClassificationLabel,
    /// Confidence score (0.0 .. 1.0).
    pub confidence: f64,
    /// Per-feature scoring details.
    pub features: Vec<FeatureScore>,
}

/// A single feature score in a classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureScore {
    /// Feature name.
    pub name: String,
    /// Score value.
    pub score: f64,
    /// Status of this feature.
    pub status: FeatureStatus,
}

/// Status of a feature in classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureStatus {
    /// Within normal bounds.
    Normal,
    /// Outside normal bounds, potentially anomalous.
    Anomalous,
}

/// Amplified context with adjusted priority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmplifiedContext {
    /// Adjusted priority.
    pub priority: u8,
    /// Extended timeout in milliseconds.
    pub timeout_ms: u64,
    /// Increased retry count.
    pub retries: u8,
    /// The amplification factor applied.
    pub amplification_factor: f64,
    /// Time after which amplification decays (ms).
    pub decay_after_ms: u64,
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn node_state_display() {
        assert_eq!(NodeState::Stem.to_string(), "Stem");
        assert_eq!(NodeState::Apoptosis.to_string(), "Apoptosis");
    }

    #[test]
    fn mrna_intent_display() {
        assert_eq!(MrnaIntent::RiskEval.to_string(), "risk_eval");
        assert_eq!(MrnaIntent::Classify.to_string(), "classify");
    }

    #[test]
    fn health_status_equality() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Low < Severity::Critical);
        assert!(Severity::Medium < Severity::High);
    }

    #[test]
    fn context_input_serde_roundtrip() {
        let input = ContextInput {
            query_vector: vec![0.1, 0.2, 0.3],
            top_k: 5,
            context: Some("test".into()),
            tenant_id: "tenant_1".into(),
        };
        let json = serde_json::to_string(&input).expect("serialize");
        let back: ContextInput = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.top_k, 5);
        assert_eq!(back.query_vector.len(), 3);
    }
}
