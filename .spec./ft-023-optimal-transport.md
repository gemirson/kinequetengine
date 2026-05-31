# FT-023 — Optimal Transport Context Distance (Wasserstein)

**Module:** Core Engine — Distance Metrics | **Version:** v6.0 | **Priority:** P0 — Critical  
**Artifact ID:** FT-023-OPTIMAL-TRANSPORT | **Update:** 2026-05-23

---

## 1. Context and Objective

Optimal Transport (OT) Context Distance calculates the **distance between two contexts as the minimum cost of transforming one semantic distribution into the other**, using the Wasserstein metric. Unlike cosine similarity (which measures angle), OT captures **distributed structure**, multimodality, semantic displacement and context evolution.

### Formulation

```
W_p(μ, ν) = ( inf_{γ ∈ Π(μ,ν)} ∫ ‖x - y‖^p dγ(x, y) )^(1/p)
```

| Mathematical Concept | KCE Mapping |
|---------------------|-------------|
| μ, ν | CARE contexts (distributions) |
| γ | Optimal transportation plan |
| cost | Real semantic difference |

### Value
- **Brutal upgrade retrieval:** cosine → quick filter, OT → final ranking
- **ECMA real evolution:** measures how much a context has changed over time
- **Immune System:** detects context drift and real anomalies (not just outliers)
- **ACO:** edge cost = Wasserstein distance → realistic global optimization
- Multimodality capture, semantic shift and context evolution

### Performance Strategy

Pure OT is O(n³). Production solution:

1. **Sinkhorn Distance** (regularized) — fast + approximate
2. **Entropic Regularization** — computational cost control
3. **Batching + GPU** (future) — massive scale

---

## 2. Acceptance Criteria (AC)

### General
- [ ] AC-001: Compiles without warnings in `--release`
- [ ] AC-002: Thread-safe via `Arc<RwLock<...>>`
- [ ] AC-003: Public interface with doc comments

### Specifics — Mathematical Properties
- [ ] AC-010: Symmetric distance: `W(A, B) == W(B, A)`
- [ ] AC-011: Distance = 0 if same contexts: `W(A, A) == 0.0`
- [ ] AC-012: Triangular inequality: `W(A, C) <= W(A, B) + W(B, C)`
- [ ] AC-013: Preserved monotonicity: more different contexts → greater distance
- [ ] AC-014: Non-negativity: `W(A, B) >= 0.0` always

### Specifics — Implementation
- [ ] AC-020: `wasserstein(ctx_a, ctx_b)` returns distance f64
- [ ] AC-021: `sinkhorn(ctx_a, ctx_b, epsilon)` returns approximate distance (Sinkhorn)
- [ ] AC-022: Sinkhorn with entropic regularization (configurable ε, default: 0.01)
- [ ] AC-023: Sinkhorn Convergence in < 100 iterations (threshold: 1e-6)
- [ ] AC-024: Automatic fallback to cosine if Sinkhorn does not converge
- [ ] AC-025: Configurable cost matrix (euclidean, cosine, custom)
- [ ] AC-026: Support for distributions of different dimensions (padding with zeros)
- [ ] AC-027: Latency < 20ms (approximate Sinkhorn version, dim ≤ 128)
- [ ] AC-028: Integration as a metric in the Retrieval Engine (FT-001)
- [ ] AC-029: Integration as edge cost in ACO (FT-013)

---

## 3. Definition of Done (DoD)

- [ ] Exact Wasserstein distance implemented (baseline/reference)
- [ ] Sinkhorn distance (approximate) implemented for production
- [ ] Entropic regularization with configurable ε
- [ ] Validated mathematical properties (symmetry, identity, triangle)
- [ ] Fallback for functional cosine
- [ ] Retrieval integration (cosine → OT ranking)
- [ ] ACO integration (edge cost)
- [ ] ECMA integration (context drift measurement)
- [ ] Immune integration (anomaly via distribution shift)
- [ ] Benchmark: Sinkhorn < 20ms for dim ≤ 128
- [ ] Unit tests ≥ 80% coverage
- [ ] No `unwrap()` in production code

---

## 4. Usage Examples

### 4.1 Basic distance between contexts

**Input:**
```json
{
  "ctx_a": [0.2, 0.8],
  "ctx_b": [0.6, 0.4],
  "method": "sinkhorn",
  "epsilon": 0.01
}
```

**Output:**
```json
{
  "wasserstein_distance": 0.32,
  "method": "sinkhorn",
  "iterations": 23,
  "converged": true,
  "computation_ms": 1.2
}
```

### 4.2 Identical contexts

**Input:**
```json
{
  "ctx_a": [0.5, 0.3, 0.2],
  "ctx_b": [0.5, 0.3, 0.2]
}
```

