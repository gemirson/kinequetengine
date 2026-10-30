# FT-013 — Pheromone Routing Engine (ACO Core)

**Module:** ACO (Ant Colony Optimization) | **Version:** v6.0 | **Priority:** P0 — Critical  
**Artifact ID:** FT-013-PHEROMONE-ROUTING | **Update:** 2026-05-23

---

## 1. Context and Objective

The Pheromone Routing Engine is the core of KCE's ACO system. Each interaction (query, retrieval, execution) deposits a `pheromone_score` on the path taken in the semantic graph. More used and more efficient paths accumulate pheromone and become preferred for future searches. Replaces static heuristics with **emergent learning** — the system learns which context routes are most valuable without hard-coded rules.

### Value
- Replaces static heuristics with emergent learning
- Context routes self-optimize with use
- Behavior improves organically without retraining

---

## 2. Acceptance Criteria (AC)

### General
- [ ] AC-001: Compiles without warnings in `--release`
- [ ] AC-002: Thread-safe via `Arc<RwLock<...>>`
- [ ] AC-003: Public interface with doc comments

### Specifics
- [ ] AC-010: Each edge of the graph has a `pheromone_score: f64` field (initialized in `1.0`)
- [ ] AC-011: `deposit(edge, score)` increases pheromone proportionally to the success of the query
- [ ] AC-012: `route(source, target)` selects path with probability proportional to pheromone
- [ ] AC-013: Probabilistic selection follows ACO formula: `P(edge) = τ^α * η^β / Σ(τ^α * η^β)` where τ=pheromone, η=heuristic
- [ ] AC-014: Parameters `α` (pheromone weight) and `β` (heuristic weight) configurable
- [ ] AC-015: Maximum (`τ_max`) and minimum (`τ_min`) pheromone to avoid stagnation (MMAS — Max-Min Ant System)
- [ ] AC-016: Deposit is proportional to the quality of the result (`1/latency` or `recall_score`)
- [ ] AC-017: Deposit history traceable for audit
- [ ] AC-018: Integration with Graph Engine (FT-002) for routes

---

## 3. Definition of Done (DoD)

- [ ] `deposit(edge, score)` implemented and tested
- [ ] `route(source, target)` with ACO probabilistic selection
- [ ] ACO formula implemented with configurable α, β
- [ ] Active τ_max / τ_min limits (MMAS)
- [ ] Integration with functional Graph Engine
- [ ] Unit tests ≥ 80% coverage
- [ ] Registered Convergence Benchmark
- [ ] No `unwrap()` in production code

---

## 4. Usage Examples

### Pheromone deposit after successful query

**Entry (deposit event):**
```json
{
  "path": [
    { "from": 42, "to": 17, "edge_id": "e_42_17" },
    { "from": 17, "to": 88, "edge_id": "e_17_88" },
    { "from": 88, "to": 5, "edge_id": "e_88_5" }
  ],
  "quality_score": 0.92,
  "latency_ms": 12.4,
  "deposit_strategy": "quality_proportional"
}
```

**Status after deposit:**
```json
{
  "edges_updated": [
    { "edge_id": "e_42_17", "pheromone_before": 1.0, "pheromone_after": 1.92, "delta": 0.92 },
    { "edge_id": "e_17_88", "pheromone_before": 1.5, "pheromone_after": 2.42, "delta": 0.92 },
    { "edge_id": "e_88_5", "pheromone_before": 0.8, "pheromone_after": 1.72, "delta": 0.92 }
  ],
  "total_deposit": 2.76
}
```

### Pheromone route selection

**Prohibited:**
```json
{
  "source_node": 42,
  "target_node": 5,
  "alpha": 1.0,
  "beta": 2.0,
  "num_candidates": 3
}
```

**Output (candidate routes with probability):**
```json
{
  "routes": [
    { "path": [42, 17, 88, 5], "probability": 0.65, "total_pheromone": 5.06 },
    { "path": [42, 33, 5], "probability": 0.25, "total_pheromone": 3.20 },
    { "path": [42, 91, 12, 5], "probability": 0.10, "total_pheromone": 1.80 }
  ],
  "selected_route": [42, 17, 88, 5],
  "selection_method": "roulette_wheel"
}
```

