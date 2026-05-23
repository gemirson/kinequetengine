# FT-015 — Ant Query Exploration Engine

**Módulo:** ACO (Ant Colony Optimization) | **Versão:** v6.0 | **Prioridade:** P1 — Alto  
**Artefato ID:** FT-015-ANT-EXPLORATION | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O Ant Query Exploration Engine executa **múltiplas "formigas" (queries paralelas)** explorando caminhos diferentes no grafo semântico simultaneamente. Cada formiga segue uma heurística levemente diferente (variando α, β, ou ponto de partida), e o melhor resultado é selecionado. Inspirado na exploração estocástica de colônias de formigas reais.

### Valor
- Melhora recall em ambientes complexos com múltiplos caminhos válidos
- Evita local optima via exploração paralela
- Rotas alternativas são descobertas naturalmente

---

## 2. Critérios de Aceite (AC)

- [ ] AC-010: `explore(query, num_ants)` lança N formigas paralelas
- [ ] AC-011: Cada formiga segue heurística diferente (variação de α, β, seed)
- [ ] AC-012: Seleção do melhor resultado por score composto (relevância + latência)
- [ ] AC-013: Paralelismo via Rayon (não bloqueia thread principal)
- [ ] AC-014: Resultados de todas formigas disponíveis para análise (não só o melhor)
- [ ] AC-015: `num_ants` configurável (default: 5, max: 20)
- [ ] AC-016: Timeout por formiga (formiga lenta é cancelada)
- [ ] AC-017: Melhores caminhos depositam feromônio (integração FT-013)
- [ ] AC-018: Diversidade mínima garantida (ao menos 2 formigas com rotas diferentes)

---

## 3. Definition of Done (DoD)

- [ ] `explore()` funcional com N formigas paralelas
- [ ] Variação de heurística por formiga implementada
- [ ] Seleção do melhor resultado funcional
- [ ] Timeout por formiga ativo
- [ ] Depósito de feromônio pela melhor formiga
- [ ] Testes unitários ≥ 80%
- [ ] Benchmark de recall comparativo (N formigas vs 1 query)

---

## 4. Exemplos de Uso

### Exploração com 5 formigas

**Entrada:**
```json
{
  "query_vector": [0.12, 0.85, 0.33, 0.67],
  "num_ants": 5,
  "timeout_ms": 50,
  "diversity_min": 2
}
```

**Saída:**
```json
{
  "best_result": {
    "ant_id": 2,
    "path": [42, 17, 88, 5],
    "score": 0.94,
    "latency_ms": 18.2,
    "heuristic": { "alpha": 1.2, "beta": 1.8 }
  },
  "all_ants": [
    { "ant_id": 0, "path": [42, 33, 5], "score": 0.82, "latency_ms": 12.1 },
    { "ant_id": 1, "path": [42, 91, 12, 5], "score": 0.76, "latency_ms": 22.4 },
    { "ant_id": 2, "path": [42, 17, 88, 5], "score": 0.94, "latency_ms": 18.2 },
    { "ant_id": 3, "path": [42, 17, 88, 5], "score": 0.91, "latency_ms": 19.0 },
    { "ant_id": 4, "path": [42, 7, 5], "score": 0.65, "latency_ms": 8.3, "status": "timeout" }
  ],
  "unique_paths": 4,
  "total_latency_ms": 22.4,
  "pheromone_deposited_on": [42, 17, 88, 5]
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Saída Esperada |
|----|------|----------------|
| UT-001 | 5 formigas lançadas | 5 resultados retornados |
| UT-002 | Best result selecionado | maior score escolhido |
| UT-003 | Diversidade mínima | ≥ 2 rotas distintas |
| UT-004 | Timeout de formiga | formiga lenta cancelada |
| UT-005 | num_ants = 1 | funciona como query simples |
| UT-006 | Heurísticas diferentes | α, β variam entre formigas |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | num_ants = 0 | `Err(InvalidAntCount)` |
| UF-002 | num_ants > 20 | `Err(TooManyAnts)` |
| UF-003 | Todas formigas timeout | `Err(AllAntsTimedOut)` |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado |
|----|---------|-----------|
| FT-001 | 5 formigas vs 1 query (recall) | Recall ≥ 10% melhor com formigas |
| FT-002 | Grafo com múltiplos caminhos | Formigas exploram rotas distintas |
| FT-003 | Sob carga (100 explorações) | Todas completam sem erro |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Exploration → Pheromone | Melhor rota recebe depósito |
| IT-002 | Exploration → Pipeline | Pipeline usa resultado da melhor formiga |
| IT-003 | Exploration → Retrieval | Formigas usam Retrieval Engine |

---

## 6. Formato CARE

**Context:** Query única pode ficar presa em local optima. Múltiplas formigas exploram caminhos divergentes simultaneamente, maximizando recall em grafos complexos.

**Assumptions:** Rayon disponível para paralelismo; grafo tem múltiplos caminhos entre nós; overhead de N formigas é aceitável (< 3x latência de 1 query); melhor formiga deposita feromônio.

**Requirements:** R-001: N formigas paralelas | R-002: Heurística variada por formiga | R-003: Seleção do melhor | R-004: Timeout por formiga | R-005: Diversidade mínima | R-006: Depósito pela melhor.

**Evidence:** `tests/ant_exploration_tests.rs` | `reports/recall_improvement.md`

---

## 7–11. (Resumo)

**Não funcionais:** Latência total < 3x query simples | Throughput ≥ 200 explorations/s | Memória < 10MB extra para 20 formigas  
**Qualidade:** Recall ≥ 10% melhor que query simples | Diversidade ≥ 2 rotas | Cobertura ≥ 80%  
**Falha (BLOQUEANTE):** Recall pior que query simples | 0 diversidade | Deadlock em parallelismo  
**Deps:** Rayon ^1.8, Pheromone (FT-013), Retrieval (FT-001), Graph (FT-002)  
**Rastreabilidade:** FT-015-ANT-EXPLORATION | `src/aco/exploration.rs`

**Roadmap:** MVP: 3 formigas sequenciais + seleção | Iter1: Rayon parallelismo + timeout + diversidade | Iter2: Adaptive num_ants + depósito + benchmark recall

---

*Documento gerado em 2026-05-23 — KineContext Engine Product Specification*
