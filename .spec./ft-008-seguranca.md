# FT-008 — Segurança (Auth + Multi-Tenant + Validação)

**Módulo:** Segurança | **Versão:** v5.9 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-008-SECURITY | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

Segurança básica obrigatória para produção: autenticação por API Key, isolamento multi-tenant e validação de input. Se o sistema vai servir dados bancários (ex: crédito), isolamento por tenant é mandatório. Sem isso, não é produto.

---

## 2. Critérios de Aceite (AC)

- [ ] AC-010: API Key obrigatória em todos endpoints (exceto /health)
- [ ] AC-011: Requisição sem chave → 401 Unauthorized
- [ ] AC-012: Requisição com chave inválida → 401 Unauthorized
- [ ] AC-013: Isolamento multi-tenant: `tenant_id` obrigatório em requests
- [ ] AC-014: Dados filtrados por `tenant_id` — tenant A não acessa dados de B
- [ ] AC-015: Input sanitizado — rejeita payloads malformados
- [ ] AC-016: Rate limiting por tenant (configurável)

---

## 3. Definition of Done (DoD)

- [ ] Auth middleware implementado e ativo
- [ ] Tenant isolation em todas queries
- [ ] Validação de input em todos endpoints
- [ ] Testes de bypass (negativos) passando
- [ ] Rate limiting funcional

---

## 4. Exemplos de Uso

### Request autenticado
```bash
curl -H "x-api-key: tk_prod_abc123" -H "x-tenant-id: tenant_42" ...
```

### Isolamento de tenant
**Tenant A insere:**
```json
{ "tenant_id": "A", "data": {"id": 1, "vector": [0.5, 0.5]} }
```
**Tenant B busca:**
```json
{ "tenant_id": "B", "query_vector": [0.5, 0.5], "top_k": 10 }
```
**Resultado:** `results: []` — tenant B não vê dados de A.

### Sem API Key
```json
{ "error": "UNAUTHORIZED", "message": "Missing x-api-key header", "code": 401 }
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Saída Esperada |
|----|------|----------------|
| UT-001 | Request com API Key válida | passa auth |
| UT-002 | Request sem API Key | 401 |
| UT-003 | Request com key inválida | 401 |
| UT-004 | Tenant isolation | tenant A ≠ tenant B |
| UT-005 | Rate limit excedido | 429 Too Many Requests |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | SQL injection em input | Rejeitado com 400 |
| UF-002 | Tenant ID vazio | 400 Bad Request |
| UF-003 | Tenant A acessa B | `results: []` ou 403 |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado |
|----|---------|-----------|
| FT-001 | Auth flow completo | 401 → add key → 200 |
| FT-002 | Cross-tenant access | 0 resultados retornados |
| FT-003 | Rate limiting | 429 após exceder limite |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Auth → API → Pipeline | Pipeline só executa autenticado |
| IT-002 | Tenant → KineSQL | Dados filtrados no storage |

---

## 6. Formato CARE

**Context:** Segurança para sistema que pode servir dados financeiros; API Key + tenant isolation + validação são requisitos mínimos para produção.

**Assumptions:** API Key é suficiente para MVP (OAuth2 futuro); tenant_id é string opaca; rate limiting in-memory (Redis em iteração futura).

**Requirements:** R-001: Auth API Key | R-002: Tenant isolation | R-003: Input validation | R-004: Rate limiting | R-005: 401/403/429 corretos.

**Evidence:** `tests/security_tests.rs`

---

## 7–11. (Resumo)

**Não funcionais:** Auth overhead < 1ms | 0 bypasses possíveis | Rate limit configurável por tenant  
**Qualidade:** 0 acessos cross-tenant | 100% requests sem key → 401 | Cobertura ≥ 80%  
**Deps:** `tower` ^0.4 (middleware), `parking_lot` ^0.12  
**Rastreabilidade:** FT-008-SECURITY | `src/api/middleware.rs`, `src/security/tenant.rs`

**Roadmap:** MVP: API Key + validação | Iter1: tenant isolation + rate limiting | Iter2: OAuth2 + audit log
