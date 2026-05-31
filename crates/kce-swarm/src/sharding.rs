//! FT-028 — Distributed Sharding (ACTA).
//!
//! Implements Ant-Colony Task Allocation (ACTA) for dynamic data distribution.

use std::collections::HashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a cluster node.
pub type NodeId = Uuid;

/// ACTA Node Load telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeLoad {
    pub cpu_utilization: f64,
    pub memory_used_bytes: u64,
    pub active_queries: usize,
}

impl Default for NodeLoad {
    fn default() -> Self {
        Self {
            cpu_utilization: 0.0,
            memory_used_bytes: 0,
            active_queries: 0,
        }
    }
}

/// A Shard definition.
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Shard {
    pub id: u32,
    pub tenant_id: String,
}

/// Shard Router for determining which node hosts which data.
pub struct ShardRouter {
    pub local_node_id: NodeId,
    /// Maps shard to primary node.
    shard_map: RwLock<HashMap<Shard, NodeId>>,
    /// Node load table for ACTA.
    node_loads: RwLock<HashMap<NodeId, NodeLoad>>,
}

impl ShardRouter {
    pub fn new(local_node_id: NodeId) -> Self {
        Self {
            local_node_id,
            shard_map: RwLock::new(HashMap::new()),
            node_loads: RwLock::new(HashMap::new()),
        }
    }

    /// Calculate shard for a given tenant and context hash.
    pub fn calculate_shard(&self, tenant_id: &str, context_hash: u64) -> Shard {
        // Simple hash-based partitioning (FT-028 AC-010)
        let shard_id = (context_hash % 256) as u32;
        Shard {
            id: shard_id,
            tenant_id: tenant_id.to_string(),
        }
    }

    /// Get target node for a shard, potentially delegating based on ACTA.
    pub fn route(&self, shard: &Shard) -> NodeId {
        let shard_map = self.shard_map.read();
        let primary = shard_map.get(shard).cloned().unwrap_or(self.local_node_id);
        
        // ACTA Delegation (FT-028 AC-011)
        if primary == self.local_node_id {
            let loads = self.node_loads.read();
            if let Some(load) = loads.get(&self.local_node_id) {
                if load.cpu_utilization > 0.85 {
                    // Find least loaded node hosting a replica (simplified for MVP: any other node)
                    if let Some(alternate) = loads.iter()
                        .filter(|(id, _)| **id != self.local_node_id)
                        .min_by(|a, b| a.1.cpu_utilization.partial_cmp(&b.1.cpu_utilization).unwrap())
                    {
                        tracing::debug!(?shard, "ACTA: delegating query to node {}", alternate.0);
                        return *alternate.0;
                    }
                }
            }
        }
        
        primary
    }

    pub fn update_load(&self, node_id: NodeId, load: NodeLoad) {
        self.node_loads.write().insert(node_id, load);
    }

    pub fn assign_shard(&self, shard: Shard, node_id: NodeId) {
        self.shard_map.write().insert(shard, node_id);
    }
}
