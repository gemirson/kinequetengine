# FT-011 — Robust Pipeline (Cognitive Pipeline Orchestration)

**Module:** Core Engine | **Version:** v5.9 | **Priority:** P0 — Critical  
**Artifact ID:** FT-011-PIPELINE | **Update:** 2026-05-23

---

## 1. Context and Objective

The Pipeline is the **orchestration** of all KCE modules in a deterministic chain: validate → rate_limit → plan → retrieve → expand → ecma_update → mce_encode → execute → persist → metrics. Each step returns `Result<T, EngineError>` — nothing assumes success. It is the operational heart of the system.

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: Pipeline returns `Result<ActionResult, EngineError>`
- [ ] AC-011: Each step uses `?` for error propagation
- [ ] AC-012: Failed at any step → clear error with step identified
- [ ] AC-013: Deterministic pipeline (same query 100x → same result)
- [ ] AC-014: Complete pipeline < 100ms (p95)
- [ ] AC-015: Metrics recorded at the end of each execution
- [ ] AC-016: Result persisted via KineSQL

---

## 3. Definition of Done (DoD)

- [ ] Complete pipeline implemented (10 steps)
- [ ] `Result<T, E>` at all stages
- [ ] Validated determinism (100x → variation < 1%)
- [ ] Built-in metrics and persistence
- [ ] End-to-end tests passing
- [ ] Coverage ≥ 80%

---

## 4. Usage Examples

### Complete pipeline
**Input:**
```json
{ "query_vector": [0.12, 0.85, 0.33, 0.67], "top_k": 5, "context": "loan_evaluation" }
```

**Internal flow:**
```
validate ✓ → rate_limit ✓ → plan ✓ → retrieve (42 candidates) ✓ 
→ expand (8 nodes) ✓ → ecma_update ✓ → mce_encode ✓ 
→ execute (risk_eval) ✓ → persist ✓ → metrics ✓
```

**Output:**
```json
{
  "action": "risk_eval",
  "result": { "risk_score": 0.72, "recommendation": "APPROVE_WITH_CONDITIONS" },
  "pipeline_ms": 45.2,
  "stages": {
    "validate": 0.1, "retrieve": 12.4, "expand": 3.2,
    "ecma": 1.1, "encode": 2.0, "execute": 2.3, "persist": 8.5
  }
}
```

### Error in step
```json
{
  "error": "PIPELINE_FAILURE",
  "stage": "retrieve",
  "message": "Dimension mismatch in retrieval",
  "code": 500
}
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Expected Output |
|----|------|----------------|
| UT-001 | Complete pipeline — success | `Ok(ActionResult)` |
| UT-002 | Failed to validate | `Err` with stage="validate" |
| UT-003 | Retrieve failure | `Err` with stage="retrieve" |
| UT-004 | 10x determinism | 10 identical results |

### 5.2 Functional Tests

| ID | Scenario | Result |
|----|---------|-----------|
| FT-001 | End-to-end pipeline | Correct final action |
| FT-002 | Same query 100x | Identical result |
| FT-003 | Pipeline under load | p95 < 100ms |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | API → Pipeline → KineSQL | HTTP full flow → persist |
| IT-002 | Pipeline → Metrics | Each run records metrics |

---

## 6. CARE Format

**Context:** Orchestration of the 6+ KCE modules in a deterministic pipeline with robust error handling; operational heart of the system.

**Assumptions:** Each module exposes `Result<T, E>`; pipeline is synchronous within a request; Metrics are recorded even on failure.

**Requirements:** R-001: `Result<T, E>` at all | R-002: 10 steps | R-003: Determinism | R-004: p95 < 100ms | R-005: Metrics + persist.

**Evidence:** `tests/pipeline_e2e.rs` | `benches/pipeline_bench.rs`

---

## 7–11. (Summary)

**Non-functional:** p95 < 100ms | Clear error with stage identified | 0 panics  
**Quality:** 100% Determinism | Stage identification in 100% of errors | Coverage ≥ 80%  
**Deps:** All core modules (Retrieval, Graph, ECMA, MCE, KineSQL)  
**Traceability:** FT-011-PIPELINE | `src/pipeline/mod.rs`

**Roadmap:** MVP: 5 basic steps (validate → retrieve → encode → execute → respond) | Iter1: +expand, +ecma, +persist, +metrics | Iter2: validated determinism + chaos testing + 1000 fps load
