# FT-027 — ACO-HNSW (Bio-Inspired Vector Indexing)

**Module:** Core Engine / ACO | **Version:** v6.0 | **Priority:** P0 — Critical  
**Artifact ID:** FT-027-ACO-HNSW | **Update:** 2026-05-30

---

## 1. Context and Objective

Modern graph-based vector search indexes, such as HNSW (Hierarchical Navigable Small World), use greedy search and static geometric connections. Although efficient, these methods do not adapt to real query usage patterns.

**ACO-HNSW** replaces the purely geometric search with a dynamic traversal based on **Ant Colony Optimization**. The multilayer graph stores not only the Euclidean distances of the edges, but also a dynamic **pheromone ($\tau$)** weight. Frequent, semantically similar queries create and reinforce "cognitive highways" that guide search ants to the most relevant neighbors, reducing the total number of nodes visited and stabilizing tail latency (P99) under high workload.

---

## 2. Acceptance Criteria (AC)

### General
- [ ] AC-001: Compilation without warnings in the Rust workspace ecosystem.
- [ ] AC-002: Thread-safe concurrent access for simultaneous search and writing of pheromones via `parking_lot`.
- [ ] AC-003: Complete documentation of the public API in code.

### Specifics
- [ ] AC-010: The search graph must contain multiple layers. Each edge must map vector distance and a pheromone level $\tau \ge 0.0$ (initialized with a default value $\tau_0$).
- [ ] AC-011: Index searches must support probabilistic traversal guided by the pheromone transition formula:
  $$P_{ij} = \frac{[\tau_{ij}]^\alpha \cdot [\eta_{ij}]^\beta}{\sum_{k \in \mathcal{N}(i)} [\tau_{ik}]^\alpha \cdot [\eta_{ik}]^\beta}$$
  where $\tau_{ij}$ represents the pheromone strength, $\eta_{ij}$ is the inverse similarity of the distance of the neighboring node $j$ with respect to the query vector, $\mathcal{N}(i)$ is the set of neighbors of the current node, and $\alpha, \beta$ are the configurable control weights.
- [ ] AC-012: **Path Reinforcement** Mechanism: at the end of a successful search, the path followed by the ants that found the best candidates (Top-K) receives an increase in pheromone:
  $$\Delta \tau_{ij} = Q \cdot \text{Score}_{\text{Similarity}}$$
- [ ] AC-013: **Temporal Evaporation** Mechanism: at each time interval or number of queries, the edge pheromone evaporates following the decay constant $\rho$ defined in feature FT-014:
  $$\tau_{ij} \leftarrow (1 - \rho) \cdot \tau_{ij}$$
- [ ] AC-014: Dynamic fallback support: If the network regulator (FT-022) indicates instability or memory exhaustion, the index disables the probabilistic ACO calculation and operates in classic HNSW greedy search mode.
- [ ] AC-015: P99 tail latency $\le 8\text{ ms}$ for datasets of up to 100k vectors (128d) at 1000 FPS.

---

## 3. Definition of Done (DoD)

- [ ] Multilayer graph data structure with vectors and pheromones implemented.
- [ ] Stochastic ant routing in the graph using thread-efficient pseudorandom number generators.
- [ ] Parallelized search threads with Rayon for ants to explore paths concurrently.
- [ ] Asynchronous evaporation system coupled to the general timer of the ACO engine.
- [ ] Unit tests with at least 85% code coverage in hot search paths.
- [ ] Comparative benchmarks showing reduced hops in the graph for repeated semantic searches compared to classic HNSW.

---

## 4. Usage Examples

### ACO-HNSW Index Search Payload (JSON)

**Input (POST /query/aco-hnsw):**
```json
{
  "query_vector": [0.05, 0.91, -0.12, 0.43],
  "top_k": 5,
  "aco_params": {
    "alpha": 1.2,
    "beta": 2.0,
    "num_ants": 8,
    "pheromone_deposit": 0.5
  }
}
```

