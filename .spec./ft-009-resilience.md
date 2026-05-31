# FT-009 — Resilience (Retry + Timeout + Circuit Breaker + Backpressure)

**Module:** Operation | **Version:** v5.9 | **Priority:** P0 — Critical  
**Artifact ID:** FT-009-RESILIENCE | **Update:** 2026-05-23

---

## 1. Context and Objective

Resilience ensures that KCE **does not collapse under partial failure**. It implements four mechanisms: retry with backoff, timeout per request, circuit breaker for fault isolation, and backpressure (semaphore) to control concurrency. Without this, the system dies at peak load.

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: Timeout configurable per request (default 100ms)
- [ ] AC-011: Retry with exponential backoff (max 3 attempts)
- [ ] AC-012: Circuit breaker opens when error_rate > 20%
- [ ] AC-013: Circuit breaker closes after configurable cooldown
- [ ] AC-014: Backpressure via Semaphore (max 100 concurrent requests)
- [ ] AC-015: Partial failure does not bring down the system
- [ ] AC-016: Latency controlled under error (p95 < 200ms even with failures)

---

## 3. Definition of Done (DoD)

- [ ] Timeout implemented with `tokio::time::timeout`
- [ ] Retry with functional backoff
- [ ] Circuit breaker with Open/Closed/HalfOpen states
- [ ] Semaphore for backpressure
- [ ] Partial failure tests passing
- [ ] Coverage ≥ 80%

---

## 4. Usage Examples

### Timeout
```json
// Request que excede timeout
{ "error": "TIMEOUT", "message": "Request exceeded 100ms timeout", "code": 408 }
```

### Open Circuit Breaker
```json
{ "error": "CIRCUIT_OPEN", "message": "Too many errors (rate: 0.25). Circuit will retry in 30s", "code": 503 }
```

### Backpressure
```json
{ "error": "OVERLOADED", "message": "Max concurrent requests (100) reached", "code": 429 }
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Expected Output |
|----|------|----------------|
| UT-001 | Timeout — fast operation | Success |
| UT-002 | Timeout — slow operation | `Err(Timeout)` |
| UT-003 | Retry — success on 2nd attempt | Success after 1 retry |
| UT-004 | Retry — fails after max attempts | `Err` after 3 attempts |
| UT-005 | Circuit breaker — error_rate < 0.2 | closed circuit |
| UT-006 | Circuit breaker — error_rate > 0.2 | Circuit opens |
| UT-007 | Semaphore — within the limit | Request processed |
| UT-008 | Semaphore — limit exceeded | 429 |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | Cascade failure | Circuit breaker prevents cascading |
| UF-002 | Timeout + retry | Retry respects total timeout |

### 5.2 Functional Tests

| ID | Scenario | Result |
|----|---------|-----------|
| FT-001 | 200 concurrent requests (limit 100) | 100 OK, 100 → 429 |
| FT-002 | Simulated failure > 20% | Circuit opens → 503 |
| FT-003 | Latency under error | p95 < 200ms |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | API → Timeout → Pipeline | Pipeline canceled in timeout |
| IT-002 | Circuit → KineSQL error | KineSQL failure → circuit opens |

---

## 6. CARE Format

**Context:** Real production = failures happen. Without resilience, partial failure becomes total collapse. Retry + timeout + circuit breaker + backpressure are mandatory.

**Assumptions:** Error rate calculated in a 60s window; exponential backoff retry (50ms, 100ms, 200ms); in-memory semaphore; circuit breaker with 30s cooldown.

**Requirements:** R-001: Timeout | R-002: Retry backoff | R-003: 3-state circuit breaker | R-004: Semaphore backpressure | R-005: Partial failure ≠ collapse | R-006: p95 < 200ms under error.

**Evidence:** `tests/resilience_tests.rs` | `benches/load_test.rs`

---

## 7–11. (Summary)

**Non-functional:** Resilience overhead < 5ms | Latency under p95 error < 200ms | 0 cascading failures  
**Quality:** Circuit prevents 100% of waterfalls | Retry recovers ≥ 50% of transient errors | Coverage ≥ 80%  
**Deps:** `tokio` ^1.35 (timeout, semaphore), `retry` ^2  
**Traceability:** FT-009-RESILIENCE | `src/resilience/mod.rs`

**Roadmap:** MVP: timeout + semaphore | Iter1: retry backoff + circuit breaker | Iter2: adaptive timeout + chaos testing
