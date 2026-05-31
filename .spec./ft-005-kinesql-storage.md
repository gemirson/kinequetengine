# FT-005 — KineSQL (Storage Engine)

**Module:** Storage | **Version:** v5.9 | **Priority:** P0 — Critical  
**Artifact ID:** FT-005-KINESQL-STORAGE | **Updated:** 2026-05-23

---

## 1. Context and Objective

KineSQL is the **embedded storage engine** of the KCE. It provides reliable persistence with real WAL (Write-Ahead Log), paging, mmap, and per-page checksums. It is the foundation of reliability: if KineSQL fails, the entire system loses data. It must survive crashes without corruption.

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: Persistent WAL on disk (not just memory)
- [ ] AC-011: `fsync` active in all critical writes (WAL + data)
- [ ] AC-012: Atomic commit: WAL → fsync WAL → apply data → fsync data
- [ ] AC-013: Functional recovery: WAL replay after crash without loss
- [ ] AC-014: CRC32 checksum per page — detects corruption
- [ ] AC-015: Paging implemented with configurable size
- [ ] AC-016: Consistent reading after restart
- [ ] AC-017: Support for mmap for fast reading
- [ ] AC-018: Concurrency via `Arc<RwLock<KineSQL>>`

---

## 3. Definition of Done (DoD)

- [ ] Persistent WAL with fsync
- [ ] Functional recovery tested with simulated crash
- [ ] CRC32 checksum validated per page
- [ ] Paging implemented
- [ ] Atomic commit implemented
- [ ] Crash/recovery tests passing
- [ ] Coverage ≥ 80%

---

## 4. Usage Examples

### Write + Recovery

**Write operation:**
```json
{
  "operation": "INSERT",
  "table": "vectors",
  "data": { "id": 42, "vector": [0.12, 0.85, 0.33, 0.67], "metadata": {"label": "credit_risk"} }
}
```

**WAL entry (internal format):**
```
1716500000|INSERT|vectors|{"id":42,"vector":[0.12,0.85,0.33,0.67]}|CRC:0xA3F2B1C4
```

**After crash + recovery:**
```json
{
  "recovery": {
    "wal_entries_replayed": 47,
    "pages_recovered": 12,
    "checksum_failures": 0,
    "data_integrity": "VALID"
  }
}
```

### Error — invalid checksum
```json
{
  "error": "CHECKSUM_FAILURE",
  "message": "Page 7 CRC mismatch: expected 0xA3F2B1C4, got 0x00000000",
  "code": 500,
  "action": "PAGE_MARKED_CORRUPT"
}
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Input | Expected Output |
|----|------|---------|----------------|
| UT-001 | Valid checksum | page with data | `checksum == expected` |
| UT-002 | Invalid checksum | corrupted page | `Err(ChecksumFailure)` |
| UT-003 | WAL append | INSERT operation | entry in WAL file |
| UT-004 | WAL replay | WAL file with 10 entries | 10 operations applied |
| UT-005 | Page full → new page | insertion exceeds size | new page allocated |
| UT-006 | Read by ID | `get(42)` | correct record |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | Corrupted WAL | `Err(WalCorrupted)` with error position |
| UF-002 | Disk full | `Err(DiskFull)` — no partial corruption |
| UF-003 | Non-existent record | `Err(NotFound)` |

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | Insert → kill → restart → read | Intact data after recovery |
| FT-002 | 10k inserts → checksum all pages | All CRC32 valid |
| FT-003 | Concurrent reads + writes | No corruption, consistent reads |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | KineSQL → Retrieval | Vectors read correctly from storage |
| IT-002 | KineSQL → Graph | Graph persists and restores |
| IT-003 | KineSQL → ECMA | Node states persist |

---

## 6. CARE Format

**Context:** Embedded storage engine that is the foundation of KCE reliability; must survive crashes without data loss/corruption using WAL + fsync + checksum.

**Assumptions:** Filesystem supports fsync; disk has enough space; WAL is single-writer (write lock); mmap available on target OS; CRC32 is sufficient for corruption detection (not for cryptographic security).

**Requirements:** R-001: Persistent WAL | R-002: fsync in all writes | R-003: Atomic commit | R-004: Lossless recovery | R-005: CRC32 per page | R-006: Paging | R-007: mmap read.

**Evidence:** `tests/kinesql_crash_tests.rs` | `tests/kinesql_recovery.rs`

---

## 7. Non-Functional Criteria

| Aspect | Target |
|---------|------|
| Write latency (fsync) | < 10ms |
| Read latency (mmap) | < 1ms |
| Recovery time (1k entries) | < 500ms |
| Max data size | ≥ 10GB |
| Durability | 0 loss in simulated crash |

---

## 8. Quality and Metrics

**Success:** 0 loss on crash | 100% valid CRC32 | Automatic recovery | Coverage ≥ 80%  
**Failure (BLOCKING):** Data loss on crash | Silent corruption | Recovery fails

---

## 9. Compatibility and Dependencies

| Crate | Version | Purpose |
|-------|--------|-----------|
| `crc32fast` | ^1.3 | Checksum |
| `parking_lot` | ^0.12 | RwLock |
| `memmap2` | ^0.9 | Memory-mapped I/O |

**OS:** Linux (prod) — fsync guaranteed | **Filesystem:** ext4, xfs (recommended)

---

## 10. Traceability

| Type | ID |
|------|----|
| Spec | FT-005-KINESQL-STORAGE |
| Code | `src/storage/mod.rs`, `src/storage/wal.rs`, `src/storage/page.rs` |
| Test | `tests/kinesql_crash_tests.rs` |

---

## 11. MVP Roadmap

### MVP (Week 1-2)
WAL on disk (append) | Basic read/write | CRC32 Checksum | 5+ unit tests

### Iteration 1 (Week 3-4)
Real fsync | Atomic commit | Recovery (WAL replay) | Paging | Crash tests

### Iteration 2 (Week 5-6)
mmap for reading | RwLock concurrency | Integration with all modules | 10k ops stress test | Docs
