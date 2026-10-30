//! FT-030 — Distributed Consensus (Raft-Lite & CRDTs).
//!
//! Eventual consistency for data (CRDTs) and Strong consistency for control (Raft-Lite).

use std::collections::HashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── CRDTs (Eventual Consistency) ─────────────────────────────────────────────

/// LWW (Last-Write-Wins) Element for CRDTs (FT-030 AC-011).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LwwElement<T: Clone> {
    pub value: T,
    pub timestamp: u64,
    pub source_node_id: Uuid,
}

impl<T: Clone> LwwElement<T> {
    pub fn merge(&mut self, other: &Self) {
        if other.timestamp > self.timestamp || (other.timestamp == self.timestamp && other.source_node_id > self.source_node_id) {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
            self.source_node_id = other.source_node_id;
        }
    }
}

/// CRDT for Pheromone intensities.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CrdtPheromones {
    /// Maps edge (from, to) to LWW pheromone value.
    pub entries: HashMap<(u64, u64), LwwElement<f64>>,
}

impl CrdtPheromones {
    pub fn update(&mut self, from: u64, to: u64, value: f64, timestamp: u64, node_id: Uuid) {
        let entry = self.entries.entry((from, to)).or_insert(LwwElement {
            value,
            timestamp,
            source_node_id: node_id,
        });
        entry.merge(&LwwElement {
            value,
            timestamp,
            source_node_id: node_id,
        });
    }

    pub fn merge(&mut self, other: &Self) {
        for (k, v) in &other.entries {
            self.entries.entry(*k).or_insert(v.clone()).merge(v);
        }
    }
}

// ── Raft-Lite (Strong Consistency) ───────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RaftRole {
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftLogEntry {
    pub index: u64,
    pub term: u64,
    pub command: String,
    pub payload: Vec<u8>,
}

/// Raft-Lite implementation (FT-030 AC-012).
pub struct RaftLite {
    pub local_id: Uuid,
    role: RwLock<RaftRole>,
    current_term: RwLock<u64>,
    voted_for: RwLock<Option<Uuid>>,
    log: RwLock<Vec<RaftLogEntry>>,
    commit_index: RwLock<u64>,
}

impl RaftLite {
    pub fn new(local_id: Uuid) -> Self {
        Self {
            local_id,
            role: RwLock::new(RaftRole::Follower),
            current_term: RwLock::new(0),
            voted_for: RwLock::new(None),
            log: RwLock::new(Vec::new()),
            commit_index: RwLock::new(0),
        }
    }

    pub fn state(&self) -> RaftRole {
        *self.role.read()
    }

    pub fn term(&self) -> u64 {
        *self.current_term.read()
    }

    /// Simplified Leader Election trigger.
    pub fn start_election(&self) {
        let mut role = self.role.write();
        let mut term = self.current_term.write();
        let mut voted_for = self.voted_for.write();

        *role = RaftRole::Candidate;
        *term += 1;
        *voted_for = Some(self.local_id);
        
        tracing::info!(term = *term, "Raft: started election");
    }

    /// Promote to leader.
    pub fn promote(&self) {
        let mut role = self.role.write();
        *role = RaftRole::Leader;
        tracing::info!("Raft: became leader for term {}", *self.current_term.read());
    }

    /// Append entry to log (FT-030 AC-013).
    pub fn append(&self, command: String, payload: Vec<u8>) -> u64 {
        let mut log = self.log.write();
        let index = log.len() as u64 + 1;
        log.push(RaftLogEntry {
            index,
            term: *self.current_term.read(),
            command,
            payload,
        });
        index
    }
}
