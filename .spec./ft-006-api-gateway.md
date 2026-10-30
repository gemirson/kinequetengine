# FT-006 — API Gateway (Axum)

**Module:** Interface | **Version:** v5.9 | **Priority:** P0 — Critical  
**Artifact ID:** FT-006-API-GATEWAY | **Updated:** 2026-05-23

---

## 1. Context and Objective

The API is the **external interface** of the KCE. Built on Axum, it exposes REST endpoints with an OpenAPI contract, API Key authentication, input validation, timeout, and consistent JSON responses. It is the entry point for clients, frontends, and integrations.

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: Endpoints: `POST /query`, `POST /encode`, `POST /action`, `GET /health`, `GET /metrics`
- [ ] AC-011: OpenAPI/Swagger UI available via `utoipa`
- [ ] AC-012: API Key authentication (`x-api-key` header) — 401 without key
- [ ] AC-013: Configurable timeout per request (default: 100ms)
- [ ] AC-014: Input validation — rejects invalid payloads with 400
- [ ] AC-015: All responses are valid JSON with correct status codes
- [ ] AC-016: p95 < 100ms
- [ ] AC-017: Error rate < 1% under normal load
- [ ] AC-018: Request ID in all responses for traceability
- [ ] AC-019: Configurable CORS

---

## 3. Definition of Done (DoD)

- [ ] 5 endpoints implemented and tested
- [ ] Automatically generated OpenAPI spec
- [ ] Functional auth middleware
- [ ] Timeout implemented
- [ ] Active input validation
- [ ] Request ID in all responses
- [ ] Unit + functional tests ≥ 80%

---

## 4. Usage Examples

### POST /query
**Request:**
```bash
curl -X POST http://localhost:3000/query \
  -H "x-api-key: secret" \
  -H "Content-Type: application/json" \
  -d '{"query_vector": [0.12, 0.85, 0.33], "top_k": 5}'
```

**Response (200):**
```json
{
  "request_id": "req_a1b2c3",
  "results": [
    { "id": 42, "score": 0.95 }
  ],
  "latency_ms": 12.4
}
```

### Without API Key (401)
```json
{ "error": "UNAUTHORIZED", "message": "Missing or invalid x-api-key", "code": 401 }
```

### Invalid payload (400)
```json
{ "error": "VALIDATION_ERROR", "message": "query_vector is required", "code": 400 }
```

### GET /health (200)
```json
{
  "status": "healthy",
  "components": {
    "wal": true,
    "db": true,
    "latency_ok": true
  },
  "uptime_seconds": 86400
}
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Expected Output |
|----|------|----------------|
| UT-001 | Valid payload validation | `Ok(parsed)` |
| UT-002 | Empty payload validation | `Err(ValidationError)` |
| UT-003 | Auth with valid key | request passes |
| UT-004 | Auth without key | 401 |
| UT-005 | Auth with wrong key | 401 |
| UT-006 | Generated Request ID | `x-request-id` header present |

**Expected failures:**

| ID | Case | Behavior |
|----|------|---------------|
| UF-001 | Non-JSON body | 400 Bad Request |
| UF-002 | Timeout exceeded | 408 Request Timeout |
| UF-003 | Non-existent endpoint | 404 Not Found |

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | Real HTTP call /query | 200 with valid JSON |
| FT-002 | /health call | 200 with components status |
| FT-003 | 100 concurrent requests | All return without error |
| FT-004 | Accessible Swagger UI | /swagger-ui renders |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | API → Retrieval → Response | Full pipeline via HTTP |
| IT-002 | API → MCE | /encode + /action functional |
| IT-003 | API → Observability | Request generates log + metric |

---

## 6. CARE Format

**Context:** KCE REST interface on Axum; entry point for all clients, with OpenAPI contract, authentication, and validation.

**Assumptions:** Axum 0.7+; API Key is sufficient for MVP (OAuth2 in future iteration); global configurable timeout; all responses are JSON.

**Requirements:** R-001: 5 endpoints | R-002: OpenAPI | R-003: Auth API Key | R-004: Timeout | R-005: Validation | R-006: p95 < 100ms | R-007: Request ID.

**Evidence:** `tests/api_tests.rs` | OpenAPI spec at `/swagger-ui`

---

## 7. Non-Functional Criteria

| Aspect | Target |
|---------|------|
| p95 latency | < 100ms |
| Error rate | < 1% |
| Concurrency | 1000 req/s sustained |
| Startup time | < 2s |

### Security
- Mandatory API Key on all endpoints except /health
- Input sanitized against injection
- Security headers (X-Content-Type-Options, X-Frame-Options)

---

## 8. Quality and Metrics

**Success:** p95 < 100ms | Error < 1% | Functional Swagger | Coverage ≥ 80%  
**Failure (BLOCKING):** p95 > 200ms | Error > 5% | Auth bypass possible

---

## 9. Compatibility and Dependencies

| Crate | Version | Purpose |
|-------|--------|-----------|
| `axum` | ^0.7 | Web framework |
| `utoipa` | ^4 | OpenAPI |
| `utoipa-swagger-ui` | ^6 | Swagger UI |
| `tokio` | ^1.35 | Async runtime |
| `tower` | ^0.4 | Middleware |

---

## 10. Traceability

| Type | ID |
|------|----|
| Spec | FT-006-API-GATEWAY |
| Code | `src/api/mod.rs`, `src/api/routes.rs`, `src/api/middleware.rs` |
| Test | `tests/api_integration.rs` |

---

## 11. MVP Roadmap

### MVP (Week 1-2)
POST /query + GET /health | JSON responses | Basic validation | 5+ tests

### Iteration 1 (Week 3-4)
Auth middleware | POST /encode + /action | GET /metrics | OpenAPI/Swagger | Timeout | Request ID

### Iteration 2 (Week 5-6)
CORS | Rate limiting | 1000 req/s load test | Full integration | Docs
