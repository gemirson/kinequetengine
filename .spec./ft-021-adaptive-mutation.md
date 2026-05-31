# FT-021 — Adaptive Mutation Engine

**Module:** AIS (Artificial Immune System) | **Version:** v6.0 | **Priority:** P1 — High  
**Artifact ID:** FT-021-ADAPTIVE-MUTATION | **Update:** 2026-05-23

---

## 1. Context and Objective

The Adaptive Mutation Engine **mutates context representations** (embeddings) in a controlled way to explore new solutions and avoid stagnation. Applies mild perturbations to the search vectors, testing new combinations that may reveal relevant context not discovered by direct search. Inspired by somatic antibody hypermutation.

### Value
- Emerging innovation — discovers context off the beaten path
- Prevents stagnation when ACO routes converge too much
- Controlled exploration without compromising stability

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: `mutate(vector, rate)` returns disturbed vector within controlled range
- [ ] AC-011: Mutation rate (`rate`) configurable (default: 0.05, range: 0.0..0.3)
- [ ] AC-012: Mutation preserves L2 norm of the vector (re-normalizes after perturbation)
- [ ] AC-013: Gaussian perturbation with σ proportional to rate
- [ ] AC-014: `explore_mutations(vector, n)` generates N variants and tests each one
- [ ] AC-015: Selection of the best mutant by retrieval score
- [ ] AC-016: Mutant superior to the original → replaces in the pipeline (elitism)
- [ ] AC-017: Mutation is reversible — original preserved if no mutant improves
- [ ] AC-018: Metrics: `kce_mutations_total`, `kce_mutations_successful`, `kce_mutation_improvement_avg`

---

## 3. Definition of Done (DoD)

- [ ] `mutate()` with Gaussian perturbation and re-normalization
- [ ] `explore_mutations()` with selection of the best
- [ ] Elitism (original preserved if better)
- [ ] Configurable rate with validation
- [ ] Mutation metrics
- [ ] Coverage ≥ 80%

---

## 4. Usage Examples

### Search vector mutation

**Input:**
```json
{
  "original_vector": [0.12, 0.85, 0.33, 0.67],
  "mutation_rate": 0.05,
  "num_mutations": 5
}
```

**Output:**
```json
{
  "original_score": 0.82,
  "mutations": [
    { "id": 0, "vector": [0.14, 0.83, 0.35, 0.65], "score": 0.85, "improvement": 0.03 },
    { "id": 1, "vector": [0.10, 0.87, 0.31, 0.69], "score": 0.79, "improvement": -0.03 },
    { "id": 2, "vector": [0.13, 0.84, 0.36, 0.64], "score": 0.88, "improvement": 0.06 },
    { "id": 3, "vector": [0.11, 0.86, 0.32, 0.68], "score": 0.81, "improvement": -0.01 },
    { "id": 4, "vector": [0.15, 0.82, 0.34, 0.66], "score": 0.80, "improvement": -0.02 }
  ],
  "best_mutation": { "id": 2, "score": 0.88, "improvement": 0.06 },
  "selected": "mutation_2",
  "elitism": true
}
```

### No improvement → preserves original
```json
{
  "original_score": 0.95,
  "best_mutation_score": 0.91,
  "selected": "original",
  "elitism": true,
  "reason": "No mutation improved on original"
}
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Expected Output |
|----|------|----------------|
| UT-001 | Mutation preserves dimension | `mutated.len() == original.len()` |
| UT-002 | L2 norm preserved | `‖mutated‖₂ ≈ ‖original‖₂` (±1%) |
| UT-003 | Rate=0.0 → no change | `mutated == original` |
| UT-004 | Rate=0.3 → visible disturbance | distance > 0 |
| UT-005 | Elitism — worst mutant | original kept |
| UT-006 | Elitism — better mutant | selected mutant |
| UT-007 | Reproducibility with seed | fixed seed → same mutation |
| UT-008 | N mutations generated | exactly N variants |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | Rate > 0.3 | `Err(MutationRateTooHigh)` |
| UF-002 | Negative rate | `Err(InvalidRate)` |
| UF-003 | Empty vector | `Err(EmptyVector)` |
| UF-004 | N = 0 | `Err(InvalidMutationCount)` |

### 5.2 Functional Tests

| ID | Scenario | Result |
|----|---------|-----------|
| FT-001 | 100 mutations: ≥ 20% improve original | Success rate ≥ 20% |
| FT-002 | Mutation under ACO convergence | Discovers new route ≥ 10% of cases |
| FT-003 | Pipeline with active mutation | Average score improves ≥ 3% |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Mutation → Retrieval | Mutants tested in the Retrieval Engine |
| IT-002 | Mutation → ACO | Mutations fuel ACO exploitation |
| IT-003 | Mutation → Metrics | `kce_mutations_successful` registered |

---

## 6. CARE Format

**Context:** When ACO converges strongly, the system may be trapped in local optima. Adaptive mutation explores new solutions by perturbing embeddings in a controlled way, inspired by somatic hypermutation of antibodies.

**Assumptions:** Gaussian perturbation is appropriate; 5% rate is conservative to begin with; L2 re-normalization maintains compatibility with cosine similarity; elitism guarantees stability; seed for debug reproducibility.

**Requirements:** R-001: Gaussian mutate() | R-002: L2 Re-normalization | R-003: explore_mutations() with selection | R-004: Elitism | R-005: Configurable rate | R-006: Metrics | R-007: Reproducibility with seed.

**Evidence:** `tests/mutation_tests.rs` | `reports/mutation_impact.md`

---

## 7–11. (Summary)

**Non-functional:** Mutation latency < 1ms | Overhead pipeline < 5ms with 5 mutants | Memory < 1MB extra  
**Quality:** Improvement rate ≥ 20% | Average score ≥ 3% better | L2 norm preserved ±1% | Coverage ≥ 80%  
**Failure (BLOCKING):** L2 standard deviates > 5% | Improvement rate < 5% | Changing crash  
**Deps:** Retrieval (FT-001), ACO (FT-013-016), `rand` ^0.8  
**Traceability:** FT-021-ADAPTIVE-MUTATION | `src/ais/mutation.rs`

**Roadmap:** MVP: Gaussian mutate() + re-norm | Iter1: explore_mutations() + elitism + seed | Iter2: ACO integration + pipeline + metrics + adaptive rate

---

*Document generated on 2026-05-23 — KineContext Engine Product Specification*