**Output:**
```json
{
  "wasserstein_distance": 0.0,
  "method": "sinkhorn",
  "iterations": 1,
  "converged": true
}
```

### 4.3 Hybrid retrieval (cosine → OT ranking)

**Pipeline:**
```json
{
  "query": [0.12, 0.85, 0.33, 0.67],
  "stage_1": {
    "method": "cosine",
    "top_k": 50,
    "latency_ms": 8.2,
    "purpose": "fast pre-filter"
  },
  "stage_2": {
    "method": "wasserstein_sinkhorn",
    "top_k": 10,
    "latency_ms": 14.5,
    "purpose": "precise re-ranking"
  },
  "final_results": [
    { "id": 42, "cosine_score": 0.91, "ot_distance": 0.08, "final_rank": 1 },
    { "id": 17, "cosine_score": 0.95, "ot_distance": 0.15, "final_rank": 2 }
  ],
  "note": "id=17 had higher cosine but worse OT → OT captured structural difference"
}
```

### 4.4 ECMA — Context Drift Measurement

**Input:**
```json
{
  "node_id": 42,
  "context_t0": [0.2, 0.8, 0.0],
  "context_t1": [0.3, 0.6, 0.1],
  "context_t2": [0.5, 0.3, 0.2]
}
```

**Output:**
```json
{
  "drift": [
    { "from": "t0", "to": "t1", "distance": 0.18 },
    { "from": "t1", "to": "t2", "distance": 0.27 },
    { "from": "t0", "to": "t2", "distance": 0.42 }
  ],
  "drift_rate": 0.21,
  "trend": "DIVERGING",
  "maturity_impact": -0.05
}
```

### 4.5 Immune — Anomaly via Distribution Shift

**Input:**
```json
{
  "baseline_distribution": [0.2, 0.5, 0.3],
  "current_distribution": [0.8, 0.1, 0.1],
  "threshold": 0.4
}
```

**Output:**
```json
{
  "ot_distance": 0.52,
  "is_anomaly": true,
  "detail": "Distribution shift exceeds threshold (0.52 > 0.40)",
  "action": "ESCALATE_TO_IMMUNE_DETECTION"
}
```

### 4.6 Error — Non-convergence → fallback

```json
{
  "error": "SINKHORN_NOT_CONVERGED",
  "iterations": 100,
  "residual": 0.05,
  "fallback": "cosine",
  "cosine_distance": 0.23,
  "warning": "Sinkhorn did not converge; using cosine fallback"
}
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Entry | Expected Output |
|----|------|---------|----------------|
| UT-001 | Identity | `W(a, a)` | `0.0` |
| UT-002 | Symmetry | `W(a, b)` vs `W(b, a)` | equal |
| UT-003 | Non-negativity | any pair | `≥ 0.0` |
| UT-004 | Triangle | `W(a,c)` vs `W(a,b) + W(b,c)` | `W(a,c) ≤ sum` |
| UT-005 | Monotonicity | a close to b, c far away | `W(a,b) < W(a,c)` |
| UT-006 | Sinkhorn convergence | ε=0.01, dim=4 | converges < 100 iter |
| UT-007 | Sinkhorn vs exact | dim=4 | difference < 5% |
| UT-008 | High epsilon (ε=1.0) | any | more blur, faster |
| UT-009 | Low epsilon (ε=0.001) | any | more accurate, slower |
| UT-010 | Different dims | dim=3 vs dim=5 | padding + valid result |
| UT-011 | Uniform distributions | [0.25, 0.25, 0.25, 0.25] x2 | `0.0` |
| UT-012 | Dirac distributions | [1,0,0] vs [0,0,1] | maximum distance |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | Distribution with negatives | `Err(InvalidDistribution)` |
| UF-002 | Distribution does not add up to 1 | `Err(NotNormalized)` or auto-normalize |
| UF-003 | Empty vector | `Err(EmptyVector)` |
| UF-004 | NaN in distribution | `Err(InvalidValue)` |
| UF-005 | ε = 0 | `Err(InvalidEpsilon)` — division by zero |
| UF-006 | Sinkhorn does not converge | Fallback cosine + warning |

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasserstein_identity() {
        let a = vec![0.2, 0.8];
        assert!((wasserstein(&a, &a) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_wasserstein_symmetry() {
        let a = vec![0.2, 0.8];
        let b = vec![0.6, 0.4];
        assert!((wasserstein(&a, &b) - wasserstein(&b, &a)).abs() < 1e-10);
    }

    #[test]
    fn test_triangle_inequality() {
        let a = vec![0.1, 0.9];
        let b = vec![0.5, 0.5];
        let c = vec![0.9, 0.1];
        assert!(wasserstein(&a, &c) <= wasserstein(&a, &b) + wasserstein(&b, &c) + 1e-10);
    }

    #[test]
    fn test_sinkhorn_convergence() {
        let a = vec![0.2, 0.3, 0.5];
        let b = vec![0.4, 0.4, 0.2];
        let result = sinkhorn(&a, &b, 0.01, 100);
        assert!(result.converged);
        assert!(result.iterations < 100);
    }

    #[test]
    fn test_sinkhorn_fallback() {
        let a = vec![0.5, 0.5];
        let b = vec![0.5, 0.5];
        let result = sinkhorn_with_fallback(&a, &b, 0.0001, 5); // max 5 iter
        // Should fallback to cosine if not converged
        assert!(result.distance.is_finite());
    }
}
```

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | Hybrid retrieval: cosine top-50 → OT re-rank top-10 | OT re-ranking improves precision@10 ≥ 5% |
| FT-002 | Compare 100 pairs of clusters | OT separates clusters better than cosine |
| FT-003 | Validated semantic ordering | OT ranking correlates with human judgment |
| FT-004 | Performance dim=128, 50 pairs | Sinkhorn < 20ms total |
| FT-005 | Context drift 1000 timesteps | Correctly calculated drift rate |
| FT-006 | Fallback under load | Cosine activates when Sinkhorn timeout |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | OT → Retrieval (FT-001) | Functional post-cosine re-ranking |
| IT-002 | OT → ECMA (FT-003) | Context drift measures real evolution |
| IT-003 | OT → Immune Detection (FT-017) | Distribution shift detects anomalies |
| IT-004 | OT → ACO Pheromone (FT-013) | Edge cost = Wasserstein |
| IT-005 | OT → Pipeline (FT-011) | OT metric available in the pipeline |
| IT-006 | OT → Metrics (FT-007) | `kce_ot_latency_ms`, `kce_ot_fallbacks` |

