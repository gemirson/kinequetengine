# FT-021 — Adaptive Mutation Engine

**Módulo:** AIS (Artificial Immune System) | **Versão:** v6.0 | **Prioridade:** P1 — Alto  
**Artefato ID:** FT-021-ADAPTIVE-MUTATION | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O Adaptive Mutation Engine **muta representações de contexto** (embeddings) de forma controlada para explorar soluções novas e evitar estagnação. Aplica perturbações leves aos vetores de busca, testando novas combinações que podem revelar contexto relevante não descoberto pela busca direta. Inspirado em hipermutação somática de anticorpos.

### Valor
- Inovação emergente — descobre contexto fora do "caminho batido"
- Evita estagnação quando rotas ACO convergem demais
- Exploração controlada sem comprometer estabilidade

---

## 2. Critérios de Aceite (AC)

- [ ] AC-010: `mutate(vector, rate)` retorna vetor perturbado dentro de range controlado
- [ ] AC-011: Taxa de mutação (`rate`) configurável (default: 0.05, range: 0.0..0.3)
- [ ] AC-012: Mutação preserva norma L2 do vetor (re-normaliza após perturbação)
- [ ] AC-013: Perturbação gaussiana com σ proporcional à rate
- [ ] AC-014: `explore_mutations(vector, n)` gera N variantes e testa cada uma
- [ ] AC-015: Seleção do melhor mutante por score de retrieval
- [ ] AC-016: Mutante superior ao original → substitui no pipeline (elitismo)
- [ ] AC-017: Mutação é reversível — original preservado se nenhum mutante melhora
- [ ] AC-018: Métricas: `kce_mutations_total`, `kce_mutations_successful`, `kce_mutation_improvement_avg`

---

## 3. Definition of Done (DoD)

- [ ] `mutate()` com perturbação gaussiana e re-normalização
- [ ] `explore_mutations()` com seleção do melhor
- [ ] Elitismo (original preservado se melhor)
- [ ] Rate configurável com validação
- [ ] Métricas de mutação
- [ ] Cobertura ≥ 80%

---

## 4. Exemplos de Uso

### Mutação de vetor de busca

**Entrada:**
```json
{
  "original_vector": [0.12, 0.85, 0.33, 0.67],
  "mutation_rate": 0.05,
  "num_mutations": 5
}
```

**Saída:**
```json
{
  "original_score": 0.82,
  "mutations": [
    { "id": 0, "vector": [0.14, 0.83, 0.35, 0.65], "score": 0.85, "improvement": 0.03 },
    { "id": 1, "vector": [0.10, 0.87, 0.31, 0.69], "score": 0.79, "improvement": -0.03 },
    { "id": 2, "vector": [0.13, 0.84, 0.36, 0.64], "score": 0.88, "improvement": 0.06 },
    { "id": 3, "vector": [0.11, 0.86, 0.32, 0.68], "score": 0.81, "improvement": -0.01 },
    { "id": 4, "vector": [0.15, 0.82, 0.34, 0.66], "score": 0.80, "improvement": -0.02 }
  ],
  "best_mutation": { "id": 2, "score": 0.88, "improvement": 0.06 },
  "selected": "mutation_2",
  "elitism": true
}
```

### Sem melhoria → preserva original
```json
{
  "original_score": 0.95,
  "best_mutation_score": 0.91,
  "selected": "original",
  "elitism": true,
  "reason": "No mutation improved on original"
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Saída Esperada |
|----|------|----------------|
| UT-001 | Mutação preserva dimensão | `mutated.len() == original.len()` |
| UT-002 | Norma L2 preservada | `‖mutated‖₂ ≈ ‖original‖₂` (±1%) |
| UT-003 | Rate=0.0 → sem mudança | `mutated == original` |
| UT-004 | Rate=0.3 → perturbação visível | distância > 0 |
| UT-005 | Elitismo — mutante pior | original mantido |
| UT-006 | Elitismo — mutante melhor | mutante selecionado |
| UT-007 | Reprodutibilidade com seed | seed fixo → mesma mutação |
| UT-008 | N mutações geradas | exatamente N variantes |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | Rate > 0.3 | `Err(MutationRateTooHigh)` |
| UF-002 | Rate negativa | `Err(InvalidRate)` |
| UF-003 | Vetor vazio | `Err(EmptyVector)` |
| UF-004 | N = 0 | `Err(InvalidMutationCount)` |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado |
|----|---------|-----------|
| FT-001 | 100 mutações: ≥ 20% melhoram original | Taxa de sucesso ≥ 20% |
| FT-002 | Mutação sob convergência ACO | Descobre rota nova ≥ 10% dos casos |
| FT-003 | Pipeline com mutação ativa | Score médio melhora ≥ 3% |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Mutation → Retrieval | Mutantes testados no Retrieval Engine |
| IT-002 | Mutation → ACO | Mutações alimentam exploração ACO |
| IT-003 | Mutation → Métricas | `kce_mutations_successful` registrado |

---

## 6. Formato CARE

**Context:** Quando ACO converge forte, o sistema pode ficar preso em local optima. A mutação adaptativa explora soluções novas perturbando embeddings de forma controlada, inspirada em hipermutação somática de anticorpos.

**Assumptions:** Perturbação gaussiana é adequada; taxa de 5% é conservadora para início; re-normalização L2 mantém compatibilidade com cosine similarity; elitismo garante estabilidade; seed para reprodutibilidade em debug.

**Requirements:** R-001: mutate() gaussiana | R-002: Re-normalização L2 | R-003: explore_mutations() com seleção | R-004: Elitismo | R-005: Rate configurável | R-006: Métricas | R-007: Reprodutibilidade com seed.

**Evidence:** `tests/mutation_tests.rs` | `reports/mutation_impact.md`

---

## 7–11. (Resumo)

**Não funcionais:** Latência mutação < 1ms | Overhead pipeline < 5ms com 5 mutantes | Memória < 1MB extra  
**Qualidade:** Taxa de melhoria ≥ 20% | Score médio ≥ 3% melhor | Norma L2 preservada ±1% | Cobertura ≥ 80%  
**Falha (BLOQUEANTE):** Norma L2 diverge > 5% | Taxa de melhoria < 5% | Crash em mutação  
**Deps:** Retrieval (FT-001), ACO (FT-013-016), `rand` ^0.8  
**Rastreabilidade:** FT-021-ADAPTIVE-MUTATION | `src/ais/mutation.rs`

**Roadmap:** MVP: mutate() gaussiana + re-norm | Iter1: explore_mutations() + elitismo + seed | Iter2: Integração ACO + pipeline + métricas + adaptive rate

---

*Documento gerado em 2026-05-23 — KineContext Engine Product Specification*
