# FT-019 — Immune Response Amplifier

**Module:** AIS (Artificial Immune System) | **Version:** v6.0 | **Priority:** P1 — High  
**Artifact ID:** FT-019-RESPONSE-AMPLIFIER | **Update:** 2026-05-23

---

## 1. Context and Objective

The Immune Response Amplifier **automatically increases the weight and priority of critical decisions**. When "dangerous" context is detected (anomaly, antigen match, high severity scenario), the amplifier scales the processing priority, allocates more resources and reduces action thresholds. Inspired by clonal amplification of the immune system.

### Value
- Faster decisions in critical scenarios
- Dynamic priority allocation without manual rules
- Amplification proportional to severity

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: `amplify(context, trigger)` returns `AmplifiedContext` with adjusted priority
- [ ] AC-011: Amplification proportional to severity: LOW=1.5x, MEDIUM=2x, HIGH=3x, CRITICAL=5x
- [ ] AC-012: Supported triggers: anomaly_detection, antigen_match, manual_escalation
- [ ] AC-013: Amplification affects: MCE priority, timeout (extended), retry count (increased)
- [ ] AC-014: Amplification decay after resolution (does not remain indefinitely)
- [ ] AC-015: Maximum amplification = 5x (cap to avoid resource starvation)
- [ ] AC-016: Detailed log of each amplification for audit
- [ ] AC-017: Metrics: `kce_amplifications_total`, `kce_amplification_avg_factor`

---

## 3. Definition of Done (DoD)

- [ ] `amplify()` functional with 4 severity levels
- [ ] Integrated triggers (detection, antigen, manual)
- [ ] Post-resolution Decay
- [ ] Amplification cap (5x)
- [ ] Metrics and audit log
- [ ] Coverage ≥ 80%

---

## 4. Usage Examples

### Anomaly amplification

**Input:**
```json
{
  "context": { "request_id": "req_abc", "priority": 5, "timeout_ms": 100, "retries": 3 },
  "trigger": { "type": "anomaly_detection", "severity": "HIGH", "anomaly_score": 0.89 }
}
```

**Output (amplified context):**
```json
{
  "amplified_context": {
    "request_id": "req_abc",
    "priority": 15,
    "timeout_ms": 300,
    "retries": 9,
    "amplification_factor": 3.0,
    "trigger": "anomaly_detection",
    "decay_after_ms": 30000
  },
  "audit": {
    "original_priority": 5,
    "amplified_priority": 15,
    "reason": "HIGH severity anomaly (score: 0.89)"
  }
}
```

### Amplification by antigen match
```json
{
  "trigger": { "type": "antigen_match", "severity": "CRITICAL", "antigen_id": "ag_001" },
  "amplified_context": {
    "priority": 25,
    "amplification_factor": 5.0,
    "action": "BLOCK_AND_ALERT"
  }
}
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Expected Output |
|----|------|----------------|
| UT-001 | LOW amplification | factor = 1.5x |
| UT-002 | HIGH amplification | factor = 3.0x |
| UT-003 | CRITICAL Amplification | factor = 5.0x |
| UT-004 | Cap respected | factor ≤ 5.0x even with multiple triggers |
| UT-005 | Decay after resolution | priority returns to original |
| UT-006 | Priority adjusted | `priority * factor` |
| UT-007 | Extended timeout | `timeout * factor` |
| UT-008 | Increased retries | `retries * factor` |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | Invalid Severity | `Err(InvalidSeverity)` |
| UF-002 | Unknown trigger | `Err(UnknownTrigger)` |
| UF-003 | Null context | `Err(EmptyContext)` |

### 5.2 Functional Tests

| ID | Scenario | Result |
|----|---------|-----------|
| FT-001 | Anomaly → amplification → rapid response | Reduced decision latency |
| FT-002 | Multiple simultaneous triggers | Cap 5x respected |
| FT-003 | Decay after 30s | Priority returns to the original |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Detection (FT-017) → Amplifier → MCE | MCE processes with high priority |
| IT-002 | Antigen (FT-018) → Amplifier → Pipeline | Pipeline scales resources |
| IT-003 | Amplifier → Metrics | Registered amplifications |

---

## 6. CARE Format

**Context:** In critical scenarios (fraud, attack), normal response is too slow. Amplifier automatically scales priority and resources, inspired by immunological clonal amplification.

**Assumptions:** 4 severity levels are sufficient; 5x cap prevents resource starvation; decay is temporal (default 30s); amplification affects priority, timeout and retries.

**Requirements:** R-001: amplify() with 4 severities | R-002: 3 triggers | R-003: Cap 5x | R-004: Decay | R-005: Metrics + audit | R-006: Integration FT-017/FT-018.

**Evidence:** `tests/amplifier_tests.rs`

---

## 7–11. (Summary)

**Non-functional:** Overhead amplification < 0.5ms | Cap 5x never broken | Decay accuracy ± 1s  
**Quality:** 100% correct amplification | Cap always respected | Coverage ≥ 80%  
**Failure (BLOCKING):** Cap violated | Amplification does not decay | Priority overflow  
**Deps:** Detection (FT-017), Antigen (FT-018), MCE (FT-004), Pipeline (FT-011)  
**Traceability:** FT-019-RESPONSE-AMPLIFIER | `src/ais/amplifier.rs`

**Roadmap:** MVP: amplify() with fixed factor by severity | Iter1: 3 triggers + decay + cap | Iter2: Full integration + metrics + audit log

---

*Document generated on 2026-05-23 — KineContext Engine Product Specification*
