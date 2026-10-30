# FT-001 — Retrieval Engine (Hybrid Search)

**Module:** Core Engine | **Version:** v5.9 | **Priority:** P0 — Critical  
**Artifact ID:** FT-001-RETRIEVAL-ENGINE | **Updated:** 2026-05-23

---

## 1. Context and Objective

The Retrieval Engine is the first stage of the KCE cognitive pipeline. It performs hybrid searches combining **cosine similarity** (SIMD/AVX2) and **prime similarity** (GCD). It receives a vector query and returns relevant candidates for the Graph Engine and ECMA. It must operate with datasets of up to 1M+ vectors in production.

---

## 2. Acceptance Criteria (AC)

### General
- [ ] AC-001: Compiles without warnings in `--release`
- [ ] AC-002: Thread-safe via `Arc<RwLock<...>>` / `parking_lot`
- [ ] AC-003: Public interface with doc comments

### Specific
- [ ] AC-010: Cosine similarity: `1.0` for identical vectors, `0.0` for orthogonal, `-1.0` for opposite
- [ ] AC-011: Prime similarity via normalized GCD
- [ ] AC-012: Hybrid search with configurable weights (`cosine_weight`, `prime_weight`)
- [ ] AC-013: SIMD (AVX2) active with safe fallback
- [ ] AC-014: Early pruning with parameterizable minimum threshold
- [ ] AC-015: Deterministic sorting (score desc, id asc as tiebreaker)
- [ ] AC-016: Rayon with configurable `max_threads`
- [ ] AC-017: Recall@10 ≥ 0.85 (synthetic dataset)
- [ ] AC-018: p95 < 50ms for 100k vectors

---

## 3. Definition of Done (DoD)

- [ ] SIMD active (AVX2 or fallback)
- [ ] Prime similarity implemented and tested
- [ ] Rayon parallelism functional and configurable
- [ ] Functional early pruning
- [ ] Guaranteed determinism (variation < 1%)
- [ ] Unit tests ≥ 80% coverage
- [ ] Functional tests passing in CI
- [ ] Baseline benchmark registered
- [ ] No `unwrap()` in production code
- [ ] Code review approved

---

## 4. Usage Examples

### Hybrid search top-5

**Input (JSON):**
```json
{
  "query_vector": [0.12, 0.85, 0.33, 0.67],
  "top_k": 5,
  "similarity": "hybrid",
  "weights": { "cosine": 0.7, "prime": 0.3 },
  "threshold": 0.1
}
```

**Output:**
```json
{
  "results": [
    { "id": 42, "score": 0.9523, "cosine_score": 0.9712, "prime_score": 0.9082 },
    { "id": 17, "score": 0.8891, "cosine_score": 0.9001, "prime_score": 0.8635 }
  ],
  "latency_ms": 12.4,
  "total_candidates": 100000,
  "pruned_candidates": 87234
}
```

### Error — dimension mismatch
```json
{ "error": "DIMENSION_MISMATCH", "message": "Query dim (3) != dataset dim (4)", "code": 400 }
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Input | Expected Output |
|----|------|---------|----------------|
| UT-001 | Cosine — identical vectors | `[1,0], [1,0]` | `1.0` |
| UT-002 | Cosine — orthogonal | `[1,0], [0,1]` | `0.0` |
| UT-003 | Cosine — opposite | `[1,0], [-1,0]` | `-1.0` |
| UT-004 | Cosine — zero vector | `[0,0], [1,0]` | `0.0` (no panic) |
| UT-005 | Prime — maximum GCD | `12, 12` | `1.0` |
| UT-006 | Hybrid — 50/50 weight | cos=0.8, prime=0.6 | `0.7` |
| UT-007 | Top-k > dataset | dataset=3, k=10 | `3 results` |
| UT-008 | Ordering — equal scores | score=0.5, score=0.5 | `smallest id first` |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | Dimension mismatch | `Err(DimensionMismatch)` |
| UF-002 | Empty vector | `Err(EmptyVector)` |
| UF-003 | Top-k = 0 | `Err(InvalidK)` |
| UF-004 | NaN in vector | `Err(InvalidVector)` |

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | Insert 10k vectors + query | Consistent Top-10, descending scores |
| FT-002 | 50 concurrent queries | All return without error |
| FT-003 | 100k vectors performance | p95 < 50ms |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Retrieval → Graph | Candidates expanded with neighbors |
| IT-002 | Retrieval → ECMA | Node maturity updated |
| IT-003 | API → Retrieval | HTTP 200 with valid JSON |

---

## 6. CARE Format

**Context:** First stage of the cognitive pipeline; hybrid vector search for high-quality semantic retrieval in datasets up to 1M+ vectors.

**Assumptions:** Target CPU supports AVX2 (fallback exists); L2 normalized vectors; dataset fits in memory/mmap; concurrency via `Arc<RwLock>`.

**Requirements:** R-001: SIMD cosine | R-002: Prime via GCD | R-003: Hybrid with weights | R-004: p95 < 50ms (100k) | R-005: Recall@10 ≥ 0.85 | R-006: Determinism < 1% variation.

**Evidence:** `benches/retrieval_bench.rs` | `testdata/synth_100k_128d.bin` | `reports/recall_analysis.md`

---

## 7. Non-Functional Criteria

| Aspect | Metric | Target |
|---------|---------|------|
| Latency p50 | 100k vectors | < 20ms |
| Latency p95 | 100k vectors | < 50ms |
| Throughput | queries/s | ≥ 500 |
| Memory | vs dataset size | < 2x |

---

## 8. Quality and Metrics

**Success:** Recall@10 ≥ 0.85 | Precision@10 ≥ 0.70 | Variation < 1% | Coverage ≥ 80%

**Failure (BLOCKING):** Recall@10 < 0.70 | p95 > 100ms | Crash in functional test | Variation > 5%

---

## 9. Compatibility and Dependencies

| Crate | Version | Purpose |
|-------|--------|-----------|
| `rayon` | ^1.8 | Parallelism |
| `parking_lot` | ^0.12 | Optimized RwLock |

**Internal deps:** KineSQL (vector reading), Graph Engine (candidate consumption), ECMA (maturity)  
**Rust:** ≥ 1.75 | **OS:** Linux (prod), macOS (dev) | **CPU:** x86_64 AVX2 (preferred)

---

## 10. Traceability

| Type | ID | Description |
|------|----|-----------|
| Spec | FT-001-RETRIEVAL-ENGINE | This specification |
| Code | `src/retrieval/mod.rs` | Implementation |
| Bench | `benches/retrieval_bench.rs` | Benchmark |
| Test | `tests/retrieval_integration.rs` | Integration |

---

## 11. MVP Roadmap

### MVP (Week 1-2)
Scalar cosine similarity | Top-k retrieval | 5+ unit tests | Internal API `retrieve(query, dataset, k)`

### Iteration 1 (Week 3-4)
SIMD AVX2 + fallback | Prime similarity | Hybrid scoring | Rayon | Baseline benchmark | p95 < 50ms

### Iteration 2 (Week 5-6)
Early pruning | Validated determinism | Complete error handling | KineSQL + Graph integration | 500 qps load tests | Complete docs
