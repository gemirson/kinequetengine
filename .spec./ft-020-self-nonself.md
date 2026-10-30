# FT-020 — Self vs Non-Self Context Classifier

**Module:** AIS (Artificial Immune System) | **Version:** v6.0 | **Priority:** P0 — Critical  
**Artifact ID:** FT-020-SELF-NONSELF | **Update:** 2026-05-23

---

## 1. Context and Objective

The Self vs Non-Self Classifier **separates trustworthy context (self) from external/suspicious context (non-self)** using binary + probabilistic classification. Inspired by the immune system's ability to distinguish its own cells from invaders. Provides adaptive safety without fixed rules — the system learns “self” from normal behavior.

### Value
- Security without fixed rules — learns reliable patterns
- Probabilistic classification (absolute non-binary)
- Continuous adaptation to new legitimate standards

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: `classify(input)` returns `Classification { label: Self|NonSelf, confidence: f64, features: Vec<FeatureScore> }`
- [ ] AC-011: Self profile constructed via negative selection algorithm (NSA)
- [ ] AC-012: Probabilistic classification (confidence 0.0..1.0), not just binary
- [ ] AC-013: Self profile incrementally updated with new valid defaults
- [ ] AC-014: Non-self threshold configurable (default: 0.6)
- [ ] AC-015: Monitored features: vector distribution, temporal pattern, tenant behavior, API usage
- [ ] AC-016: Rating < 2ms per request
- [ ] AC-017: Self profile persisted in KineSQL
- [ ] AC-018: Integration with Immune Detection (FT-017) for cross-validation

---

## 3. Definition of Done (DoD)

- [ ] Negative selection algorithm implemented
- [ ] Self profile with incremental learning
- [ ] Functional probabilistic classification
- [ ] KineSQL Persistence
- [ ] FT-017 Integration
- [ ] Coverage ≥ 80%

---

## 4. Usage Examples

### Self Classification
```json
{
  "input": { "vector": [0.12, 0.85, 0.33], "tenant": "bank_01", "hour": 14, "endpoint": "/query" },
  "result": {
    "label": "SELF",
    "confidence": 0.92,
    "features": [
      { "name": "vector_distribution", "score": 0.95, "status": "NORMAL" },
      { "name": "temporal_pattern", "score": 0.88, "status": "NORMAL" },
      { "name": "tenant_behavior", "score": 0.93, "status": "NORMAL" }
    ]
  }
}
```

### Non-Self Classification
```json
{
  "input": { "vector": [0.99, 0.01, 0.99], "tenant": "bank_01", "hour": 3, "endpoint": "/action" },
  "result": {
    "label": "NON_SELF",
    "confidence": 0.81,
    "features": [
      { "name": "vector_distribution", "score": 0.25, "status": "ANOMALOUS" },
      { "name": "temporal_pattern", "score": 0.35, "status": "ANOMALOUS" },
      { "name": "tenant_behavior", "score": 0.78, "status": "NORMAL" }
    ],
    "recommended_action": "ESCALATE_TO_IMMUNE_DETECTION"
  }
}
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Expected Output |
|----|------|----------------|
| UT-001 | Normal Input → Self | `label: SELF`, confidence > 0.8 |
| UT-002 | Anomalous Input → Non-Self | `label: NON_SELF`, confidence > 0.6 |
| UT-003 | Confidence range | always 0.0..1.0 |
| UT-004 | Self profile update | new valid standard incorporated |
| UT-005 | NSA — generated detectors | detectors don't match self |
| UT-006 | Threshold edge case | score = threshold → configurable |
| UT-007 | Multi-feature scoring | correct feature scores |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | Empty self profile | `Err(NoSelfProfile)` |
| UF-002 | Input with incompatible dims | `Err(DimensionMismatch)` |
| UF-003 | Threshold > 1.0 | `Err(InvalidThreshold)` |

### 5.2 Functional Tests

| ID | Scenario | Result |
|----|---------|-----------|
| FT-001 | 1000 self + 50 non-self | Accuracy > 90% |
| FT-002 | Self profile evolves | New legitimate accepted standards |
| FT-003 | Inline sorting | Overhead < 2ms |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Classifier → Detection (FT-017) | Non-self scaled for detection |
| IT-002 | Classifier → Pipeline | Pipeline adapts flow by classification |
| IT-003 | Classifier → KineSQL | Self profile persists |

---

## 6. CARE Format

**Context:** Fixed security rules are fragile. The Self/Non-Self classifier automatically learns reliable patterns and identifies external/suspicious context probabilistically, continuously adapting.

**Assumptions:** Negative Selection Algorithm is suitable for MVP; self profile built in warm-up (1000+ samples); classification is probabilistic; features are independent; self evolves incrementally.

**Requirements:** R-001: probabilistic classify() | R-002: NSA for self profile | R-003: Incremental learning | R-004: < 2ms | R-005: Persistence | R-006: Multi-feature | R-007: FT-017 Integration.

**Evidence:** `tests/self_nonself_tests.rs` | `reports/classifier_accuracy.md`

---

## 7–11. (Summary)

**Non-functional:** Rating < 2ms | Accuracy > 90% | Self profile memory < 20MB  
**Quality:** Accuracy > 90% | 0 false negatives for known threats | Coverage ≥ 80%  
**Failure (BLOCKING):** Accuracy < 75% | Overhead > 10ms | Self profile corrupts  
**Deps:** Detection (FT-017), KineSQL (FT-005), Pipeline (FT-011)  
**Traceability:** FT-020-SELF-NONSELF | `src/ais/classifier.rs`

**Roadmap:** MVP: classify() with fixed threshold + euclidean distance | Iter1: NSA + multi-feature + incremental | Iter2: Persistence + integration + accuracy tuning

---

*Document generated on 2026-05-23 — KineContext Engine Product Specification*
