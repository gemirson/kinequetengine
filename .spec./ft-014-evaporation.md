# FT-014 — Evaporation Mechanism

**Módulo:** ACO (Ant Colony Optimization) | **Versão:** v6.0 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-014-EVAPORATION | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O Evaporation Mechanism implementa o **decaimento temporal de feromônio** no sistema ACO. Sem evaporação, caminhos antigos dominam permanentemente — criando "memória tóxica" e overfitting de contexto. A evaporação garante que rotas obsoletas perdem relevância, mantendo o sistema adaptativo e vivo. Fórmula clássica: `τ(t+1) = (1 - ρ) * τ(t)`, onde `ρ` é a taxa de evaporação.

### Valor
- Evita "memória tóxica" — contexto obsoleto decai naturalmente
- Mantém sistema adaptativo — novas rotas têm chance de emergir
- Previne overfitting — conhecimento estagnado é gradualmente removido

---

## 2. Critérios de Aceite (AC)

### Gerais
- [ ] AC-001: Compila sem warnings em `--release`
- [ ] AC-002: Thread-safe para evaporação concorrente

### Específicos
- [ ] AC-010: Evaporação segue fórmula `τ(t+1) = (1 - ρ) * τ(t)`
- [ ] AC-011: Taxa de evaporação `ρ` configurável (default: 0.1, range: 0.0..1.0)
- [ ] AC-012: Evaporação executada periodicamente (intervalo configurável, default: 60s)
- [ ] AC-013: Respeita `τ_min` — feromônio nunca cai abaixo do mínimo (MMAS)
- [ ] AC-014: `evaporate_all()` processa todas as arestas do grafo
- [ ] AC-015: `evaporate_edge(edge_id)` processa aresta individual
- [ ] AC-016: Métricas de evaporação registradas (`edges_evaporated`, `total_decay`)
- [ ] AC-017: Evaporação não bloqueia queries (read lock durante evaporação)
- [ ] AC-018: Log de evaporação para auditoria

---

## 3. Definition of Done (DoD)

- [ ] Fórmula de evaporação implementada
- [ ] ρ configurável com validação de range
- [ ] Timer periódico funcional (tokio interval)
- [ ] τ_min respeitado em todos os cenários
- [ ] Integração com Pheromone Engine (FT-013)
- [ ] Métricas de evaporação expostas
- [ ] Testes unitários ≥ 80% cobertura
- [ ] Sem `unwrap()` em produção

---

## 4. Exemplos de Uso

### Evaporação periódica

**Estado antes (arestas do grafo):**
```json
{
  "edges": [
    { "edge_id": "e_42_17", "pheromone": 5.0 },
    { "edge_id": "e_17_88", "pheromone": 2.0 },
    { "edge_id": "e_88_5",  "pheromone": 0.3 }
  ],
  "rho": 0.1,
  "tau_min": 0.1
}
```

**Após evaporação (ρ = 0.1):**
```json
{
  "edges": [
    { "edge_id": "e_42_17", "pheromone": 4.5,  "decay": -0.5 },
    { "edge_id": "e_17_88", "pheromone": 1.8,  "decay": -0.2 },
    { "edge_id": "e_88_5",  "pheromone": 0.27, "decay": -0.03 }
  ],
  "total_decay": -0.73,
  "edges_processed": 3,
  "edges_at_tau_min": 0
}
```

### Evaporação com τ_min atingido
```json
{
  "edge_id": "e_88_5",
  "pheromone_before": 0.12,
  "pheromone_calculated": 0.108,
  "pheromone_after": 0.1,
  "clamped_to_tau_min": true
}
```

