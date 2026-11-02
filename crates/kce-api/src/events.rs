//! Engine event bus (SSE payloads).

use serde::Serialize;

/// Broadcast channel capacity for engine events.
pub const EVENT_CHANNEL_CAPACITY: usize = 64;

/// Events emitted by the engine and streamed over `GET /events`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EngineEvent {
    /// A query completed successfully.
    QueryCompleted {
        /// End-to-end pipeline latency in milliseconds.
        latency_ms: f64,
        /// Executed action name (e.g. `classify`).
        action: String,
    },
    /// A node was created via `POST /nodes`.
    NodeCreated {
        /// New node id.
        id: u64,
    },
    /// A node was removed via `DELETE /nodes/:id`.
    NodeRemoved {
        /// Removed node id.
        id: u64,
    },
    /// Pheromone was deposited via `POST /pheromone/deposit`.
    PheromoneDeposited {
        /// Source node.
        from: u64,
        /// Target node.
        to: u64,
        /// Resulting pheromone level.
        level: f64,
    },
    /// AIS anomaly detected by `POST /ais/probe`.
    AisAnomaly {
        /// Anomaly score.
        score: f64,
        /// Whether the vector would be blocked.
        blocked: bool,
    },
}

/// Handle type for the event broadcast channel.
pub type EngineEventTx = tokio::sync::broadcast::Sender<EngineEvent>;
