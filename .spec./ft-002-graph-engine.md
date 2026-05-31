# FT-002 — Graph Engine (Semantic Graph)

**Module:** Core Engine | **Version:** v5.9 | **Priority:** P0 — Critical  
**Artifact ID:** FT-002-GRAPH-ENGINE | **Updated:** 2026-05-23

---

## 1. Context and Objective

The Graph Engine manages the KCE semantic graph, responsible for the **contextual expansion** of candidates returned by the Retrieval Engine. It connects concepts via weighted edges and expands the search context beyond direct similarity, allowing ECMA and MCE to operate with a relational view of knowledge.

---

## 2. Acceptance Criteria (AC)

### General
- [ ] AC-001: Compiles without warnings in `--release`
- [ ] AC-002: Thread-safe for concurrent reading

### Specific
- [ ] AC-010: `add_edge(a, b, weight)` creates a bidirectional edge
- [ ] AC-011: `neighbors(node_id)` returns direct neighbors with weights
- [ ] AC-012: `expand(nodes, depth)` returns expanded subgraph up to depth `depth`
- [ ] AC-013: Expansion returns ≤ 2x input nodes
- [ ] AC-014: No node duplication in the expansion result
- [ ] AC-015: Expansion latency < 10ms
- [ ] AC-016: Deterministic expansion (same input → same output)
- [ ] AC-017: Functional edge removal (`remove_edge`)
- [ ] AC-018: Graph supports ≥ 100k nodes without degradation

---

## 3. Definition of Done (DoD)

- [ ] `add_edge` functional and tested
- [ ] `neighbors` consistent with addition/removal
- [ ] Deterministic expansion implemented
- [ ] No node duplication
- [ ] Unit tests ≥ 80% coverage
- [ ] Functional tests passing
- [ ] Public API documentation

---

## 4. Usage Examples

### Contextual expansion
**Input:**
```json
{
  "seed_nodes": [42, 17, 88],
  "expansion_depth": 2,
  "max_nodes": 20
}
```

**Output:**
```json
{
  "expanded_nodes": [42, 17, 88, 5, 12, 33, 91, 7],
  "edges": [
    { "from": 42, "to": 5, "weight": 0.85 },
    { "from": 17, "to": 12, "weight": 0.72 },
    { "from": 88, "to": 33, "weight": 0.68 }
  ],
  "depth_reached": 2,
  "total_nodes": 8
}
```

### Error — node does not exist
```json
{ "error": "NODE_NOT_FOUND", "message": "Node 9999 does not exist in graph", "code": 404 }
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Input | Expected Output |
|----|------|---------|----------------|
| UT-001 | bidirectional add_edge | `add_edge(1, 2, 0.5)` | `neighbors(1)` contains 2 and vice versa |
| UT-002 | neighbors — isolated node | `neighbors(99)` | `[]` |
| UT-003 | expand depth=1 | seed=[1], depth=1 | node 1 + direct neighbors |
| UT-004 | expand without duplicates | cycle 1→2→3→1 | each node appears 1x |
| UT-005 | remove_edge | `remove_edge(1,2)` | `neighbors(1)` does not contain 2 |
| UT-006 | expand max_nodes | 100 neighbors, max=5 | exactly 5 nodes |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | Node does not exist | `Err(NodeNotFound)` |
| UF-002 | Negative depth | `Err(InvalidDepth)` |
| UF-003 | Duplicate edge | Ignores silently (idempotent) |

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | Create 1k node graph + expand | Correct subgraph in < 10ms |
| FT-002 | Concurrent expansion | 50 simultaneous expansions without error |
| FT-003 | Graph after persistence | KineSQL reload maintains structure |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Retrieval → Graph → ECMA | Expanded candidates feed ECMA |
| IT-002 | Graph → KineSQL | Graph persists and retrieves correctly |

---

## 6. CARE Format

**Context:** Semantic graph for contextual expansion of Retrieval Engine candidates; allows relations beyond direct similarity.

**Assumptions:** Graph is sparse (avg degree < 10); fits in memory; edges are bidirectional with weight; read concurrency is more frequent than write.

**Requirements:** R-001: add/remove_edge | R-002: neighbors O(1) | R-003: expand with depth | R-004: ≤ 2x input nodes | R-005: latency < 10ms | R-006: determinism.

**Evidence:** `tests/graph_tests.rs` | `benches/graph_bench.rs`

---

## 7. Non-Functional Criteria

| Aspect | Target |
|---------|------|
| Expansion latency | < 10ms |
| Memory (100k nodes) | < 100MB |
| Concurrency | 50 simultaneous reads without degradation |

---

## 8. Quality and Metrics

**Success:** No duplicates in expansion | 100% determinism | Coverage ≥ 80%  
**Failure (BLOCKING):** Duplicates in result | Latency > 50ms | Crash in concurrency

---

## 9. Compatibility and Dependencies

**Internal deps:** Retrieval Engine (input), ECMA (output), KineSQL (persistence)  
**Rust:** ≥ 1.75 | **Crates:** `parking_lot` ^0.12

---

## 10. Traceability

| Type | ID |
|------|----|
| Spec | FT-002-GRAPH-ENGINE |
| Code | `src/graph/mod.rs` |
| Test | `tests/graph_integration.rs` |

---

## 11. MVP Roadmap

### MVP (Week 1-2)
`add_edge` + `neighbors` | 3+ unit tests | Adjacency list structure

### Iteration 1 (Week 3-4)
`expand` with depth | Deduplication | `remove_edge` | Determinism | Benchmark

### Iteration 2 (Week 5-6)
KineSQL integration (persistence) | Retrieval→Graph→ECMA integration | Load tests | Docs
