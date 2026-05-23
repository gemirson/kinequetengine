# FT-009 — Resiliência (Retry + Timeout + Circuit Breaker + Backpressure)

**Módulo:** Operação | **Versão:** v5.9 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-009-RESILIENCE | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

Resiliência garante que o KCE **não colapsa sob falha parcial**. Implementa quatro mecanismos: retry com backoff, timeout por request, circuit breaker para isolamento de falha, e backpressure (semaphore) para controlar concorrência. Sem isso, o sistema morre em pico de carga.

---

## 2. Critérios de Aceite (AC)

- [ ] AC-010: Timeout configurável por request (default 100ms)
- [ ] AC-011: Retry com backoff exponencial (max 3 tentativas)
- [ ] AC-012: Circuit breaker abre quando error_rate > 20%
- [ ] AC-013: Circuit breaker fecha após cooldown configurável
- [ ] AC-014: Backpressure via Semaphore (max 100 concurrent requests)
- [ ] AC-015: Falha parcial não derruba o sistema
- [ ] AC-016: Latência controlada sob erro (p95 < 200ms mesmo com falhas)

---

## 3. Definition of Done (DoD)

- [ ] Timeout implementado com `tokio::time::timeout`
- [ ] Retry com backoff funcional
- [ ] Circuit breaker com estados Open/Closed/HalfOpen
- [ ] Semaphore para backpressure
- [ ] Testes de falha parcial passando
- [ ] Cobertura ≥ 80%

---

## 4. Exemplos de Uso

### Timeout
```json
// Request que excede timeout
{ "error": "TIMEOUT", "message": "Request exceeded 100ms timeout", "code": 408 }
```

### Circuit Breaker aberto
```json
{ "error": "CIRCUIT_OPEN", "message": "Too many errors (rate: 0.25). Circuit will retry in 30s", "code": 503 }
```

### Backpressure
```json
{ "error": "OVERLOADED", "message": "Max concurrent requests (100) reached", "code": 429 }
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Saída Esperada |
|----|------|----------------|
| UT-001 | Timeout — operação rápida | Sucesso |
| UT-002 | Timeout — operação lenta | `Err(Timeout)` |
| UT-003 | Retry — sucesso na 2ª tentativa | Sucesso após 1 retry |
| UT-004 | Retry — falha após max tentativas | `Err` após 3 tentativas |
| UT-005 | Circuit breaker — error_rate < 0.2 | Circuito fechado |
| UT-006 | Circuit breaker — error_rate > 0.2 | Circuito abre |
| UT-007 | Semaphore — dentro do limite | Request processado |
| UT-008 | Semaphore — limite excedido | 429 |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | Cascade failure | Circuit breaker previne cascata |
| UF-002 | Timeout + retry | Retry respeita timeout total |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado |
|----|---------|-----------|
| FT-001 | 200 requests concorrentes (limit 100) | 100 OK, 100 → 429 |
| FT-002 | Falha simulada > 20% | Circuit abre → 503 |
| FT-003 | Latência sob erro | p95 < 200ms |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | API → Timeout → Pipeline | Pipeline cancelado em timeout |
| IT-002 | Circuit → KineSQL error | KineSQL failure → circuit abre |

---

## 6. Formato CARE

**Context:** Produção real = falhas acontecem. Sem resiliência, falha parcial vira colapso total. Retry + timeout + circuit breaker + backpressure são obrigatórios.

**Assumptions:** Error rate calculado em janela de 60s; retry backoff exponencial (50ms, 100ms, 200ms); semaphore in-memory; circuit breaker com cooldown de 30s.

**Requirements:** R-001: Timeout | R-002: Retry backoff | R-003: Circuit breaker 3 estados | R-004: Semaphore backpressure | R-005: Falha parcial ≠ colapso | R-006: p95 < 200ms sob erro.

**Evidence:** `tests/resilience_tests.rs` | `benches/load_test.rs`

---

## 7–11. (Resumo)

**Não funcionais:** Overhead de resiliência < 5ms | Latência sob erro p95 < 200ms | 0 cascading failures  
**Qualidade:** Circuit previne 100% das cascatas | Retry recupera ≥ 50% dos erros transientes | Cobertura ≥ 80%  
**Deps:** `tokio` ^1.35 (timeout, semaphore), `retry` ^2  
**Rastreabilidade:** FT-009-RESILIENCE | `src/resilience/mod.rs`

**Roadmap:** MVP: timeout + semaphore | Iter1: retry backoff + circuit breaker | Iter2: adaptive timeout + chaos testing
