# FT-022 — Immune Network Regulation

**Module:** AIS (Artificial Immune System) | **Version:** v6.0 | **Priority:** P0 — Critical  
**Artifact ID:** FT-022-NETWORK-REGULATION | **Update:** 2026-05-23

---

## 1. Context and Objective

Immune Network Regulation is the **global balance controller** of the KCE system. Regulates the interaction between all AIS and ACO modules, avoiding hyperactivity (overfitting), under-activity (loss of sensitivity) and uncontrolled oscillations. Inspired by Jerne's immune network theory — antibodies regulate each other to maintain homeostasis.

### Value
- Systemic stability — prevents oscillations and cascades
- Prevents hyperactivity (overfitting/excess amplification)
- Automatically regulates interaction between modules
- Homeostasis: system maintains balance without manual intervention

---

## 2. Acceptance Criteria (AC)

### General
- [ ] AC-001: Compiles without warnings in `--release`
- [ ] AC-002: Thread-safe for concurrent monitoring

### Specifics
- [ ] AC-010: `regulate()` monitors metrics from all AIS/ACO modules and applies adjustments
- [ ] AC-011: Detects hyperactivity: amplification_rate > threshold → reduces sensitivity
- [ ] AC-012: Detects sub-activity: detection_rate < min → increases sensitivity
- [ ] AC-013: Regulates mutation rate: high convergence → increases rate; divergence → decreases
- [ ] AC-014: Regulates evaporation: stagnation → increases ρ; volatility → decreases ρ
- [ ] AC-015: Feedback loop: metrics → regulation → adjustment → new measurement
- [ ] AC-016: Configurable regulation interval (default: 30s)
- [ ] AC-017: Regulation limits (guardrails) — no parameters leave range safe
- [ ] AC-018: Regulation status dashboard (JSON endpoint `/regulation/status`)
- [ ] AC-019: Detailed log of each adjustment for audit

### Guardrails (safe limits)
| Parameter | Min | Max | Default |
|-----------|-----|-----|---------|
| detection_threshold | 0.3 | 0.95 | 0.7 |
| amplification_cap | 2.0 | 10.0 | 5.0 |
| mutation_rate | 0.01 | 0.3 | 0.05 |
| evaporation_rho | 0.01 | 0.5 | 0.1 |
| ant_count | 2 | 20 | 5 |

---

## 3. Definition of Done (DoD)

- [ ] Functional `regulate()` with metrics monitoring
- [ ] Overactivity and underactivity detection
- [ ] Automatic adjustment of 5+ parameters
- [ ] Guardrails implemented and validated
- [ ] Functional feedback loop
- [ ] Endpoint `/regulation/status`
- [ ] Stability Tests
- [ ] Coverage ≥ 80%

---

## 4. Usage Examples

### Regulation cycle

**Input metrics (collected):**
```json
{
  "metrics": {
    "anomaly_detection_rate": 0.35,
    "false_positive_rate": 0.12,
    "amplification_rate": 0.28,
    "mutation_success_rate": 0.08,
    "aco_convergence_cycles": 5,
    "aco_stagnation_detected": false,
    "pheromone_avg": 3.2,
    "pheromone_variance": 0.8
  }
}
```

**Regulation applied:**
```json
{
  "adjustments": [
    {
      "parameter": "detection_threshold",
      "from": 0.7,
      "to": 0.65,
      "reason": "FPR (0.12) above target (0.05); lowering threshold to be more selective",
      "guardrail_check": "WITHIN_BOUNDS"
    },
    {
      "parameter": "mutation_rate",
      "from": 0.05,
      "to": 0.08,
      "reason": "Mutation success rate (0.08) low; increasing exploration",
      "guardrail_check": "WITHIN_BOUNDS"
    },
    {
      "parameter": "evaporation_rho",
      "from": 0.1,
      "to": 0.1,
      "reason": "ACO not stagnated; no adjustment needed",
      "guardrail_check": "NO_CHANGE"
    }
  ],
  "system_status": "BALANCED",
  "regulation_cycle": 42,
  "next_regulation_in_seconds": 30
}
```

