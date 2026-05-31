# FT-007 — Observability (Tracing + Metrics)

**Module:** Operation | **Version:** v5.9 | **Priority:** P1 — High  
**Artifact ID:** FT-007-OBSERVABILITY | **Update:** 2026-05-23

---

## 1. Context and Objective

Observability is **non-negotiable** for production. Without it, the system silently degrades. This module implements structured tracing, Prometheus-ready metrics and logs with request_id for correlation. Allows you to answer: what latency is p95? what error per minute? which recall?

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: Active tracing with `tracing` crate — structured logs
- [ ] AC-011: Logs include `request_id` for correlation
- [ ] AC-012: Metrics exported via `/metrics` endpoint (Prometheus format)
- [ ] AC-013: Essential metrics: `kce_query_latency_ms`, `kce_retrieval_hits`, `kce_mrna_executions`, `kce_error_rate`
- [ ] AC-014: Log levels: `info`, `warn`, `error` used correctly
- [ ] AC-015: Metrics include labels (endpoint, status, tenant)

---

## 3. Definition of Done (DoD)

- [ ] `tracing_subscriber::fmt::init()` configured
- [ ] Structured logs across all modules
- [ ] `/metrics` active endpoint with Prometheus format
- [ ] 4+ essential metrics exported
- [ ] Request_id propagated throughout pipeline
- [ ] Export validation tests

---

## 4. Usage Examples

### Structured log
```
2026-05-23T10:00:00Z INFO kce::api request_id=req_a1b2c3 query_received endpoint="/query"
2026-05-23T10:00:00Z INFO kce::retrieval request_id=req_a1b2c3 candidates=42 latency_ms=12.4
2026-05-23T10:00:00Z WARN kce::retrieval request_id=req_a1b2c3 low_similarity threshold=0.3
```

### Endpoint /metrics (Prometheus)
```
# HELP kce_query_latency_ms Query latency in milliseconds
# TYPE kce_query_latency_ms histogram
kce_query_latency_ms_bucket{le="10"} 450
kce_query_latency_ms_bucket{le="50"} 980
kce_query_latency_ms_bucket{le="100"} 998
kce_query_latency_ms_count 1000
kce_query_latency_ms_sum 15234

kce_error_rate 0.002
kce_retrieval_hits 45023
kce_mrna_executions 44890
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Expected Output |
|----|------|----------------|
| UT-001 | Log contains request_id | present field |
| UT-002 | Metric increments | counter += 1 |
| UT-003 | Histogram records latency | correct bucket |

### 5.2 Functional Tests

| ID | Scenario | Result |
|----|---------|-----------|
| FT-001 | GET /metrics returns Prometheus | 200, text/plain, valid format |
| FT-002 | Query generates log with request_id | visible correlated log |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | API → Pipeline → Logs | request_id propagated end-to-end |
| IT-002 | Metrics → Prometheus scrape | Prometheus manages to collect |

---

## 6. CARE Format

**Context:** Observability for production: without it, silent degradation is inevitable. Tracing + metrics + correlated logs.

**Assumptions:** Prometheus available for scrape; tracing_subscriber is sufficient (without Jaeger/OTLP in MVP); logs are stdout (container-friendly).

**Requirements:** R-001: Structured tracing | R-002: /metrics Prometheus | R-003: 4+ metrics | R-004: request_id | R-005: Labels per endpoint.

**Evidence:** `/metrics` endpoint | stdout logs

---

## 7–11. (Summary)

**Non-functional:** /metrics < 10ms | Log overhead < 1% latency | Retention: stdout → external log aggregator  
**Quality:** 4+ active metrics | request_id in 100% of logs | Coverage ≥ 80%  
**Deps:** `tracing` ^0.1, `tracing-subscriber` ^0.3, `metrics` ^0.22, `metrics-exporter-prometheus` ^0.13  
**Traceability:** FT-007-OBSERVABILITY | `src/observability/mod.rs`

**Roadmap:** MVP: basic tracing + /metrics | Iter1: request_id + labels | Iter2: dashboards + alerts
