# FT-003 — ECMA (Embryological Cognitive Memory Architecture)

**Module:** Core Engine | **Version:** v5.9 | **Priority:** P0 — Critical  
**Artifact ID:** FT-003-ECMA-ENGINE | **Updated:** 2026-05-23

---

## 1. Context and Objective

ECMA is the **cognitive evolution** engine of the KCE. Knowledge nodes evolve through states (Stem → Progenitor → Specialized → Apoptosis) based on usage, entropy, and connections. It implements a biologically-inspired lifecycle that ensures relevant knowledge matures and obsolete knowledge is automatically degraded.

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: States implemented: `Stem`, `Progenitor`, `Specialized`, `Apoptosis`
- [ ] AC-011: Automatic transitions based on `usage_count`, `entropy`, `connections`
- [ ] AC-012: Nodes evolve to `Specialized` after 10+ interactions with high relevance
- [ ] AC-013: Nodes with low relevance (entropy > threshold) degrade to `Apoptosis`
- [ ] AC-014: No invalid transitions occur (e.g., Apoptosis → Stem)
- [ ] AC-015: Function `maturity(node)` returns score 0.0..1.0
- [ ] AC-016: Feedback loop: MCE execution result feeds back into maturity
- [ ] AC-017: Node state is persisted in KineSQL

---

## 3. Definition of Done (DoD)

- [ ] All 4 states implemented
- [ ] State machine with validated transitions
- [ ] Maturity function active and tested
- [ ] MCE → ECMA feedback loop functional
- [ ] State persistence in KineSQL
- [ ] Unit tests ≥ 80% coverage
- [ ] No invalid transitions possible

---

## 4. Usage Examples

### Node evolution
**Input (initial state):**
```json
{
  "node_id": 42,
  "state": "Stem",
  "usage_count": 0,
  "entropy": 0.9,
  "connections": 2,
  "maturity": 0.1
}
```

**After 15 interactions with high relevance:**
```json
{
  "node_id": 42,
  "state": "Specialized",
  "usage_count": 15,
  "entropy": 0.2,
  "connections": 8,
  "maturity": 0.87
}
```

### Degradation (Apoptosis)
```json
{
  "node_id": 99,
  "state": "Apoptosis",
  "usage_count": 2,
  "entropy": 0.95,
  "connections": 0,
  "maturity": 0.05,
  "reason": "HIGH_ENTROPY_LOW_USAGE"
}
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Input | Expected Output |
|----|------|---------|----------------|
| UT-001 | Maturity > 0.5 for active node | usage=10, entropy=0.3 | `maturity > 0.5` |
| UT-002 | Stem → Progenitor transition | usage=5, entropy=0.5 | `state == Progenitor` |
| UT-003 | Transition → Specialized | usage=15, entropy=0.2 | `state == Specialized` |
| UT-004 | Degradation → Apoptosis | usage=1, entropy=0.95 | `state == Apoptosis` |
| UT-005 | Invalid transition blocked | Apoptosis → Stem | `Err(InvalidTransition)` |
| UT-006 | Maturity range | any node | `0.0 <= maturity <= 1.0` |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | Apoptosis → Stem transition | `Err(InvalidTransition)` |
| UF-002 | Negative entropy | `Err(InvalidEntropy)` |
| UF-003 | Non-existent node for update | `Err(NodeNotFound)` |

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | Simulate 20 interactions | Node evolves Stem → Progenitor → Specialized |
| FT-002 | Simulate abandonment | Node degrades to Apoptosis |
| FT-003 | MCE feedback loop | Positive result increases maturity |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Retrieval → ECMA | Returned nodes have updated maturity |
| IT-002 | ECMA → KineSQL | States persist after restart |
| IT-003 | MCE → ECMA | Execution feedback feeds back |

---

## 6. CARE Format

**Context:** Cognitive evolution engine that implements a biological lifecycle for knowledge nodes; ensures relevant information matures and obsolete information is removed.

**Assumptions:** Transition threshold is configurable; entropy is calculated externally; usage_count is incremented by the Retrieval Engine; persistence via KineSQL.

**Requirements:** R-001: 4 states | R-002: automatic transitions | R-003: maturity 0..1 | R-004: MCE feedback loop | R-005: persistence | R-006: no invalid transitions.

**Evidence:** `tests/ecma_tests.rs` | `src/ecma/state_machine.rs`

---

## 7. Non-Functional Criteria

| Aspect | Target |
|---------|------|
| Update latency | < 1ms per node |
| Memory per node | < 256 bytes |
| Transitions/s | ≥ 10k |

---

## 8. Quality and Metrics

**Success:** 0 invalid transitions | Maturity always in [0,1] | Coverage ≥ 80%  
**Failure (BLOCKING):** Invalid transition occurs | Maturity out of range | Corrupted state after restart

---

## 9. Compatibility and Dependencies

**Internal deps:** Retrieval (usage_count), Graph (connections), MCE (feedback), KineSQL (persistence)  
**Rust:** ≥ 1.75

---

## 10. Traceability

| Type | ID |
|------|----|
| Spec | FT-003-ECMA-ENGINE |
| Code | `src/ecma/mod.rs`, `src/ecma/state_machine.rs` |
| Test | `tests/ecma_integration.rs` |

---

## 11. MVP Roadmap

### MVP (Week 1-2)
4 states | Manual transitions | `maturity()` | 5+ unit tests

### Iteration 1 (Week 3-4)
Automatic transitions by threshold | MCE feedback loop | Transition validation | Benchmark

### Iteration 2 (Week 5-6)
KineSQL persistence | Full pipeline integration | Load tests | Docs
