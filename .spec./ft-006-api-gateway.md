# FT-006 — API Gateway (Axum)

**Módulo:** Interface | **Versão:** v5.9 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-006-API-GATEWAY | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

A API é a **interface externa** do KCE. Construída sobre Axum, expõe endpoints REST com contrato OpenAPI, autenticação por API Key, validação de input, timeout e respostas JSON consistentes. É o ponto de entrada para clientes, frontends e integrações.

---

## 2. Critérios de Aceite (AC)

- [ ] AC-010: Endpoints: `POST /query`, `POST /encode`, `POST /action`, `GET /health`, `GET /metrics`
- [ ] AC-011: OpenAPI/Swagger UI disponível via `utoipa`
- [ ] AC-012: Autenticação por API Key (`x-api-key` header) — 401 sem chave
- [ ] AC-013: Timeout configurável por request (default: 100ms)
- [ ] AC-014: Validação de input — rejeita payloads inválidos com 400
- [ ] AC-015: Todas respostas são JSON válido com status codes corretos
- [ ] AC-016: p95 < 100ms
- [ ] AC-017: Taxa de erro < 1% sob carga normal
- [ ] AC-018: Request ID em todas respostas para rastreabilidade
- [ ] AC-019: CORS configurável

---

## 3. Definition of Done (DoD)

- [ ] 5 endpoints implementados e testados
- [ ] OpenAPI spec gerada automaticamente
- [ ] Auth middleware funcional
- [ ] Timeout implementado
- [ ] Validação de input ativa
- [ ] Request ID em todas respostas
- [ ] Testes unitários + funcionais ≥ 80%

---

## 4. Exemplos de Uso

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

### Sem API Key (401)
```json
{ "error": "UNAUTHORIZED", "message": "Missing or invalid x-api-key", "code": 401 }
```

### Payload inválido (400)
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

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Saída Esperada |
|----|------|----------------|
| UT-001 | Validação payload válido | `Ok(parsed)` |
| UT-002 | Validação payload vazio | `Err(ValidationError)` |
| UT-003 | Auth com chave válida | request passa |
| UT-004 | Auth sem chave | 401 |
| UT-005 | Auth chave errada | 401 |
| UT-006 | Request ID gerado | header `x-request-id` presente |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | Body não-JSON | 400 Bad Request |
| UF-002 | Timeout excedido | 408 Request Timeout |
| UF-003 | Endpoint inexistente | 404 Not Found |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | Chamada HTTP real /query | 200 com JSON válido |
| FT-002 | Chamada /health | 200 com status components |
| FT-003 | 100 requests concorrentes | Todas retornam sem erro |
| FT-004 | Swagger UI acessível | /swagger-ui renderiza |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | API → Retrieval → Response | Pipeline completo via HTTP |
| IT-002 | API → MCE | /encode + /action funcionais |
| IT-003 | API → Observabilidade | Request gera log + métrica |

---

## 6. Formato CARE

**Context:** Interface REST do KCE sobre Axum; ponto de entrada para todos os clientes, com contrato OpenAPI, autenticação e validação.

**Assumptions:** Axum 0.7+; API Key é suficiente para MVP (OAuth2 em iteração futura); timeout global configurável; todas respostas são JSON.

**Requirements:** R-001: 5 endpoints | R-002: OpenAPI | R-003: Auth API Key | R-004: Timeout | R-005: Validação | R-006: p95 < 100ms | R-007: Request ID.

**Evidence:** `tests/api_tests.rs` | OpenAPI spec em `/swagger-ui`

---

## 7. Critérios Não Funcionais

| Aspecto | Alvo |
|---------|------|
| Latência p95 | < 100ms |
| Erro rate | < 1% |
| Concorrência | 1000 req/s sustentado |
| Startup time | < 2s |

### Segurança
- API Key obrigatória em todos endpoints exceto /health
- Input sanitizado contra injection
- Headers de segurança (X-Content-Type-Options, X-Frame-Options)

---

## 8. Qualidade e Métricas

**Sucesso:** p95 < 100ms | Erro < 1% | Swagger funcional | Cobertura ≥ 80%  
**Falha (BLOQUEANTE):** p95 > 200ms | Erro > 5% | Auth bypass possível

---

## 9. Compatibilidade e Dependências

| Crate | Versão | Propósito |
|-------|--------|-----------|
| `axum` | ^0.7 | Web framework |
| `utoipa` | ^4 | OpenAPI |
| `utoipa-swagger-ui` | ^6 | Swagger UI |
| `tokio` | ^1.35 | Async runtime |
| `tower` | ^0.4 | Middleware |

---

## 10. Rastreabilidade

| Tipo | ID |
|------|----|
| Spec | FT-006-API-GATEWAY |
| Código | `src/api/mod.rs`, `src/api/routes.rs`, `src/api/middleware.rs` |
| Teste | `tests/api_integration.rs` |

---

## 11. Roadmap MVP

### MVP (Semana 1-2)
POST /query + GET /health | Respostas JSON | Validação básica | 5+ testes

### Iteração 1 (Semana 3-4)
Auth middleware | POST /encode + /action | GET /metrics | OpenAPI/Swagger | Timeout | Request ID

### Iteração 2 (Semana 5-6)
CORS | Rate limiting | Load test 1000 req/s | Integração completa | Docs