**Output:**
```json
{
  "results": [
    { "id": 1024, "score": 0.985, "hops": 3 },
    { "id": 2048, "score": 0.912, "hops": 4 },
    { "id": 512, "score": 0.887, "hops": 5 }
  ],
  "metrics": {
    "total_hops_visited": 12,
    "avg_hops_per_ant": 4.1,
    "latency_ms": 3.42,
    "pheromone_highways_utilized": ["1024->2048"]
  }
}
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Entry | Expected Output |
|----|------|---------|----------------|
| UT-001 | Transition Formula | $\tau_{ij}=1.0$, $\eta_{ij}=0.8$, $\alpha=1.0$, $\beta=1.0$ | Uniform probability between neighbors with identical weights |
| UT-002 | Edge Reinforcement | Edge visited with score 0.9 | Pheromone $\tau$ increased according to the constant $Q$ |
| UT-003 | Edge Evaporation | $\tau=2.0$, $\rho=0.1$ | After evaporation, $\tau$ should be $1.8$ |
| UT-004 | Pheromone Clamping | Repeated reductions by evaporation | $\tau$ must not fall below the defined minimum limit |

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | Repeated Semantic Load | 100 repeated queries should use identical paths with fewer hops as the pheromone accumulates |
| FT-002 | Distribution Change (Drift) | When the subject of consultations changes abruptly, old pheromone pathways must evaporate and new connections must be reinforced |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | ACO-HNSW → Evaporation (FT-014) | Periodic evaporation cycles from the ACO module clean the edges of the HNSW |
| IT-002 | ACO-HNSW → Regulation (FT-022) | The regulator reduces the number of ants if the p95 latency exceeds 10ms |

---

## 6. CARE Format

**Context:** Very low latency approximate vector recovery for the KCE production cognitive pipeline, operating under dynamic real-world workloads.

**Assumptions:** RAM is sufficient to maintain the vectors and sparse matrix of search edge pheromones. Repeated search paths follow common power laws in real semantic search applications.

**Requirements:** R-001: ACO Probability-Based Routing | R-002: Integration with HNSW multi-layers | R-003: Dynamic reading and reinforcement feedback.

**Evidence:** Benchmarks located in `benches/aco_hnsw_bench.rs` demonstrating hit rates and jump counts.

---

## 7. Non-Functional Criteria

| Appearance | Metric | Target |
|---------|---------|------|
| Search Latency | p95 for 100k vector base | < 5ms |
| Writing Latency | Time to update pheromone | < 0.5ms (asynchronous) |
| Memory Overhead | Additional space for pheromone float per edge | < 12% compared to the pure HNSW graph |

---

## 8. Quality and Metrics

**Success:**
- Recall@10 greater than 0.90 in synthetic load tests.
- At least 20% reduction in the number of nodes visited in repeat queries versus classic HNSW search.
- Zero leaks or panics in concurrent pheromone writing.

**Failure (BLOCKING):**
- Recall@10 below 0.80.
- Deadlocks in pheromone writing.

---

## 9. Compatibility and Dependencies

| Crate | Version | Purpose |
|-------|--------|-----------|
| `rayon` | ^1.8 | Parallelism of scout ants |
| `rand` | ^0.8 | Stochastic probabilistic routing |
| `parking_lot` | ^0.12 | Fast locks without kernel contention |

---

## 10. Traceability

| Type | ID | Description |
|------|----|-----------|
| Spec | FT-027-ACO-HNSW | This specification |
| Code | `crates/kce-retrieval/src/aco_hnsw.rs` | Index implementation |
| Bench | `benches/aco_hnsw_bench.rs` | Performance comparative test |

---

## 11. Roadmap

### MVP (Phase 1)
Navigable single-layer graph implementation using deterministic single ant search.

### Iteration 1 (Phase 2)
Multilayer graph implementation (full HNSW) with stochastic search for multiple parallel ants with Rayon.

### Iteration 2 (Phase 3)
Accelerated hardware integration, auto-tuning of $\alpha$ and $\beta$ by global regulator and pheromone state persistence in KineSQL.