### Error — nodes disconnected
```json
{ "error": "NO_ROUTE", "message": "No path between node 42 and node 999", "code": 404 }
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Prohibited | Expected Output |
|----|------|---------|----------------|
| UT-001 | Deposit increases pheromone | deposit(edge, 0.5) | pheromone += 0.5 |
| UT-002 | Initial pheromone = 1.0 | new edge | `pheromone == 1.0` |
| UT-003 | τ_max respected | excessive deposits | `pheromone <= τ_max` |
| UT-004 | τ_min respected | after evaporation | `pheromone >= τ_min` |
| UT-005 | Probabilistic selection | 2 routes, τ=[5.0, 1.0] | route 1 selected ~83% (α=1, β=0) |
| UT-006 | α=0 ignores pheromone | any τ | uniform selection |
| UT-007 | Seed deterministic route | fixed seed | same route always |
| UT-008 | Deposit quality_proportional | score=0.9 | proportional delta |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | We disconnected | `Err(NoRoute)` |
| UF-002 | Negative score | `Err(InvalidScore)` |
| UF-003 | negative α or β | `Err(InvalidParams)` |
| UF-004 | Non-existent Edge | `Err(EdgeNotFound)` |

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | 100 queries on the same route | Pheromone accumulates, route becomes dominant |
| FT-002 | Alternative routes under load | Best route converges for >60% selection |
| FT-003 | Pipeline integration | Pipeline uses ACO route instead of fixed |
| FT-004 | Deposit concurrency | 50 simultaneous deposits without corruption |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Pheromone → Graph Engine | Deposits reflect in the graph |
| IT-002 | Pipeline → Pheromone → Retrieval | Retrieval uses pheromone-weighted routes |
| IT-003 | Pheromone → KineSQL | Scores persist after restart |
| IT-004 | Pheromone → Evaporation (FT-014) | Evaporation correctly reduces scores |

---

## 6. CARE Format

**Context:** KCE uses static heuristics for context routing. Pheromone Routing replaces this with ant colony-inspired emergent learning (ACO). Each interaction deposits digital pheromone on the graph's paths, causing more efficient routes to naturally reinforce themselves.

**Assumptions:**
- The semantic graph (FT-002) already exists and supports weights on edges
- Pheromone is an f64 float with configurable limits
- Probabilistic selection uses roulette wheel selection
- Deposit concurrency is managed via RwLock
- α and β default are 1.0 and 2.0 respectively (classic ACO)

**Requirements:**
- R-001: Pheromone deposit proportional to quality
- R-002: ACO probabilistic selection (P = τ^α * η^β / Σ)
- R-003: MMAS — limits τ_max, τ_min
- R-004: Configurable α, β parameters
- R-005: Integration with Graph Engine
- R-006: Persistence via KineSQL
- R-007: Thread-safe for concurrent deposits

**Evidence:**
- `benches/pheromone_convergence.rs` — convergence benchmark
- `tests/pheromone_routing_tests.rs` — unit tests
- `reports/aco_analysis.md` — convergence analysis vs static heuristics

---

## 7. Non-Functional Acceptance Criteria

### Performance
| Metric | Target | Method |
|---------|------|--------|
| Deposit latency | < 0.5ms | Benchmark |
| Route selection latency | < 5ms | Benchmark |
| Convergence | < 50 iterations for optimal route | Synthetic dataset |
| Throughput deposits | ≥ 5000/s | `wrk` |

### Security
- Validated scores (non-negative, non-NaN)
- τ_max/τ_min limits prevent manipulation
- Deposit audit trail

### Accessibility
- N/A (backend module)

---

## 8. Quality Criteria and Metrics

### Success Metrics
| Metric | Target Value |
|---------|------------|
| Recall improvement vs static | ≥ 10% |
| Convergence to optimal route | < 50 iterations |
| Exploration rate maintained | ≥ 15% (does not get stuck) |
| Test coverage | ≥ 80% |

### Failure Criteria
- Convergence > 200 iterations → **BLOCKING**
- Pheromone outside [τ_min, τ_max] → **BLOCKING**
- Stagnation (0% exploration) → **BLOCKING**
- Corruption in competing deposit → **BLOCKING**

---

## 9. Compatibility and Dependencies

### Direct Dependencies
| Crate | Version | Purpose |
|-------|--------|-----------|
| `rand` | ^0.8 | Probabilistic selection |
| `parking_lot` | ^0.12 | Optimized RwLock |

### Internal Dependencies
- **Graph Engine (FT-002):** edge structure for deposit
- **Evaporation (FT-014):** pheromone decay
- **KineSQL (FT-005):** score persistence
- **Pipeline (FT-011):** integration into the flow

### Compatibility
| Item | Requirement |
|------|-----------|
| Rust | ≥ 1.75 stable |
| YOU | Linux (prod), macOS (dev) |

---

## 10. Traceability Criteria

### Change History
| Date | Version | Description | Author |
|------|--------|-----------|-------|
| 2026-05-23 | v1.0 | Creating the initial specification | Product Specialist |

### Related Artifact IDs
| Type | ID | Description |
|------|----|-----------|
| Spec | FT-013-PHEROMONE-ROUTING | This specification |
| Code | `src/aco/pheromone.rs` | Main implementation |
| Code | `src/aco/routing.rs` | Route selection |
| Bench | `benches/pheromone_convergence.rs` | Benchmark convergence |
| Test | `tests/pheromone_routing_tests.rs` | Unit tests |
| Dep | FT-002, FT-005, FT-014 | Dependent features |

---

## 11. MVP Delivery and Acceptance Criteria — Roadmap

### MVP (Week 1-2)
| Item | Acceptance Criteria |
|------|--------------------|
| `deposit(edge, score)` | Increases pheromone correctly |
| `get_pheromone(edge)` | Returns current score |
| Initial pheromone = 1.0 | Tested |
| 5+ unit tests | Passing |

**Output:** Functional pheromone deposit on graph edges.

---

### Iteration 1 (Week 3-4)
| Item | Acceptance Criteria |
|------|--------------------|
| `route(source, target)` | ACO probabilistic selection |
| ACO formula (τ^α * η^β) | Implemented |
| MMAS (τ_max, τ_min) | Active Limits |
| Graph Engine Integration | Deposits reflect in the graph |
| Benchmark convergence | Registered (< 50 iter) |

**Output:** Functional routing with ACO selection integrated into the graph.

---

### Iteration 2 (Week 5-6)
| Item | Acceptance Criteria |
|------|--------------------|
| KineSQL Persistence | Scores survive restart |
| Pipeline Integration | Pipeline uses ACO routes |
| Evaporation Integration (FT-014) | Functional decay |
| Validated concurrency | 50 simultaneous deposits OK |
| Audit trail | Traceable deposit history |
| Load Tests | 5000 deposits/s sustained |
| Complete documentation | API docs + examples |

**Output:** Pheromone Routing production-ready integrated into the KCE pipeline.

---

*Document generated on 2026-05-23 — KineContext Engine Product Specification*
