# FT-028 — Distributed Sharding (ACTA)

**Module:** Infrastructure / Core Engine | **Version:** v6.0 | **Priority:** P0 — Critical  
**Artifact ID:** FT-028-DISTRIBUTED-SHARDING | **Update:** 2026-05-30

---

## 1. Context and Objective

As KCE scales to multiple nodes and servers, maintaining the entire semantic graph and vector space on a single node becomes infeasible due to CPU and memory limitations. This specification defines **Distributed Sharding**, which slices data based on partition keys and employs a dynamic mechanism inspired by the division of labor in ant colonies: the **Ant-Colony Task Allocation (ACTA)**. The objective is to distribute the search and indexing load dynamically according to actual usage, mitigating hot shards problems.

---

## 2. Acceptance Criteria (AC)

### General
- [ ] AC-001: Clean compilation without warnings in the Rust compiler.
- [ ] AC-002: Strict isolation between shards from different tenants.
- [ ] AC-003: Complete thread-safety for concurrent reads and asynchronous redistributions.

### Specifics
- [ ] AC-010: Definition of the partition key composed of `tenant_id` (primary isolation) and `context_hash` (secondary distribution of the graph).
- [ ] AC-011: Dynamic query routing: If a node $A$ hosting the primary Shard is under high utilization ($>85\%$ CPU/RAM), it must delegate the task (query) transparently to secondary replicas on the less loaded node $B$.
- [ ] AC-012: The homeostasis engine distributes shards in such a way as to "attract" semantically correlated data to geographically or physically close nodes, minimizing inter-node network hops.
- [ ] AC-013: Asynchronous shard migration: redistribution of data without blocking read queries on the hot path.
- [ ] AC-014: Support for up to 256 logical partitions dynamically mapped to available physical nodes.

---

## 3. Definition of Done (DoD)

- [ ] Implementation of traits and data partitioning structures (`ShardRouter` and `TaskAllocator`).
- [ ] ACTA algorithm simulated and validated in stress scenarios with background redistribution.
- [ ] Migration testing without packet loss or service interruption.
- [ ] Unit tests with at least 80% coverage.
- [ ] Coexistence of local shards (KineSQL) with network routing.

---

## 4. Usage Examples

### Shard Routing Structure (internal JSON)

**Shared router configured on the node:**
```json
{
  "node_id": "8a72-f19b-449e-ba02",
  "assigned_shards": [
    { "shard_id": 12, "tenant_id": 1, "hash_range": [0, 1000] },
    { "shard_id": 13, "tenant_id": 1, "hash_range": [1001, 2000] }
  ],
  "node_load": {
    "cpu_utilization": 0.42,
    "memory_free_bytes": 8589934592,
    "active_delegations": 0
  }
}
```

### Task Delegation Routing Response (Query)

**JSON returned when delegating search to neighboring node:**
```json
{
  "query_id": "9a1f-82bc",
  "status": "DELEGATED",
  "delegated_to_node": "3c01-d822-411a-ba73",
  "reason": "Source node load above 85%; routing to low-latency replica",
  "latency_overhead_ms": 1.2
}
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Entry | Expected Output |
|----|------|---------|----------------|
| UT-001 | Shard Determination | tenant=1, vector_hash=420 | consistent shard_id (ex: 12) |
| UT-002 | ACTA Delegation Trigger | Node A load = 90% | `TaskAllocator::should_delegate` returns `true` |
| UT-003 | Shard Migration | Redistribution signal | Data moved to the destination node; validated checksum |

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | Hot Shard Simulation | Artificial load on 1 node results in automatic redirection of 40% of queries to secondary nodes in 30 seconds |
| FT-002 | Primary Node Drop | Immediate detection and promotion of a secondary node to take over shard reads |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Sharding → KineSQL (FT-005) | Migrated data is serialized and downloaded to the disk database at the destination |
| IT-002 | Sharding → Homeostasis (FT-022) | Homeostasis adjusts load thresholds that activate task delegation |

---

## 6. CARE Format

**Context:** Horizontal scaling and mitigation of hardware bottlenecks in KCE clusters operating in real-world scenarios with multiple simultaneous clients.

**Assumptions:** Each physical node knows the topology of neighboring nodes and the declared maximum hardware capabilities. The inter-node network has RTT latency of less than 2ms.

**Requirements:** R-001: Tenant and data hash-based routing | R-002: Load-Based Task Delegation (ACTA) | R-003: Asynchronous fault-tolerant migration.

**Evidence:** Execution of distributed load scripts simulating dynamic redistribution of shards and validation of p95 latencies.

---

## 7. Non-Functional Criteria

| Appearance | Metric | Target |
|---------|---------|------|
| Routing Overhead | Additional network latency for redirection decision | < 0.2ms |
| Convergence Time | Time for a new node to take over a delegated shard | < 1s |
| Data Loss During Migration | Integrity of ongoing queries | 0 errors (Transitional) |

---

## 8. Quality and Metrics

**Success:**
- Standard deviation of CPU load between cluster nodes less than 15% after ACTA stabilization.
- Zero data loss or inconsistency during migration.
- Unit test coverage greater than 80%.

**Failure (BLOCKING):**
- Partition inconsistency (two identical keys on separate active shards with no replication configured).
- Tenant data leak during redeployment.

---

## 9. Compatibility and Dependencies

| Crate | Version | Purpose |
|-------|--------|-----------|
| `uuid` | ^1.6 | Unique node and transaction IDs |
| `parking_lot` | ^0.12 | Concurrent locks for route tables |

---

## 10. Traceability

| Type | ID | Description |
|------|----|-----------|
| Spec | FT-028-DISTRIBUTED-SHARDING | This specification |
| Design | `docs/kce_distributed_architecture.md` | Swarm distributed architecture |

---

## 11. Roadmap

### MVP (Phase 1)
Static in-memory partitioning with tenant_id-based routing. No asynchronous migration.

### Iteration 1 (Phase 2)
Implementation of ACTA (dynamic delegation based on CPU telemetry). Support active replicas and transparent search redirection.

### Iteration 2 (Phase 3)
Asynchronous dynamic shard migration with remote fsync and full integration with global homeostasis.