---

## 6. CARE Format

### Context
Most retrieval systems use cosine similarity, which only measures the angle between vectors. This ignores the **distributed structure** of context. Optimal Transport (Wasserstein distance) captures the minimum cost of transforming one semantic distribution into another, revealing multimodality, semantic shift and context evolution that cosine cannot detect.

### Assumptions
- Contexts are representable as probability distributions (normalized, non-negative)
- Sinkhorn (regularized) is sufficient for production — exact OT as baseline/reference only
- Entropic regularization with ε=0.01 provides good accuracy/speed tradeoff
- Standard cost matrix is Euclidean; cosine available as an alternative
- Sinkhorn convergence in < 100 iterations for most inputs
- Fallback to cosine is acceptable when Sinkhorn does not converge
- GPU acceleration is future roadmap (not MVP)

### Requirements
- R-001: Exact Wasserstein distance (reference/tests)
- R-002: Sinkhorn distance (production) with configurable ε
- R-003: Validated metric properties (symmetry, identity, triangle)
- R-004: Latency < 20ms (Sinkhorn, dim ≤ 128)
- R-005: Automatic fallback to cosine
- R-006: Retrieval Integration (re-ranking)
- R-007: ECMA integration (context drift)
- R-008: Immune Integration (distribution shift)
- R-009: ACO integration (edge cost)
- R-010: Configurable cost matrix

### Evidence
- `benches/ot_bench.rs` — Sinkhorn benchmark vs exact vs cosine
- `tests/ot_properties.rs` — validation of metric properties
- `reports/ot_vs_cosine.md` — comparative precision analysis
- `reports/sinkhorn_convergence.md` — convergence analysis by ε

---

## 7. Non-Functional Acceptance Criteria

### Performance

| Metric | Target | Method |
|---------|------|--------|
| Sinkhorn latency (dim=4) | < 1ms | Benchmark |
| Sinkhorn latency (dim=128) | < 20ms | Benchmark |
| Sinkhorn latency (dim=512) | < 100ms | Benchmark |
| Exact latency (dim=128) | < 500ms | Benchmark (ref only) |
| Re-ranking 50 candidates | < 50ms | Benchmark |
| Sinkhorn Convergence | < 100 iterations | Test |
| Fallback rate | < 5% | Monitoring |

### Security
- Validated inputs (non-negative, finite, normalized)
- No buffer overflow in the cost matrix (bounds checking)
- ε validated (> 0, < 10)

### Accessibility
- N/A (backend module)

---

## 8. Quality Criteria and Metrics

### Success Metrics

| Metric | Target Value |
|---------|------------|
| Precision@10 improvement vs cosine-only | ≥ 5% |
| Sinkhorn vs exact relative error | < 5% (ε=0.01) |
| Metric properties | 100% validated |
| Context drift correlation with real evolution | ≥ 0.80 |
| Test coverage | ≥ 80% |