### Configuração (CSV para batch)
```csv
parameter,value,description
rho,0.1,taxa de evaporação
tau_min,0.1,feromônio mínimo
interval_seconds,60,intervalo entre ciclos
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Entrada | Saída Esperada |
|----|------|---------|----------------|
| UT-001 | Evaporação básica | τ=5.0, ρ=0.1 | τ=4.5 |
| UT-002 | Evaporação com ρ=0 | τ=5.0, ρ=0.0 | τ=5.0 (sem mudança) |
| UT-003 | Evaporação com ρ=1 | τ=5.0, ρ=1.0 | τ=τ_min (clamp) |
| UT-004 | τ_min respeitado | τ=0.12, ρ=0.5 | τ=τ_min=0.1 |
| UT-005 | evaporate_all | 100 arestas | todas processadas |
| UT-006 | Múltiplos ciclos | 10 evaporações | decaimento exponencial correto |
| UT-007 | Concorrência | evap + deposit simultâneos | sem corrupção |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | ρ negativo | `Err(InvalidEvaporationRate)` |
| UF-002 | ρ > 1.0 | `Err(InvalidEvaporationRate)` |
| UF-003 | τ_min > τ_max | `Err(InvalidBounds)` |
| UF-004 | Grafo vazio | `Ok` com edges_processed=0 |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | 100 ciclos de evaporação | Todos feromônios convergem para τ_min sem uso |
| FT-002 | Evaporação + depósito simultâneo | Equilíbrio dinâmico alcançado |
| FT-003 | Timer periódico 60s | Evaporação dispara corretamente |
| FT-004 | Rota dominante decai | Nova rota emerge após evaporação |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Evaporation → Pheromone (FT-013) | Scores no grafo decaem corretamente |
| IT-002 | Evaporation → KineSQL | Scores evaporados persistem |
| IT-003 | Evaporation → Métricas | `kce_evaporation_total` registrado |
| IT-004 | Evaporation + Pipeline | Rotas se adaptam após evaporação |

---

## 6. Formato CARE

**Context:** Sem evaporação, o sistema ACO sofre de "memória tóxica" — caminhos antigos dominam permanentemente, impedindo adaptação. O mecanismo garante que feromônio decai exponencialmente com o tempo, forçando o sistema a revalidar rotas continuamente.

**Assumptions:**
- ρ default = 0.1 é adequado para a maioria dos workloads
- Evaporação periódica (não contínua) é suficiente para MVP
- τ_min garante que nenhuma aresta fica completamente "morta"
- Concorrência entre evaporação e depósito é resolvida por RwLock (write lock breve)
- Timer via `tokio::time::interval`

**Requirements:**
- R-001: Fórmula `τ(t+1) = (1-ρ)*τ(t)` implementada
- R-002: ρ configurável (0.0..1.0)
- R-003: τ_min como floor (MMAS)
- R-004: Timer periódico configurável
- R-005: Métricas de evaporação
- R-006: Não bloqueia queries

**Evidence:**
- `tests/evaporation_tests.rs`
- `benches/evaporation_bench.rs`
- `reports/evaporation_dynamics.md`

---

## 7. Critérios de Aceitação Não Funcionais

| Aspecto | Alvo |
|---------|------|
| Latência evaporate_all (10k arestas) | < 50ms |
| Lock time durante evaporação | < 10ms |
| Overhead no throughput de queries | < 2% |
| Memória adicional | < 1MB |

---

## 8. Critérios de Qualidade e Métricas

**Sucesso:** τ_min nunca violado | Decaimento exponencial correto | Novas rotas emergem | Cobertura ≥ 80%  
**Falha (BLOQUEANTE):** Feromônio abaixo de τ_min | Bloqueio de queries > 100ms | Stagnation não resolvida

---

## 9. Compatibilidade e Dependências

**Deps internas:** Pheromone Engine (FT-013), Graph Engine (FT-002), KineSQL (FT-005)  
**Crates:** `tokio` ^1.35 (timer), `parking_lot` ^0.12  
**Rust:** ≥ 1.75

---

## 10. Rastreabilidade

| Tipo | ID |
|------|----|
| Spec | FT-014-EVAPORATION |
| Código | `src/aco/evaporation.rs` |
| Teste | `tests/evaporation_tests.rs` |
| Dep | FT-013, FT-002, FT-005 |

---

## 11. Roadmap MVP

### MVP (Semana 1-2)
`evaporate_edge()` com fórmula ACO | ρ configurável | τ_min clamp | 5+ testes unitários

### Iteração 1 (Semana 3-4)
`evaporate_all()` | Timer periódico | Integração Pheromone Engine | Métricas | Benchmark

### Iteração 2 (Semana 5-6)
Concorrência validada | Persistência KineSQL | Adaptive ρ (auto-tune) | Testes de carga | Docs

---

*Documento gerado em 2026-05-23 — KineContext Engine Product Specification*
