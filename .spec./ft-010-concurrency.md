# FT-010 — Concurrency (Thread Safety)

**Module:** Infrastructure | **Version:** v5.9 | **Priority:** P0 — Critical  
**Artifact ID:** FT-010-CONCURRENCY | **Update:** 2026-05-23

---

## 1. Context and Objective

KCE v5 is single-thread safe "luckily". For production, **safe concurrency is mandatory**. All mutable structures must use `Arc<RwLock<...>>` (via `parking_lot` for performance). Idempotence of requests via cache + request_id.

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: Mutable structures protected by `Arc<RwLock<...>>`
- [ ] AC-011: `parking_lot` used instead of `std::sync`
- [ ] AC-012: Read lock for reading, write lock for writing
- [ ] AC-013: No deadlocks under concurrent load (100 threads)
- [ ] AC-014: Request idempotency via request_id + cache
- [ ] AC-015: Deterministic pipeline under parallelism (stable ordering)

---

## 3. Definition of Done (DoD)

- [ ] All structures mutable with `Arc<RwLock<...>>`
- [ ] Built-in `parking_lot`
- [ ] Idempotence via request cache
- [ ] Stable ordering in the pipeline
- [ ] Tests with `#[test]` + concurrent threads
- [ ] 0 deadlocks in stress test

---

## 4. Usage Examples

### SharedState
```rust
pub struct SharedState {
    pub db: Arc<RwLock<KineSQL>>,
    pub cache: Arc<RwLock<HashMap<String, Response>>>,
}

// Leitura
let db = state.db.read();
// Escrita
let mut db = state.db.write();
```

### Idempotence
```json
// Request 1
{ "request_id": "req_abc", "query_vector": [0.5, 0.5] }
// Response: computed and cached

// Request 2 (mesmo request_id)
{ "request_id": "req_abc", "query_vector": [0.5, 0.5] }
// Response: returned from cache (idêntica)
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Expected Output |
|----|------|----------------|
| UT-001 | Competitor read lock | 50 simultaneous readers OK |
| UT-002 | Exclusive write lock | Writer blocks readers |
| UT-003 | Idempotence | duplicate request_id → cache hit |
| UT-004 | Stable ordering | same query, different threads → same order |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | Write lock timeout | `Err` after timeout (no deadlock) |

### 5.2 Functional Tests

| ID | Scenario | Result |
|----|---------|-----------|
| FT-001 | 100 concurrent threads | 0 panics, 0 deadlocks |
| FT-002 | Simultaneous read + write | Maintained consistency |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | API → SharedState → Pipeline | Secure end-to-end concurrency |

---

## 6. CARE Format

**Context:** System needs to support 1000+ concurrent req/s; single-thread "luckily" is not acceptable in production.

**Assumptions:** `parking_lot` is faster than `std::sync`; read-heavy workload (90% reads); In-memory cache idempotence with TTL.

**Requirements:** R-001: Arc<RwLock> at all mutable | R-002: parking_lot | R-003: 0 deadlocks | R-004: idempotence | R-005: determinism.

**Evidence:** `tests/concurrency_stress.rs`

---

## 7–11. (Summary)

**Non-functional:** Lock overhead < 1μs | 0 deadlocks | Throughput ≥ 1000 req/s  
**Quality:** 0 deadlocks | 0 data races | Coverage ≥ 80%  
**Deps:** `parking_lot` ^0.12  
**Traceability:** FT-010-CONCURRENCY | `src/state.rs`

**Roadmap:** MVP: Arc<RwLock> + parking_lot | Iter1: cache idempotence | Iter2: lock-free structures for hot paths
