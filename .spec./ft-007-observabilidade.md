# FT-007 — Observabilidade (Tracing + Métricas)

**Módulo:** Operação | **Versão:** v5.9 | **Prioridade:** P1 — Alto  
**Artefato ID:** FT-007-OBSERVABILITY | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

Observabilidade é **não negociável** para produção. Sem ela, o sistema degrada silenciosamente. Este módulo implementa tracing estruturado, métricas Prometheus-ready e logs com request_id para correlação. Permite responder: qual latência p95? qual erro por minuto? qual recall?

---

## 2. Critérios de Aceite (AC)

- [ ] AC-010: Tracing ativo com `tracing` crate — logs estruturados
- [ ] AC-011: Logs incluem `request_id` para correlação
- [ ] AC-012: Métricas exportadas via endpoint `/metrics` (Prometheus format)
- [ ] AC-013: Métricas essenciais: `kce_query_latency_ms`, `kce_retrieval_hits`, `kce_mrna_executions`, `kce_error_rate`
- [ ] AC-014: Níveis de log: `info`, `warn`, `error` usados corretamente
- [ ] AC-015: Métricas incluem labels (endpoint, status, tenant)

---

## 3. Definition of Done (DoD)

- [ ] `tracing_subscriber::fmt::init()` configurado
- [ ] Logs estruturados em todos os módulos
- [ ] `/metrics` endpoint ativo com formato Prometheus
- [ ] 4+ métricas essenciais exportadas
- [ ] Request_id propagado em todo pipeline
- [ ] Testes de validação de export

---

## 4. Exemplos de Uso

### Log estruturado
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

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Saída Esperada |
|----|------|----------------|
| UT-001 | Log contém request_id | campo presente |
| UT-002 | Métrica incrementa | counter += 1 |
| UT-003 | Histogram registra latência | bucket correto |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado |
|----|---------|-----------|
| FT-001 | GET /metrics retorna Prometheus | 200, text/plain, formato válido |
| FT-002 | Query gera log com request_id | log correlacionado visível |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | API → Pipeline → Logs | request_id propagado end-to-end |
| IT-002 | Métricas → Prometheus scrape | Prometheus consegue coletar |

---

## 6. Formato CARE

**Context:** Observabilidade para produção: sem ela, degradação silenciosa é inevitável. Tracing + métricas + logs correlacionados.

**Assumptions:** Prometheus disponível para scrape; tracing_subscriber é suficiente (sem Jaeger/OTLP no MVP); logs são stdout (container-friendly).

**Requirements:** R-001: Tracing estruturado | R-002: /metrics Prometheus | R-003: 4+ métricas | R-004: request_id | R-005: Labels por endpoint.

**Evidence:** `/metrics` endpoint | logs stdout

---

## 7–11. (Resumo)

**Não funcionais:** /metrics < 10ms | Log overhead < 1% latência | Retenção: stdout → log aggregator externo  
**Qualidade:** 4+ métricas ativas | request_id em 100% dos logs | Cobertura ≥ 80%  
**Deps:** `tracing` ^0.1, `tracing-subscriber` ^0.3, `metrics` ^0.22, `metrics-exporter-prometheus` ^0.13  
**Rastreabilidade:** FT-007-OBSERVABILITY | `src/observability/mod.rs`

**Roadmap:** MVP: tracing básico + /metrics | Iter1: request_id + labels | Iter2: dashboards + alertas
