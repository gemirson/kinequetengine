# FT-014 — Evaporation Mechanism

**Module:** ACO (Ant Colony Optimization) | **Version:** v6.0 | **Priority:** P0 — Critical  
**Artifact ID:** FT-014-EVAPORATION | **Update:** 2026-05-23

---

## 1. Context and Objective

The Evaporation Mechanism implements **temporal pheromone decay** in the ACO system. Without evaporation, old paths permanently dominate — creating “toxic memory” and context overfitting. Evaporation ensures that obsolete routes lose relevance, keeping the system adaptive and alive. Classic formula: `τ(t+1) = (1 - ρ) * τ(t)`, where `ρ` is the evaporation rate.

### Value
- Avoids "toxic memory" — obsolete context decays naturally
- Maintains adaptive system — new routes have a chance to emerge
- Prevents overfitting — stagnant knowledge is gradually removed

---

## 2. Acceptance Criteria (AC)

### General
- [ ] AC-001: Compiles without warnings in `--release`
- [ ] AC-002: Thread-safe for concurrent evaporation

### Specifics
- [ ] AC-010: Evaporation follows formula `τ(t+1) = (1 - ρ) * τ(t)`
- [ ] AC-011: Evaporation rate `ρ` configurable (default: 0.1, range: 0.0..1.0)
- [ ] AC-012: Evaporation executed periodically (configurable interval, default: 60s)
- [ ] AC-013: Respects `τ_min` — pheromone never drops below minimum (MMAS)
- [ ] AC-014: `evaporate_all()` processes all edges of the graph
- [ ] AC-015: `evaporate_edge(edge_id)` processes individual edge
- [ ] AC-016: Logged evaporation metrics (`edges_evaporated`, `total_decay`)
- [ ] AC-017: Evaporation does not block queries (read lock during evaporation)
- [ ] AC-018: Evaporation log for audit

---

## 3. Definition of Done (DoD)

- [ ] Implemented evaporation formula
- [ ] ρ configurable with range validation
- [ ] Functional periodic timer (tokio interval)
- [ ] τ_min respected in all scenarios
- [ ] Integration with Pheromone Engine (FT-013)
- [ ] Exposed evaporation metrics
- [ ] Unit tests ≥ 80% coverage
- [ ] No `unwrap()` in production

---

## 4. Usage Examples

### Periodic evaporation

**State before (graph edges):**
```json
{
  "edges": [
    { "edge_id": "e_42_17", "pheromone": 5.0 },
    { "edge_id": "e_17_88", "pheromone": 2.0 },
    { "edge_id": "e_88_5",  "pheromone": 0.3 }
  ],
  "rho": 0.1,
  "tau_min": 0.1
}
```

**After evaporation (ρ = 0.1):**
```json
{
  "edges": [
    { "edge_id": "e_42_17", "pheromone": 4.5,  "decay": -0.5 },
    { "edge_id": "e_17_88", "pheromone": 1.8,  "decay": -0.2 },
    { "edge_id": "e_88_5",  "pheromone": 0.27, "decay": -0.03 }
  ],
  "total_decay": -0.73,
  "edges_processed": 3,
  "edges_at_tau_min": 0
}
```

### Evaporation with τ_min reached
```json
{
  "edge_id": "e_88_5",
  "pheromone_before": 0.12,
  "pheromone_calculated": 0.108,
  "pheromone_after": 0.1,
  "clamped_to_tau_min": true
}
```

