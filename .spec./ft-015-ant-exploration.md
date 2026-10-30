# FT-015 — Ant Query Exploration Engine

**Module:** ACO (Ant Colony Optimization) | **Version:** v6.0 | **Priority:** P1 — High  
**Artifact ID:** FT-015-ANT-EXPLORATION | **Update:** 2026-05-23

---

## 1. Context and Objective

The Ant Query Exploration Engine executes **multiple "ants" (parallel queries)** exploring different paths in the semantic graph simultaneously. Each ant follows a slightly different heuristic (varying α, β, or starting point), and the best result is selected. Inspired by the stochastic exploration of real ant colonies.

### Value
- Improves recall in complex environments with multiple valid paths
- Local avoidance optima via parallel exploration
- Alternative routes are discovered naturally

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: `explore(query, num_ants)` launches N parallel ants
- [ ] AC-011: Each ant follows different heuristics (variation of α, β, seed)
- [ ] AC-012: Selection of the best result by composite score (relevance + latency)
- [ ] AC-013: Parallelism via Rayon (does not block main thread)
- [ ] AC-014: Results from all ants available for analysis (not just the best)
- [ ] AC-015: `num_ants` configurable (default: 5, max: 20)
- [ ] AC-016: Timeout per ant (slow ant is cancelled)
- [ ] AC-017: Better pathways deposit pheromone (FT-013 integration)
- [ ] AC-018: Guaranteed minimum diversity (at least 2 ants with different routes)

---

## 3. Definition of Done (DoD)

- [ ] `explore()` functional with N parallel ants
- [ ] Heuristic variation per ant implemented
- [ ] Selection of the best functional result
- [ ] Timeout per active ant
- [ ] Pheromone deposit by the best ant
- [ ] Unit tests ≥ 80%
- [ ] Comparative recall benchmark (N ants vs 1 query)

---

## 4. Usage Examples

### Exploration with 5 ants

**Input:**
```json
{
  "query_vector": [0.12, 0.85, 0.33, 0.67],
  "num_ants": 5,
  "timeout_ms": 50,
  "diversity_min": 2
}
```

**Output:**
```json
{
  "best_result": {
    "ant_id": 2,
    "path": [42, 17, 88, 5],
    "score": 0.94,
    "latency_ms": 18.2,
    "heuristic": { "alpha": 1.2, "beta": 1.8 }
  },
  "all_ants": [
    { "ant_id": 0, "path": [42, 33, 5], "score": 0.82, "latency_ms": 12.1 },
    { "ant_id": 1, "path": [42, 91, 12, 5], "score": 0.76, "latency_ms": 22.4 },
    { "ant_id": 2, "path": [42, 17, 88, 5], "score": 0.94, "latency_ms": 18.2 },
    { "ant_id": 3, "path": [42, 17, 88, 5], "score": 0.91, "latency_ms": 19.0 },
    { "ant_id": 4, "path": [42, 7, 5], "score": 0.65, "latency_ms": 8.3, "status": "timeout" }
  ],
  "unique_paths": 4,
  "total_latency_ms": 22.4,
  "pheromone_deposited_on": [42, 17, 88, 5]
}
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Expected Output |
|----|------|----------------|
| UT-001 | 5 ants released | 5 results returned |
| UT-002 | Best result selected | highest score chosen |
| UT-003 | Minimal diversity | ≥ 2 different routes |
| UT-004 | Ant timeout | slow ant canceled |
| UT-005 | num_ants = 1 | works like simple query |
| UT-006 | Different heuristics | α, β vary between ants |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | num_ants = 0 | `Err(InvalidAntCount)` |
| UF-002 | num_ants > 20 | `Err(TooManyAnts)` |
| UF-003 | All ants timeout | `Err(AllAntsTimedOut)` |

### 5.2 Functional Tests

| ID | Scenario | Result |
|----|---------|-----------|
| FT-001 | 5 ants vs 1 query (recall) | Recall ≥ 10% better with ants |
| FT-002 | Graph with multiple paths | Ants explore different routes |
| FT-003 | Under load (100 explorations) | All complete without error |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Exploration → Pheromone | Best route receives deposit |
| IT-002 | Exploration → Pipeline | Pipeline uses best ant result |
| IT-003 | Exploration → Retrieval | Ants use Retrieval Engine |

---

## 6. CARE Format

**Context:** Single query can get stuck in optimal location. Multiple ants explore divergent paths simultaneously, maximizing recall in complex graphs.

**Assumptions:** Rayon available for parallelism; graph has multiple paths between nodes; overhead of N ants is acceptable (< 3x latency of 1 query); best ant deposits pheromone.

**Requirements:** R-001: N parallel ants | R-002: Heuristic varied per ant | R-003: Selection of the best | R-004: Timeout per ant | R-005: Minimum diversity | R-006: Deposit for the best.

**Evidence:** `tests/ant_exploration_tests.rs` | `reports/recall_improvement.md`

---

## 7–11. (Summary)

**Non-functional:** Total latency < 3x simple query | Throughput ≥ 200 explorations/s | Extra memory < 10MB for 20 ants  
**Quality:** Recall ≥ 10% better than simple query | Diversity ≥ 2 routes | Coverage ≥ 80%  
**Failure (BLOCKING):** Recall worse than simple query | 0 diversity | Deadlock in parallelism  
**Deps:** Rayon ^1.8, Pheromone (FT-013), Retrieval (FT-001), Graph (FT-002)  
**Traceability:** FT-015-ANT-EXPLORATION | `src/aco/exploration.rs`

**Roadmap:** MVP: 3 sequential ants + selection | Iter1: Rayon parallelism + timeout + diversity | Iter2: Adaptive num_ants + deposit + benchmark recall

---

*Document generated on 2026-05-23 — KineContext Engine Product Specification*
