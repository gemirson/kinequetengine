# FT-008 — Security (Auth + Multi-Tenant + Validation)

**Module:** Security | **Version:** v5.9 | **Priority:** P0 — Critical  
**Artifact ID:** FT-008-SECURITY | **Update:** 2026-05-23

---

## 1. Context and Objective

Mandatory basic security for production: API Key authentication, multi-tenant isolation and input validation. If the system will serve banking data (e.g. credit), isolation per tenant is mandatory. Without this, it is not a product.

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: API Key mandatory on all endpoints (except /health)
- [ ] AC-011: Request without key → 401 Unauthorized
- [ ] AC-012: Request with invalid key → 401 Unauthorized
- [ ] AC-013: Multi-tenant isolation: `tenant_id` mandatory in requests
- [ ] AC-014: Data filtered by `tenant_id` — tenant A does not access data from B
- [ ] AC-015: Input sanitized — rejects malformed payloads
- [ ] AC-016: Rate limiting per tenant (configurable)

---

## 3. Definition of Done (DoD)

- [ ] Implemented and active Auth middleware
- [ ] Tenant isolation on all queries
- [ ] Input validation on all endpoints
- [ ] Bypass tests (negative) passing
- [ ] Functional rate limiting

---

## 4. Usage Examples

### Authenticated Request
```bash
curl -H "x-api-key: tk_prod_abc123" -H "x-tenant-id: tenant_42" ...
```

### Tenant isolation
**Tenant A inserts:**
```json
{ "tenant_id": "A", "data": {"id": 1, "vector": [0.5, 0.5]} }
```
**Tenant B search:**
```json
{ "tenant_id": "B", "query_vector": [0.5, 0.5], "top_k": 10 }
```
**Result:** `results: []` — tenant B does not see data from A.

### No API Key
```json
{ "error": "UNAUTHORIZED", "message": "Missing x-api-key header", "code": 401 }
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Expected Output |
|----|------|----------------|
| UT-001 | Request with valid API Key | pass auth |
| UT-002 | Request without API Key | 401 |
| UT-003 | Request with invalid key | 401 |
| UT-004 | Tenant isolation | tenant A ≠ tenant B |
| UT-005 | Rate limit exceeded | 429 Too Many Requests |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | SQL injection into input | Rejected with 400 |
| UF-002 | Empty Tenant ID | 400 Bad Request |
| UF-003 | Tenant A accesses B | `results: []` or 403 |

### 5.2 Functional Tests

| ID | Scenario | Result |
|----|---------|-----------|
| FT-001 | Complete authentication flow | 401 → add key → 200 |
| FT-002 | Cross-tenant access | 0 results returned |
| FT-003 | Rate limiting | 429 after exceeding limit |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Auth → API → Pipeline | Pipeline only runs authenticated |
| IT-002 | Tenant → KineSQL | Data filtered in storage |

---

## 6. CARE Format

**Context:** Security for a system that can serve financial data; API Key + tenant isolation + validation are minimum requirements for production.

**Assumptions:** API Key is sufficient for MVP (future OAuth2); tenant_id is opaque string; rate limiting in-memory (Redis in future iteration).

**Requirements:** R-001: Auth API Key | R-002: Tenant isolation | R-003: Input validation | R-004: Rate limiting | R-005: 401/403/429 correct.

**Evidence:** `tests/security_tests.rs`

---

## 7–11. (Summary)

**Non-functional:** Auth overhead < 1ms | 0 possible bypasses | Configurable rate limit per tenant  
**Quality:** 0 cross-tenant accesses | 100% requests without key → 401 | Coverage ≥ 80%  
**Deps:** `tower` ^0.4 (middleware), `parking_lot` ^0.12  
**Traceability:** FT-008-SECURITY | `src/api/middleware.rs`, `src/security/tenant.rs`

**Roadmap:** MVP: API Key + validation | Iter1: tenant isolation + rate limiting | Iter2: OAuth2 + audit log
