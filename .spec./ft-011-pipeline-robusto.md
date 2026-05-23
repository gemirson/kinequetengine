# FT-011 — Pipeline Robusto (Orquestração do Pipeline Cognitivo)

**Módulo:** Core Engine | **Versão:** v5.9 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-011-PIPELINE | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O Pipeline é a **orquestração** de todos os módulos do KCE em uma cadeia determinística: validate → rate_limit → plan → retrieve → expand → ecma_update → mce_encode → execute → persist → metrics. Cada etapa retorna `Result<T, EngineError>` — nada assume sucesso. É o coração operacional do sistema.

---

## 2. Critérios de Aceite (AC)

- [ ] AC-010: Pipeline retorna `Result<ActionResult, EngineError>`
- [ ] AC-011: Cada etapa usa `?` para propagação de erro
- [ ] AC-012: Falha em qualquer etapa → erro claro com etapa identificada
- [ ] AC-013: Pipeline determinístico (mesma query 100x → mesmo resultado)
- [ ] AC-014: Pipeline completo < 100ms (p95)
- [ ] AC-015: Métricas registradas ao final de cada execução
- [ ] AC-016: Resultado persistido via KineSQL

---

## 3. Definition of Done (DoD)

- [ ] Pipeline completo implementado (10 etapas)
- [ ] `Result<T, E>` em todas as etapas
- [ ] Determinismo validado (100x → variação < 1%)
- [ ] Métricas e persistência integradas
- [ ] Testes end-to-end passando
- [ ] Cobertura ≥ 80%

---

## 4. Exemplos de Uso

### Pipeline completo
**Entrada:**
```json
{ "query_vector": [0.12, 0.85, 0.33, 0.67], "top_k": 5, "context": "loan_evaluation" }
```

**Fluxo interno:**
```
validate ✓ → rate_limit ✓ → plan ✓ → retrieve (42 candidates) ✓ 
→ expand (8 nodes) ✓ → ecma_update ✓ → mce_encode ✓ 
→ execute (risk_eval) ✓ → persist ✓ → metrics ✓
```

**Saída:**
```json
{
  "action": "risk_eval",
  "result": { "risk_score": 0.72, "recommendation": "APPROVE_WITH_CONDITIONS" },
  "pipeline_ms": 45.2,
  "stages": {
    "validate": 0.1, "retrieve": 12.4, "expand": 3.2,
    "ecma": 1.1, "encode": 2.0, "execute": 2.3, "persist": 8.5
  }
}
```

### Erro em etapa
```json
{
  "error": "PIPELINE_FAILURE",
  "stage": "retrieve",
  "message": "Dimension mismatch in retrieval",
  "code": 500
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Saída Esperada |
|----|------|----------------|
| UT-001 | Pipeline completo — sucesso | `Ok(ActionResult)` |
| UT-002 | Falha em validate | `Err` com stage="validate" |
| UT-003 | Falha em retrieve | `Err` com stage="retrieve" |
| UT-004 | Determinismo 10x | 10 resultados idênticos |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado |
|----|---------|-----------|
| FT-001 | Pipeline end-to-end | Ação final correta |
| FT-002 | Mesma query 100x | Resultado idêntico |
| FT-003 | Pipeline sob carga | p95 < 100ms |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | API → Pipeline → KineSQL | Fluxo completo HTTP → persist |
| IT-002 | Pipeline → Métricas | Cada execução registra métricas |

---

## 6. Formato CARE

**Context:** Orquestração dos 6+ módulos do KCE em pipeline determinístico com error handling robusto; coração operacional do sistema.

**Assumptions:** Cada módulo expõe `Result<T, E>`; pipeline é síncrono dentro de um request; métricas são registradas mesmo em falha.

**Requirements:** R-001: `Result<T, E>` em tudo | R-002: 10 etapas | R-003: Determinismo | R-004: p95 < 100ms | R-005: Métricas + persist.

**Evidence:** `tests/pipeline_e2e.rs` | `benches/pipeline_bench.rs`

---

## 7–11. (Resumo)

**Não funcionais:** p95 < 100ms | Erro claro com stage identificado | 0 panics  
**Qualidade:** Determinismo 100% | Stage identification em 100% dos erros | Cobertura ≥ 80%  
**Deps:** Todos módulos core (Retrieval, Graph, ECMA, MCE, KineSQL)  
**Rastreabilidade:** FT-011-PIPELINE | `src/pipeline/mod.rs`

**Roadmap:** MVP: 5 etapas básicas (validate → retrieve → encode → execute → respond) | Iter1: +expand, +ecma, +persist, +metrics | Iter2: determinismo validado + chaos testing + carga 1000 qps
