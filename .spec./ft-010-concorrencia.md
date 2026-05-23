# FT-010 — Concorrência (Thread Safety)

**Módulo:** Infraestrutura | **Versão:** v5.9 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-010-CONCURRENCY | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O KCE v5 é single-thread safe "por sorte". Para produção, **concorrência segura é obrigatória**. Todas estruturas mutáveis devem usar `Arc<RwLock<...>>` (via `parking_lot` para performance). Idempotência de requests via cache + request_id.

---

## 2. Critérios de Aceite (AC)

- [ ] AC-010: Estruturas mutáveis protegidas por `Arc<RwLock<...>>`
- [ ] AC-011: `parking_lot` usado ao invés de `std::sync`
- [ ] AC-012: Read lock para leitura, write lock para escrita
- [ ] AC-013: Sem deadlocks sob carga concorrente (100 threads)
- [ ] AC-014: Request idempotência via request_id + cache
- [ ] AC-015: Pipeline determinístico sob paralelismo (ordenação estável)

---

## 3. Definition of Done (DoD)

- [ ] Todas estruturas mutáveis com `Arc<RwLock<...>>`
- [ ] `parking_lot` integrado
- [ ] Idempotência via request cache
- [ ] Ordenação estável no pipeline
- [ ] Testes com `#[test]` + threads concorrentes
- [ ] 0 deadlocks em stress test

---

## 4. Exemplos de Uso

### SharedState
```rust
pub struct SharedState {
    pub db: Arc<RwLock<KineSQL>>,
    pub cache: Arc<RwLock<HashMap<String, Response>>>,
}

// Leitura
let db = state.db.read();
// Escrita
let mut db = state.db.write();
```

### Idempotência
```json
// Request 1
{ "request_id": "req_abc", "query_vector": [0.5, 0.5] }
// Response: computed and cached

// Request 2 (mesmo request_id)
{ "request_id": "req_abc", "query_vector": [0.5, 0.5] }
// Response: returned from cache (idêntica)
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Saída Esperada |
|----|------|----------------|
| UT-001 | Read lock concorrente | 50 readers simultâneos OK |
| UT-002 | Write lock exclusivo | Writer bloqueia readers |
| UT-003 | Idempotência | request_id duplicado → cache hit |
| UT-004 | Ordenação estável | mesma query, threads diferentes → mesma ordem |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | Write lock timeout | `Err` após timeout (sem deadlock) |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado |
|----|---------|-----------|
| FT-001 | 100 threads concorrentes | 0 panics, 0 deadlocks |
| FT-002 | Read + write simultâneo | Consistência mantida |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | API → SharedState → Pipeline | Concorrência segura end-to-end |

---

## 6. Formato CARE

**Context:** Sistema precisa suportar 1000+ req/s concorrentes; single-thread "por sorte" não é aceitável em produção.

**Assumptions:** `parking_lot` é mais rápido que `std::sync`; read-heavy workload (90% reads); idempotência cache em memória com TTL.

**Requirements:** R-001: Arc<RwLock> em tudo mutável | R-002: parking_lot | R-003: 0 deadlocks | R-004: idempotência | R-005: determinismo.

**Evidence:** `tests/concurrency_stress.rs`

---

## 7–11. (Resumo)

**Não funcionais:** Lock overhead < 1μs | 0 deadlocks | Throughput ≥ 1000 req/s  
**Qualidade:** 0 deadlocks | 0 data races | Cobertura ≥ 80%  
**Deps:** `parking_lot` ^0.12  
**Rastreabilidade:** FT-010-CONCURRENCY | `src/state.rs`

**Roadmap:** MVP: Arc<RwLock> + parking_lot | Iter1: idempotência cache | Iter2: lock-free structures para hot paths
