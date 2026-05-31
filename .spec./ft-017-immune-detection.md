# FT-017 — Immune Detection Engine (Anomaly Recognition)

**Module:** AIS (Artificial Immune System) | **Version:** v6.0 | **Priority:** P0 — Critical  
**Artifact ID:** FT-017-IMMUNE-DETECTION | **Update:** 2026-05-23

---

## 1. Context and Objective

The Immune Detection Engine implements **anomaly detection** inspired by the biological immune system. Compares input patterns with a baseline of "normal" behavior (self). Patterns that diverge significantly are classified as "antigens" (anomalies). Provides natural anti-fraud and non-standard behavior detection without hard-coded fixed rules.

### Value
- Natural anti-fraud — detects anomalous patterns without hardcoded rules
- Real-time non-standard behavior detection
- Continuous learning — baseline evolves over time

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: `detect(input)` returns `AnomalyResult { is_anomaly: bool, score: f64, details: Vec<Deviation> }`
- [ ] AC-011: Baseline built from N first interactions (configurable warm-up)
- [ ] AC-012: Anomaly score = normalized distance from baseline (0.0 = normal, 1.0 = anomalous)
- [ ] AC-013: Configurable detection threshold (default: 0.7)
- [ ] AC-014: Multiple dimensions monitored: latency, feature distribution, frequency, temporal pattern
- [ ] AC-015: False positive rate < 5% after warm-up
- [ ] AC-016: Real-time detection (< 2ms overhead per request)
- [ ] AC-017: Baseline updated incrementally (sliding window)
- [ ] AC-018: Anomalies generate event for Antigen Memory (FT-018)

---

## 3. Definition of Done (DoD)

- [ ] Implemented detection algorithm (distance-based or negative selection)
- [ ] Baseline with warm-up and sliding window
- [ ] Configurable Threshold
- [ ] Pipeline integration for inline detection
- [ ] Validated false positive rate < 5%
- [ ] Unit tests ≥ 80%

---

## 4. Usage Examples

### Anomaly detection

**Normal input:**
```json
{
  "request": {
    "query_vector": [0.12, 0.85, 0.33, 0.67],
    "tenant_id": "bank_01",
    "source_ip": "10.0.1.50",
    "request_frequency": 12
  }
}
```

**Result (normal):**
```json
{
  "is_anomaly": false,
  "anomaly_score": 0.15,
  "threshold": 0.7,
  "deviations": []
}
```

**Anomalous input (potential fraud):**
```json
{
  "request": {
    "query_vector": [0.99, 0.01, 0.99, 0.01],
    "tenant_id": "bank_01",
    "source_ip": "185.220.101.1",
    "request_frequency": 500
  }
}
```

**Result (anomaly detected):**
```json
{
  "is_anomaly": true,
  "anomaly_score": 0.89,
  "threshold": 0.7,
  "deviations": [
    { "dimension": "query_vector", "deviation": 0.82, "detail": "Distribution shift detected" },
    { "dimension": "request_frequency", "deviation": 0.95, "detail": "41x above baseline mean" },
    { "dimension": "source_ip", "deviation": 0.70, "detail": "New IP, known Tor exit node" }
  ],
  "action": "FLAG_FOR_REVIEW",
  "antigen_created": true
}
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Expected Output |
|----|------|----------------|
| UT-001 | Normal input | `is_anomaly: false`, score < 0.7 |
| UT-002 | Anomalous input (high frequency) | `is_anomaly: true`, score > 0.7 |
| UT-003 | Anomalous input (distant vector) | `is_anomaly: true` |
| UT-004 | Warm-up period | Detection disabled during warm-up |
| UT-005 | Baseline sliding window | Baseline updates with new data |
| UT-006 | Threshold edge case | score = threshold → configurable behavior |
| UT-007 | Multiple dimensions | Correct dimension deviations |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | Empty baseline (pre warm-up) | `Ok(skip)` — no detection |
| UF-002 | Threshold > 1.0 | `Err(InvalidThreshold)` |
| UF-003 | Input with NaN | `Err(InvalidInput)` |

### 5.2 Functional Tests

| ID | Scenario | Result |
|----|---------|-----------|
| FT-001 | 1000 normal requests → 10 anomalous | 10 detected, FPR < 5% |
| FT-002 | Inline detection in the pipeline | Overhead < 2ms |
| FT-003 | Baseline evolves with 10k requests | FPR decreases over time |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Detection → Antigen Memory (FT-018) | Anomaly creates antigen |
| IT-002 | Detection → Pipeline | Pipeline flagga anomalous request |
| IT-003 | Detection → Metrics | `kce_anomalies_detected` registered |
| IT-004 | Detection → Response Amplifier (FT-019) | Anomaly amplifies priority |

---

## 6. CARE Format

**Context:** Financial production systems need to detect fraud and anomalous behavior in real time. Fixed rules are fragile and easily circumvented. Immune detection compares with learned baseline, continually adapting.

**Assumptions:** Baseline is built with warm-up of 1000+ requests; sliding window of 10k requests; anomaly score is normalized distance; multiple dimensions are monitored independently; target false positive rate < 5%.

**Requirements:** R-001: detect() inline | R-002: Baseline sliding window | R-003: Multi-dimensional | R-004: Configurable Threshold | R-005: FPR < 5% | R-006: < 2ms overhead | R-007: Antigen Memory Integration.

**Evidence:** `tests/immune_detection_tests.rs` | `reports/fpr_analysis.md`

---

## 7–11. (Summary)

**Non-functional:** Overhead < 2ms | FPR < 5% | True Positive Rate > 90% | Baseline memory < 50MB  
**Quality:** FPR < 5% | TPR > 90% | Stable baseline after warm-up | Coverage ≥ 80%  
**Failure (BLOCKING):** FPR > 10% | TPR < 70% | Overhead > 10ms | Baseline corrupts  
**Deps:** Pipeline (FT-011), Antigen Memory (FT-018), Metrics (FT-007)  
**Traceability:** FT-017-IMMUNE-DETECTION | `src/ais/detection.rs`

**Roadmap:** MVP: detect() with Euclidean distance + fixed threshold | Iter1: Multi-dimensional + sliding window + baseline auto | Iter2: Antigen Memory integration + inline pipeline + FPR tuning

---

*Document generated on 2026-05-23 — KineContext Engine Product Specification*
