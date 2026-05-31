# FT-030 — Distributed Consensus (Raft-Lite & CRDTs)

**Module:** Infrastructure | **Version:** v6.0 | **Priority:** P0 — Critical  
**Artifact ID:** FT-030-DISTRIBUTED-CONSENSUS | **Update:** 2026-05-30

---

## 1. Context and Objective

Efficient distributed systems require a balance between performance and data consistency. Attempting to enforce strong consistency across all search and vector telemetry operations severely degrades latency. This specification defines the KCE **Hybrid Consensus**:
1. **Eventual Consistency (via CRDTs)** in the hot data path (pheromones, semantic connections, anomaly logs) to ensure fast responses.
2. **Strong Consistency (via Raft-Lite)** in control path (tenant configurations, access keys, fixed shard assignments and cluster member table) to ensure integrity.

---

## 2. Acceptance Criteria (AC)

### General
- [ ] AC-001: Clean build in Rust.
- [ ] AC-002: No global lock contention on CRDT paths.
- [ ] AC-003: Treatment of network partitions with deterministic conflict resolution.

### Specifics
- [ ] AC-010: Implementation of *State-based* and *Delta-based* CRDTs for eventual synchronization of cognitive graphs and pheromone intensities.
- [ ] AC-011: LWW-Element-Graph (Last-Write-Wins) conflict resolution based on causal temporal watermarks with deterministic tiebreaker by node ID.
- [ ] AC-012: The **Raft-Lite** protocol must manage leader election and log replication with simple quorum ($N/2 + 1$) for writing critical configurations.
- [ ] AC-013: Persistent writing of Raft-Lite log in KineSQL WAL (FT-005) before control transaction commit (Distributed Atomic Commit).
- [ ] AC-014: Automatic detection of Raft-Lite leader loss and re-election in less than 1.5 seconds.
- [ ] AC-015: Synchronization and automatic reintegration of nodes recovered after network partition, applying deltas in the background.

---

## 3. Definition of Done (DoD)

- [ ] Implementation of CRDT structures (`CrdtGraph` and `CrdtPheromones`).
- [ ] Finitely regulated state machine implementation for the Raft-Lite protocol.
- [ ] Automated election testing under node outage scenarios in 3- and 5-node clusters.
- [ ] Integration with KineSQL for recording transaction logs.
- [ ] Unit test coverage greater than 80%.

---

## 4. Usage Examples

### Raft-Lite Proposal Message (JSON)

**Proposal for insertion of a new tenant sent by the leader:**
```json
{
  "term": 3,
  "leader_id": "node-1-uuid",
  "prev_log_index": 142,
  "prev_log_term": 3,
  "entries": [
    {
      "index": 143,
      "term": 3,
      "command": "CREATE_TENANT",
      "payload": { "tenant_id": 4, "secret_hash": "e3b0c442..." }
    }
  ],
  "leader_commit": 142
}
```

### CRDT Conflict Resolution Response (Graph Delta)

**Deterministic merge result of competing graphs:**
```json
{
  "merged_nodes": 12,
  "conflicts_resolved": 3,
  "strategy": "LWW_CAUSAL",
  "local_timestamp": 1780000000000,
  "remote_timestamp": 1780000000050,
  "applied_deltas": ["edge_10->12_updated"]
}
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Entry | Expected Output |
|----|------|---------|----------------|
| UT-001 | Merge CRDT Pheromones | Local $\tau=1.5$, Remote $\tau=2.5$ | Deterministic mathematical convergence |
| UT-002 | LWW-Element-Graph tiebreaker | Identical timestamps, different IDs | The node with the highest alphanumeric ID wins |
| UT-003 | Raft-Lite Initial Election | 3 active nodes, no leader | A node declares candidacy and receives majority votes |
| UT-004 | Minor Log Rejection | Leader with outdated term sends log | Returns error; proposal rejected |

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | Simulated Network Partition | Classic split-brain: the minority partition (2 nodes of a cluster of 5) blocks strong writes, while the majority (3 nodes) elects a new leader and keeps operations active |
| FT-002 | Batch Sync | A node offline for 5 minutes receives only the incremental difference of graphs and pheromones, recovering alignment with the mesh |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Raft-Lite → KineSQL (FT-005) | Raft logs are physically written and downloaded with fsync |
| IT-002 | CRDT → Graph (FT-002) | Nodes and edges of the semantic graph are instantiated and synchronized via CRDT |

---

## 6. CARE Format

**Context:** Consistency and transactional integrity of data and metadata in KCE operating in multi-node environments subject to cloud infrastructure instabilities.

**Assumptions:** The majority of cluster nodes ($N/2 + 1$) are online and reachable. Server hardware clocks are reasonably synchronized via NTP.

**Requirements:** R-001: Raft-Lite Consensus for Control | R-002: CRDT eventual consistency for hot data | R-003: Persistence in KineSQL WAL.

**Evidence:** Execution of simplified Jepsen network chaos tests validating linearity for writes and final convergence for data.

---

## 7. Non-Functional Criteria

| Appearance | Metric | Target |
|---------|---------|------|
| CRDT Merge Overhead | Time to merge states of 10k node graphs | < 5ms |
| Raft Election Latency | Time to elect new leader | < 1.5s |
| Raft-Lite Log Size | Periodic compression and purging of committed logs | 50MB limit before snapshot |

---

## 8. Quality and Metrics

**Success:**
- 100% deterministic graph convergence in all network partition recovery scenarios.
- Zero occurrences of persistent split-brain after physical partition resolution.
- Unit test coverage greater than 80%.

**Failure (BLOCKING):**
- Loss of linearity in strong writes (e.g. two leaders accepting simultaneous writes in the same term).
- Corruption of the semantic graph due to CRDT merge failure.

---

## 9. Compatibility and Dependencies

| Crate | Version | Purpose |
|-------|--------|-----------|
| `parking_lot` | ^0.12 | Fast concurrent locks |
| `serde` | ^1.0 | Serialization of messages and states |

---

## 10. Traceability

| Type | ID | Description |
|------|----|-----------|
| Spec | FT-030-DISTRIBUTED-CONSENSUS | This specification |
| Design | `docs/kce_distributed_architecture.md` | Distributed Consensus Design |

---

## 11. Roadmap

### MVP (Phase 1)
Simple eventual consistency based on purely in-memory Last-Write-Wins. No support for automatic elections.

### Iteration 1 (Phase 2)
Full implementation of the Raft-Lite state machine with leader and quorum election. Saving Tenants and Shards configurations persisted in KineSQL.

### Iteration 2 (Phase 3)
Optimized delta-based synchronization of CRDTs with payload compression and full integration with AIS controlled hypermutations.
