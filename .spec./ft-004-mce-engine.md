# FT-004 — MCE (mRNA Cognitive Engine)

**Module:** Core Engine | **Version:** v5.9 | **Priority:** P0 — Critical  
**Artifact ID:** FT-004-MCE-ENGINE | **Updated:** 2026-05-23

---

## 1. Context and Objective

MCE transforms **semantic context into executable instructions** (mRNA). It receives enriched nodes from ECMA, encodes them into compact payloads with intent, priority, and TTL, and executes the corresponding action. It is the final stage of the cognitive pipeline — where knowledge turns into action.

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: Encoding transforms ECMA nodes into `mRNA` struct with `intent`, `priority`, `ttl`, `payload`
- [ ] AC-011: Payload reduction > 50% vs raw input
- [ ] AC-012: mRNA execution < 5ms
- [ ] AC-013: TTL is respected — expired mRNA does not execute
- [ ] AC-014: Priority automatically calculated based on node maturity
- [ ] AC-015: Action consistency: same input → same action
- [ ] AC-016: Result feedback feeds back to ECMA
- [ ] AC-017: Support for multiple intents (`risk_eval`, `data_enrich`, `alert`, `classify`)

---

## 3. Definition of Done (DoD)

- [ ] Functional encoding with reduction > 50%
- [ ] Functional execution < 5ms
- [ ] TTL applied and validated
- [ ] Automatically calculated priority
- [ ] Feedback loop → ECMA
- [ ] Unit tests ≥ 80%
- [ ] No `unwrap()` in production

---

## 4. Usage Examples

### Encoding + Execution

**Input (ECMA context):**
```json
{
  "nodes": [
    { "id": 42, "state": "Specialized", "maturity": 0.87, "data": { "type": "credit_risk", "value": 0.72 } }
  ],
  "context": "loan_evaluation"
}
```

**Generated mRNA:**
```json
{
  "intent": "risk_eval",
  "priority": 8,
  "ttl_ms": 5000,
  "payload": "0x03A2F1...",
  "payload_size_bytes": 64,
  "original_size_bytes": 156,
  "compression_ratio": 0.59
}
```

**Execution result:**
```json
{
  "action": "risk_eval",
  "result": { "risk_score": 0.72, "recommendation": "APPROVE_WITH_CONDITIONS" },
  "execution_ms": 2.3,
  "feedback": { "success": true, "maturity_delta": 0.02 }
}
```

### Error — expired mRNA
```json
{ "error": "MRNA_EXPIRED", "message": "TTL exceeded (5000ms)", "code": 408 }
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Input | Expected Output |
|----|------|---------|----------------|
| UT-001 | Encoding generates correct intent | context="risk" | `mrna.intent == "risk_eval"` |
| UT-002 | Compression ratio > 50% | payload 156 bytes | output < 78 bytes |
| UT-003 | Valid TTL | ttl=5000ms, age=1000ms | executes normally |
| UT-004 | Expired TTL | ttl=100ms, age=200ms | `Err(MrnaExpired)` |
| UT-005 | Priority by maturity | maturity=0.87 | `priority >= 7` |
| UT-006 | Determinism | same input 2x | same output |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | Node without data | `Err(EmptyPayload)` |
| UF-002 | Unknown intent | `Err(UnknownIntent)` |
| UF-003 | TTL = 0 | `Err(InvalidTTL)` |

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | Full pipeline: encode → execute | Correct final action |
| FT-002 | Batch of 100 mRNAs | All execute < 5ms |
| FT-003 | mRNA with short TTL under load | Expired rejected correctly |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | ECMA → MCE | Specialized nodes generate valid mRNA |
| IT-002 | MCE → ECMA (feedback) | Result feeds back maturity |
| IT-003 | API → MCE | functional POST /encode + POST /action |

---

## 6. CARE Format

**Context:** Last stage of the cognitive pipeline; transforms processed knowledge into executable actions via compact encoding inspired by molecular biology (mRNA).

**Assumptions:** Input nodes are validated by ECMA; intents are finite and known; TTL is defined by the caller or calculated by priority; execution is synchronous.

**Requirements:** R-001: Encoding with reduction > 50% | R-002: Execution < 5ms | R-003: TTL enforcement | R-004: Automatic priority | R-005: Feedback → ECMA | R-006: Determinism.

**Evidence:** `tests/mce_tests.rs` | `benches/mce_bench.rs`

---

## 7. Non-Functional Criteria

| Aspect | Target |
|---------|------|
| Encoding latency | < 2ms |
| Execution latency | < 5ms |
| Compression ratio | > 50% |
| Throughput | ≥ 1000 mRNA/s |

---

## 8. Quality and Metrics

**Success:** Compression > 50% | Execution < 5ms | 100% determinism | Coverage ≥ 80%  
**Failure (BLOCKING):** Compression < 30% | Execution > 20ms | Inconsistent action for same input

---

## 9. Compatibility and Dependencies

**Internal deps:** ECMA (input), KineSQL (result persistence), API (/encode, /action endpoints)  
**Rust:** ≥ 1.75

---

## 10. Traceability

| Type | ID |
|------|----|
| Spec | FT-004-MCE-ENGINE |
| Code | `src/mce/mod.rs` |
| Test | `tests/mce_integration.rs` |

---

## 11. MVP Roadmap

### MVP (Week 1-2)
Basic encoding | Synchronous execution | 1 intent (`risk_eval`) | 5+ tests

### Iteration 1 (Week 3-4)
TTL enforcement | Automatic priority | Multiple intents | Optimized compression | Benchmark

### Iteration 2 (Week 5-6)
ECMA feedback loop | API integration | Batch execution | Load tests | Docs