### Configuration (CSV for batch)
```csv
parameter,value,description
rho,0.1,taxa de evaporação
tau_min,0.1,feromônio mínimo
interval_seconds,60,intervalo entre ciclos
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Entry | Expected Output |
|----|------|---------|----------------|
| UT-001 | Basic evaporation | τ=5.0, ρ=0.1 | τ=4.5 |
| UT-002 | Evaporation with ρ=0 | τ=5.0, ρ=0.0 | τ=5.0 (no change) |
| UT-003 | Evaporation with ρ=1 | τ=5.0, ρ=1.0 | τ=τ_min (clamp) |
| UT-004 | τ_min respected | τ=0.12, ρ=0.5 | τ=τ_min=0.1 |
| UT-005 | evaporate_all | 100 edges | all processed |
| UT-006 | Multiple cycles | 10 evaporations | correct exponential decay |
| UT-007 | Concurrency | evap + deposit simultaneously | no corruption |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | negative ρ | `Err(InvalidEvaporationRate)` |
| UF-002 | ρ > 1.0 | `Err(InvalidEvaporationRate)` |
| UF-003 | τ_min > τ_max | `Err(InvalidBounds)` |
| UF-004 | Empty graph | `Ok` with edges_processed=0 |

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | 100 evaporation cycles | All pheromones converge to τ_min without use |
| FT-002 | Evaporation + simultaneous deposit | Dynamic balance achieved |
| FT-003 | Periodic timer 60s | Evaporation triggers correctly |
| FT-004 | Dominant route decays | New route emerges after evaporation |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Evaporation → Pheromone (FT-013) | Scores in the graph decay correctly |
| IT-002 | Evaporation → KineSQL | Evaporated scores persist |
| IT-003 | Evaporation → Metrics | `kce_evaporation_total` registered |
| IT-004 | Evaporation + Pipeline | Routes adapt after evaporation |

---

## 6. CARE Format

**Context:** Without evaporation, the ACO system suffers from "toxic memory" — old pathways permanently dominate, preventing adaptation. The mechanism ensures that pheromone decays exponentially over time, forcing the system to continually revalidate routes.

**Assumptions:**
- ρ default = 0.1 is suitable for most workloads
- Periodic (not continuous) evaporation is sufficient for MVP
- τ_min guarantees that no edge is completely "dead"
- Concurrency between evaporation and deposit is resolved by RwLock (brief write lock)
- Timer via `tokio::time::interval`

**Requirements:**
- R-001: Formula `τ(t+1) = (1-ρ)*τ(t)` implemented
- R-002: Configurable ρ (0.0..1.0)
- R-003: τ_min as floor (MMAS)
- R-004: Configurable periodic timer
- R-005: Evaporation metrics
- R-006: Does not block queries

**Evidence:**
- `tests/evaporation_tests.rs`
- `benches/evaporation_bench.rs`
- `reports/evaporation_dynamics.md`

---

## 7. Non-Functional Acceptance Criteria

| Appearance | Target |
|---------|------|
| evaporate_all latency (10k edges) | < 50ms |
| Lock time during evaporation | < 10ms |
| Overhead in query throughput | < 2% |
| Additional memory | < 1MB |

---

## 8. Quality Criteria and Metrics

**Success:** τ_min never violated | Correct exponential decay | New routes emerge | Coverage ≥ 80%  
**Failure (BLOCKING):** Pheromone below τ_min | Queries blocking > 100ms | Unresolved stagnation

---

## 9. Compatibility and Dependencies

**Internal deps:** Pheromone Engine (FT-013), Graph Engine (FT-002), KineSQL (FT-005)  
**Crates:** `tokio` ^1.35 (timer), `parking_lot` ^0.12  
**Rust:** ≥ 1.75

---

## 10. Traceability

| Type | ID |
|------|----|
| Spec | FT-014-EVAPORATION |
| Code | `src/aco/evaporation.rs` |
| Test | `tests/evaporation_tests.rs` |
| Dep | FT-013, FT-002, FT-005 |

---

## 11. Roadmap MVP

### MVP (Week 1-2)
`evaporate_edge()` with ACO formula | ρ configurable | τ_min clamp | 5+ unit tests

### Iteration 1 (Week 3-4)
`evaporate_all()` | Periodic timer | Pheromone Engine Integration | Metrics | Benchmark

### Iteration 2 (Week 5-6)
Concurrency validated | KineSQL Persistence | Adaptive ρ (auto-tune) | Load Testing | Docs

---

*Document generated on 2026-05-23 — KineContext Engine Product Specification*
