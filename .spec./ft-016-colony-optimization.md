# FT-016 — Context Colony Optimization

**Module:** ACO (Ant Colony Optimization) | **Version:** v6.0 | **Priority:** P1 — High  
**Artifact ID:** FT-016-COLONY-OPTIMIZATION | **Update:** 2026-05-23

---

## 1. Context and Objective

Context Colony Optimization combines **multiple searches and explorations** to converge on a global — not just local — optimal solution. Aggregates results from multiple ACO cycles (deposit + evaporate + explore) to produce a final ranking that considers the entire accumulated pheromone history. Replaces simple ranking with **emergent global optimization system**.

### Value
- Replaces simple ranking with emergent global optimization
- Combines information from multiple cycles for convergence
- Context decisions improve over time

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: `optimize(query, cycles)` executes N complete ACO cycles
- [ ] AC-011: Each cycle: explore (ants) → deposit (pheromone) → evaporate → rank
- [ ] AC-012: Measured convergence: variation between cycles < threshold → stable
- [ ] AC-013: Final result is global ranking considering complete ACO history
- [ ] AC-014: `convergence_threshold` configurable (default: 0.05)
- [ ] AC-015: Early stop when convergence reached before max_cycles
- [ ] AC-016: Convergence metrics exposed (`cycles_to_converge`, `final_score_variance`)
- [ ] AC-017: Fallback to static ranking if ACO does not converge on max_cycles

---

## 3. Definition of Done (DoD)

- [ ] Full ACO cycle (explore → deposit → evaporate → rank) implemented
- [ ] Measured convergence and functional early stop
- [ ] Fallback for static ranking
- [ ] Convergence metrics exposed
- [ ] Convergence tests passing
- [ ] Coverage ≥ 80%

---

## 4. Usage Examples

### Optimization with convergence

**Input:**
```json
{
  "query_vector": [0.12, 0.85, 0.33, 0.67],
  "max_cycles": 20,
  "ants_per_cycle": 5,
  "convergence_threshold": 0.05
}
```

**Output:**
```json
{
  "optimal_result": {
    "path": [42, 17, 88, 5],
    "score": 0.96,
    "confidence": 0.93
  },
  "convergence": {
    "cycles_executed": 12,
    "converged": true,
    "final_variance": 0.03,
    "score_progression": [0.72, 0.78, 0.85, 0.89, 0.92, 0.94, 0.95, 0.95, 0.96, 0.96, 0.96, 0.96]
  },
  "total_latency_ms": 145.2
}
```

### Non-convergence → fallback
```json
{
  "optimal_result": { "path": [42, 33, 5], "score": 0.78 },
  "convergence": {
    "cycles_executed": 20,
    "converged": false,
    "final_variance": 0.12,
    "fallback": "static_ranking"
  }
}
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Expected Output |
|----|------|----------------|
| UT-001 | Convergence in < 20 cycles | early stop activated |
| UT-002 | Non-convergence | max_cycles reached, fallback |
| UT-003 | Score improves between cycles | increasing monotonic progression |
| UT-004 | Variance < threshold | `converged: true` |
| UT-005 | 1 cycle | works like simple explore |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | max_cycles = 0 | `Err(InvalidCycles)` |
| UF-002 | Empty graph | `Err(EmptyGraph)` |

### 5.2 Functional Tests

| ID | Scenario | Result |
|----|---------|-----------|
| FT-001 | Optimization on 10k dataset | Convergence in < 20 cycles |
| FT-002 | ACO vs static ranking comparison | ACO ≥ 15% better in score |
| FT-003 | Under concurrent load | 20 simultaneous optimizations OK |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Colony → Exploration → Pheromone → Evaporation | Complete functional cycle |
| IT-002 | Colony → Pipeline | Pipeline uses optimized result |
| IT-003 | Colony → Metrics | Registered convergence |

---

## 6. CARE Format

**Context:** Local search (simple top-k) does not guarantee global optimality. Colony Optimization runs multiple full ACO cycles to converge on global solution, combining pheromone history from all iterations.

**Assumptions:** 20 maximum cycles is sufficient for convergence in most graphs; variance < 0.05 indicates convergence; static fallback is acceptable if ACO does not converge; total latency < 500ms is acceptable for global optimization.

**Requirements:** R-001: Complete ACO cycle | R-002: Measured convergence | R-003: Early stop | R-004: Static fallback | R-005: Metrics | R-006: Overall result.

**Evidence:** `tests/colony_optimization_tests.rs` | `reports/aco_vs_static.md`

---

## 7–11. (Summary)

**Non-functional:** Total latency < 500ms for 20 cycles | Convergence in < 20 cycles (90% of cases) | Improvement ≥ 15% vs static  
**Quality:** Convergence 90%+ | Score ≥ 15% better | Coverage ≥ 80%  
**Failure (BLOCKING):** Score worse than static | Convergence < 50% | Latency > 2s  
**Deps:** Exploration (FT-015), Pheromone (FT-013), Evaporation (FT-014)  
**Traceability:** FT-016-COLONY-OPTIMIZATION | `src/aco/colony.rs`

**Roadmap:** MVP: 1 ACO cycle | Iter1: multi-cycle + convergence + early stop | Iter2: fallback + metrics + benchmark vs static + docs

---

*Document generated on 2026-05-23 — KineContext Engine Product Specification*