### Failure Criteria
- Violated metric property (symmetry, triangle) → **BLOCKING**
- Sinkhorn latency > 50ms (dim ≤ 128) → **BLOCKING**
- Precision@10 worse than cosine-only → **BLOCKING**
- Sinkhorn vs exact error > 20% → **BLOCKING**
- Fallback rate > 20% → **WARNING**

---

## 9. Compatibility and Dependencies

### Direct Dependencies

| Crate | Version | Purpose |
|-------|--------|-----------|
| `ndarray` | ^0.15 | Matrices for cost matrix and transport plan |
| `parking_lot` | ^0.12 | RwLock |

### Internal Dependencies

| Module | Type of Integration |
|--------|--------------------|
| Retrieval (FT-001) | Re-ranking metric (cosine → OT) |
| ECMA (FT-003) | Context drift measurement |
| ACO Pheromone (FT-013) | Edge cost = Wasserstein |
| Immune Detection (FT-017) | Distribution shift anomaly |
| Pipeline (FT-011) | Metric available in the pipeline |
| Metrics (FT-007) | `kce_ot_*` exposed |

### Compatibility

| Item | Requirement |
|------|-----------|
| Rust | ≥ 1.75 stable |
| OS | Linux (prod), macOS (dev) |
| CPU | x86_64 (SIMD for future matrix ops) |

---

## 10. Traceability Criteria

### Change History

| Date | Version | Description | Author |
|------|--------|-----------|-------|
| 2026-05-23 | v1.0 | Specification creation | Product Specialist |

### Related Artifact IDs

| Type | ID | Description |
|------|----|-----------|
| Spec | FT-023-OPTIMAL-TRANSPORT | This specification |
| Code | `src/metrics/optimal_transport.rs` | OT implementation |
| Code | `src/metrics/sinkhorn.rs` | Regularized Sinkhorn |
| Code | `src/metrics/cost_matrix.rs` | Cost matrices |
| Bench | `benches/ot_bench.rs` | Benchmark |
| Test | `tests/ot_properties.rs` | Metric properties |
| Test | `tests/ot_integration.rs` | Integration |
| Deps | FT-001, FT-003, FT-013, FT-017 | Built-in features |

---

## 11. MVP Delivery and Acceptance Criteria — Roadmap

### MVP (Week 1-2)

| Item | Acceptance Criteria |
|------|--------------------|
| Exact 1D Wasserstein | Exact distance for 1D distributions (sorting-based, O(n log n)) |
| Sinkhorn distance | Basic implementation with fixed ε (0.01) |
| Euclidean cost matrix | Functional for dims ≤ 128 |
| Metric properties | Symmetry, identity, non-negativity validated |
| 8+ unit tests | Passing (including properties) |
| Internal API | `fn wasserstein(a, b) -> f64` + `fn sinkhorn(a, b, eps) -> SinkhornResult` |

**Output:** Functional OT as internal metrics library.

---

### Iteration 1 (Week 3-4)

| Item | Acceptance Criteria |
|------|--------------------|
| Entropic regularization | ε configurable with validation |
| Adaptive convergence | Configurable threshold + max_iter |
| Fallback to cosine | Automatic when Sinkhorn does not converge |
| Customizable cost matrix | Euclidean, cosine, custom fn |
| Retrieval Integration | cosine top-50 → OT re-rank top-10 |
| Benchmark | Sinkhorn < 20ms (dim ≤ 128) |
| Triangular inequality | Numerically validated |

**Output:** OT integrated into Retrieval as a re-ranking metric.

---

### Iteration 2 (Week 5-6)

| Item | Acceptance Criteria |
|------|--------------------|
| ECMA Integration | Functional context drift measurement |
| Immune Integration | Distribution shift → anomaly detection |
| ACO Integration | Edge cost = Wasserstein |
| batch computation | Multiple pairs in parallel (Rayon) |
| Prometheus Metrics | `kce_ot_latency_ms`, `kce_ot_fallbacks`, `kce_ot_convergence_iter` |
| Precision@10 benchmark | ≥ 5% improvement vs cosine-only |
| Dims ≤ 512 | Sinkhorn < 100ms |
| Complete documentation | API docs + mathematical reference + examples |

**Output:** OT production-ready integrated into 4 KCE core modules.

---

### Future (Backlog)

| Item | Description |
|------|-----------|
| GPU acceleration | cuML/CUDA Sinkhorn for high dims |
| Sliced Wasserstein | 1D projection for ultra-fast approximation |
| Wasserstein barycenter | Center of mass of multiple contexts |
| Online OT | Sinkhorn streaming for incremental updates |

---

*Document generated on 2026-05-23 — KineContext Engine Product Specification*
