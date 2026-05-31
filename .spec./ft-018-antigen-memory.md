# FT-018 — Antigen Memory System

**Module:** AIS (Artificial Immune System) | **Version:** v6.0 | **Priority:** P0 — Critical  
**Artifact ID:** FT-018-ANTIGEN-MEMORY | **Update:** 2026-05-23

---

## 1. Context and Objective

The Antigen Memory System **memorizes critical patterns** (attacks, fraud, rare events) persistently, allowing for faster response in future occurrences. Inspired by the immune system's memory B cells — after first exposure to an antigen, the system "remembers" it and responds exponentially faster upon re-exposure.

### Value
- True continuous learning — system never forgets threats
- Faster response in the future (O(1) lookup vs O(n) detection)
- Anomaly knowledge base grows organically

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: `store_antigen(pattern, metadata)` persists anomalous pattern
- [ ] AC-011: `match_antigen(input)` checks if input is similar to known antigen
- [ ] AC-012: Match uses configurable similarity threshold (default: 0.85)
- [ ] AC-013: Lookup in < 1ms (hash-based or index)
- [ ] AC-014: Antigens persisted in KineSQL — survive restart
- [ ] AC-015: Each antigen has: `pattern`, `severity`, `first_seen`, `last_seen`, `match_count`, `metadata`
- [ ] AC-016: Configurable Antigen Expiry (default: 90 days, 0 = permanent)
- [ ] AC-017: Antigen memory exportable (JSON/CSV) for auditing
- [ ] AC-018: Integration with Immune Detection (FT-017) — anomalies become antigens

---

## 3. Definition of Done (DoD)

- [ ] store/match functional and tested
- [ ] KineSQL Persistence
- [ ] Lookup < 1ms
- [ ] Functional expiration
- [ ] Export JSON/CSV
- [ ] FT-017 Integration
- [ ] Coverage ≥ 80%

---

## 4. Usage Examples

### Antigen storage

**Input (anomaly detected):**
```json
{
  "pattern": {
    "vector_signature": [0.99, 0.01, 0.99, 0.01],
    "frequency_range": [400, 600],
    "ip_class": "tor_exit"
  },
  "severity": "HIGH",
  "metadata": { "source": "immune_detection", "incident_id": "INC-2026-0042" }
}
```

**Antigen created:**
```json
{
  "antigen_id": "ag_001",
  "pattern": { "vector_signature": [0.99, 0.01, 0.99, 0.01], "frequency_range": [400, 600] },
  "severity": "HIGH",
  "first_seen": "2026-05-23T10:00:00Z",
  "last_seen": "2026-05-23T10:00:00Z",
  "match_count": 0,
  "expiry_days": 90,
  "status": "ACTIVE"
}
```

### Known antigen match
**Antigen-like input:**
```json
{ "query_vector": [0.98, 0.02, 0.97, 0.03], "request_frequency": 450 }
```

**Result:**
```json
{
  "antigen_match": true,
  "matched_antigen": "ag_001",
  "similarity": 0.97,
  "severity": "HIGH",
  "response_time_ms": 0.3,
  "action": "BLOCK_IMMEDIATELY",
  "match_count_updated": 1
}
```

### Export (CSV)
```csv
antigen_id,severity,first_seen,last_seen,match_count,status
ag_001,HIGH,2026-05-23T10:00:00Z,2026-05-23T14:30:00Z,12,ACTIVE
ag_002,MEDIUM,2026-05-20T08:00:00Z,2026-05-22T16:00:00Z,3,ACTIVE
ag_003,LOW,2026-03-01T12:00:00Z,2026-03-01T12:00:00Z,0,EXPIRED
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Expected Output |
|----|------|----------------|
| UT-001 | Store antigen | antigen_id generated, persisted |
| UT-002 | Match — similar (0.97) | `antigen_match: true` |
| UT-003 | Match — dissimilar (0.30) | `antigen_match: false` |
| UT-004 | Match count increments | `match_count += 1` |
| UT-005 | Expiry — expired antigen | not returned in match |
| UT-006 | Lookup performance | < 1ms for 1000 antigens |
| UT-007 | Export JSON | valid format |
| UT-008 | Export CSV | valid format |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | Empty pattern | `Err(EmptyPattern)` |
| UF-002 | Invalid Severity | `Err(InvalidSeverity)` |
| UF-003 | Exact duplicate antigen | Update existing (idempotent) |

### 5.2 Functional Tests

| ID | Scenario | Result |
|----|---------|-----------|
| FT-001 | Store 100 antigens → match | Everyone matches correctly |
| FT-002 | Antigen survives restart | Persists via KineSQL |
| FT-003 | Expiry after 90 days (simulated) | Antigen removed from matches |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Detection (FT-017) → Antigen Memory | Anomaly creates antigen automatically |
| IT-002 | Antigen → Response Amplifier (FT-019) | Antigen match amplifies response |
| IT-003 | Antigen → KineSQL | Functional persistence |

---

## 6. CARE Format

**Context:** Anomaly detection is expensive (O(n)). Memorizing critical patterns allows O(1) response on re-exposure. Biological memory B cells inspire long-term persistence to known threats.

**Assumptions:** Antigens are vectors with similarity match; 1000 antigens is a practical ceiling for MVP; hash-based lookup for < 1ms; KineSQL persistence; expiry configurable.

**Requirements:** R-001: store/match | R-002: Persistence | R-003: < 1ms lookup | R-004: Expiry | R-005: Export JSON/CSV | R-006: Integration FT-017 | R-007: Idempotence.

**Evidence:** `tests/antigen_memory_tests.rs` | `reports/antigen_response_time.md`

---

## 7–11. (Summary)

**Non-functional:** Lookup < 1ms | Storage < 10MB for 1000 antigens | Expiry accuracy ± 1 hour  
**Quality:** Match accuracy > 95% | 0 false negatives for exact match | Coverage ≥ 80%  
**Failure (BLOCKING):** Lookup > 10ms | Antigen lost after restart | False negative > 5%  
**Deps:** KineSQL (FT-005), Immune Detection (FT-017), Response Amplifier (FT-019)  
**Traceability:** FT-018-ANTIGEN-MEMORY | `src/ais/antigen.rs`

**Roadmap:** MVP: store/match with hash lookup | Iter1: KineSQL persistence + expiry + CSV export | Iter2: Similarity match + FT-017 integration + audit trail

---

*Document generated on 2026-05-23 — KineContext Engine Product Specification*