### Status endpoint (GET /regulation/status)
```json
{
  "status": "BALANCED",
  "modules": {
    "immune_detection": { "health": "OK", "sensitivity": "NORMAL" },
    "antigen_memory": { "health": "OK", "antigens_active": 23 },
    "response_amplifier": { "health": "OK", "amplification_rate": "NORMAL" },
    "self_classifier": { "health": "OK", "accuracy": 0.93 },
    "mutation_engine": { "health": "TUNING", "rate_adjusted": true },
    "aco_colony": { "health": "OK", "convergence": "STABLE" }
  },
  "guardrails": { "violations": 0, "warnings": 1 },
  "last_regulation": "2026-05-23T14:00:00Z"
}
```

### Hyperactivity detected
```json
{
  "alert": "HYPERACTIVITY_DETECTED",
  "module": "response_amplifier",
  "metric": "amplification_rate",
  "value": 0.45,
  "threshold": 0.30,
  "action": "REDUCE_SENSITIVITY",
  "adjustment": { "amplification_cap": { "from": 5.0, "to": 3.0 } }
}
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Expected Output |
|----|------|----------------|
| UT-001 | High FPR → reduce threshold | threshold decreases |
| UT-002 | Low detection rate → increase sensitivity | threshold decreases |
| UT-003 | Stagnation ACO → increase ρ | evaporation_rho increases |
| UT-004 | Mutation success low → increase rate | mutation_rate increases |
| UT-005 | Guardrail violated | parameter clamped to min |
| UT-006 | Guardrail max violated | parameter clamped to max |
| UT-007 | Balanced system | no adjustments applied |
| UT-008 | Feedback loop 10 cycles | system converges to equilibrium |
| UT-009 | Hyperactivity detected | alert generated + reduced sensitivity |
| UT-010 | Sub-activity detected | increased sensitivity |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | Unavailable metrics | `Ok(skip)` — regulation deferred |
| UF-002 | Negative range | `Err(InvalidInterval)` |
| UF-003 | Inverted guardrail (min > max) | `Err(InvalidGuardrail)` |

### 5.2 Functional Tests

| ID | Scenario | Result |
|----|---------|-----------|
| FT-001 | 100 regulation cycles | System maintains balance |
| FT-002 | Massive anomaly injection | Regulator stabilizes in < 10 cycles |
| FT-003 | Endpoint /regulation/status | Valid JSON with status of all modules |
| FT-004 | Guardrails never breached | 0 violations in 1000 cycles |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Regulation → Detection (FT-017) | Adjusted Threshold |
| IT-002 | Regulation → Amplifier (FT-019) | Adjusted cap |
| IT-003 | Regulation → Mutation (FT-021) | Adjusted rate |
| IT-004 | Regulation → Evaporation (FT-014) | ρ adjusted |
| IT-005 | Regulation → ACO (FT-015) | Ant count adjusted |
| IT-006 | Regulation → API | Endpoint /regulation/functional status |
| IT-007 | Regulation → Metrics | `kce_regulation_adjustments` registered |

---

## 6. CARE Format

**Context:** Without global regulation, AIS and ACO modules can enter vicious cycles: overactivity (everything is anomaly), underactivity (nothing detected), or oscillation (flip-flop between states). Network Regulation implements homeostasis inspired by Jerne's immune network theory — modules regulate each other for systemic balance.

**Assumptions:**
- Metrics from all AIS/ACO modules available via internal API
- Periodic regulation (30s) is sufficient — it does not need to be real-time
- Guardrails are hard limits — never violated
- Feedback loop converges in < 10 cycles in most scenarios
- Overactivity is more dangerous than underactivity (false positives cost more)

**Requirements:**
- R-001: regulate() with global monitoring
- R-002: Detection of overactivity and underactivity
- R-003: Automatic adjustment of 5+ parameters
- R-004: Guardrails with hard limits
- R-005: Convergent feedback loop
- R-006: Endpoint /regulation/status
- R-007: Audit adjustment log
- R-008: Integration with all AIS + ACO modules

**Evidence:**
- `tests/regulation_tests.rs`
- `tests/stability_tests.rs` — feedback loop convergence tests
- `reports/regulation_dynamics.md` — stability analysis

---

## 7. Non-Functional Acceptance Criteria

| Appearance | Target |
|---------|------|
| latency regulate() | < 10ms |
| System overhead | < 1% throughput |
| Feedback loop convergence | < 10 cycles |
| Guardrail violations | 0 (absolute) |
| Additional memory | < 5MB |
| Availability /regulation/status | 100% |

---

## 8. Quality Criteria and Metrics

**Success:**
- 0 guardrail violations in all tests
- Feedback loop converges in < 10 cycles (90%+ of scenarios)
- System maintains balance for 7+ days under load
- Coverage ≥ 80%

**Failure (BLOCKING):**
- Violated guardrail → unstable system → **BLOCKING**
- Feedback loop does not converge in 50 cycles → **BLOCKING**
- Continuous oscillation (flip-flop > 20 cycles) → **BLOCKING**
- Endpoint /regulation/status unavailable → **BLOCKING**

---

## 9. Compatibility and Dependencies

### Internal Dependencies (all regulated modules)
| Module | Regulated Parameters |
|--------|----------------------|
| Detection (FT-017) | detection_threshold |
| Antigen (FT-018) | expiry_days |
| Amplifier (FT-019) | amplification_cap |
| Classifier (FT-020) | nonself_threshold |
| Mutation (FT-021) | mutation_rate |
| Evaporation (FT-014) | evaporation_rho |
| Exploration (FT-015) | ant_count |

### Crates
| Crate | Version | Purpose |
|-------|--------|-----------|
| `tokio` | ^1.35 | Periodic timer |
| `parking_lot` | ^0.12 | RwLock |
| `serde_json` | ^1.0 | Endpoint status |

---

## 10. Traceability

| Type | ID |
|------|----|
| Spec | FT-022-NETWORK-REGULATION |
| Code | `src/ais/regulation.rs`, `src/ais/guardrails.rs` |
| Test | `tests/regulation_tests.rs`, `tests/stability_tests.rs` |
| API | `GET /regulation/status` |
| Deps | FT-014, FT-015, FT-017, FT-018, FT-019, FT-020, FT-021 |

---

## 11. MVP Delivery and Acceptance Criteria — Roadmap

### MVP (Week 1-2)
| Item | Acceptance Criteria |
|------|--------------------|
| `regulate()` basic | Monitors 2 metrics (FPR, detection_rate) |
| Hardcoded guardrails | 5 parameters with limits |
| Adjustment log | Console output |
| 5+ unit tests | Passing |

**Output:** Basic regulation of 2 parameters with guardrails.

---

### Iteration 1 (Week 3-4)
| Item | Acceptance Criteria |
|------|--------------------|
| 5+ metrics monitoring | All AIS modules |
| Automatic adjustment of 5 parameters | Detection, Amplifier, Mutation, Evaporation, Exploration |
| Feedback loop | Convergence in < 10 cycles |
| Over/under activity detection | Alerts generated |
| Endpoint /regulation/status | Functional JSON |

**Output:** Complete regulation with feedback loop and endpoint status.

---

### Iteration 2 (Week 5-6)
| Item | Acceptance Criteria |
|------|--------------------|
| Integration with all 7 modules | End-to-end regulated parameters |
| 7-day stability tests | Balance maintained |
| Dynamic guardrails | Adjustable by config |
| Complete audit trail | Adjustment history |
| Prometheus Metrics | `kce_regulation_*` exported |
| Documentation | API docs + operational runbook |

**Output:** Network Regulation production-ready — autonomous systemic homeostasis.

---

*Document generated on 2026-05-23 — KineContext Engine Product Specification*
