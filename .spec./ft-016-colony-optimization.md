# FT-016 — Context Colony Optimization

**Módulo:** ACO (Ant Colony Optimization) | **Versão:** v6.0 | **Prioridade:** P1 — Alto  
**Artefato ID:** FT-016-COLONY-OPTIMIZATION | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O Context Colony Optimization combina **múltiplas buscas e explorações** para convergir em uma solução ótima global — não apenas local. Agrega resultados de múltiplos ciclos ACO (deposit + evaporate + explore) para produzir um ranking final que considera todo o histórico de feromônio acumulado. Substitui ranking simples por **sistema emergente de otimização global**.

### Valor
- Substitui ranking simples por otimização global emergente
- Combina informação de múltiplos ciclos para convergência
- Decisões de contexto melhoram ao longo do tempo

---

## 2. Critérios de Aceite (AC)

- [ ] AC-010: `optimize(query, cycles)` executa N ciclos ACO completos
- [ ] AC-011: Cada ciclo: explore (formigas) → deposit (feromônio) → evaporate → rank
- [ ] AC-012: Convergência medida: variação entre ciclos < threshold → estável
- [ ] AC-013: Resultado final é ranking global considerando histórico ACO completo
- [ ] AC-014: `convergence_threshold` configurável (default: 0.05)
- [ ] AC-015: Early stop quando convergência atingida antes de max_cycles
- [ ] AC-016: Métricas de convergência expostas (`cycles_to_converge`, `final_score_variance`)
- [ ] AC-017: Fallback para ranking estático se ACO não converge em max_cycles

---

## 3. Definition of Done (DoD)

- [ ] Ciclo ACO completo (explore → deposit → evaporate → rank) implementado
- [ ] Convergência medida e early stop funcional
- [ ] Fallback para ranking estático
- [ ] Métricas de convergência expostas
- [ ] Testes de convergência passando
- [ ] Cobertura ≥ 80%

---

## 4. Exemplos de Uso

### Otimização com convergência

**Entrada:**
```json
{
  "query_vector": [0.12, 0.85, 0.33, 0.67],
  "max_cycles": 20,
  "ants_per_cycle": 5,
  "convergence_threshold": 0.05
}
```

**Saída:**
```json
{
  "optimal_result": {
    "path": [42, 17, 88, 5],
    "score": 0.96,
    "confidence": 0.93
  },
  "convergence": {
    "cycles_executed": 12,
    "converged": true,
    "final_variance": 0.03,
    "score_progression": [0.72, 0.78, 0.85, 0.89, 0.92, 0.94, 0.95, 0.95, 0.96, 0.96, 0.96, 0.96]
  },
  "total_latency_ms": 145.2
}
```

### Não convergência → fallback
```json
{
  "optimal_result": { "path": [42, 33, 5], "score": 0.78 },
  "convergence": {
    "cycles_executed": 20,
    "converged": false,
    "final_variance": 0.12,
    "fallback": "static_ranking"
  }
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Saída Esperada |
|----|------|----------------|
| UT-001 | Convergência em < 20 ciclos | early stop ativado |
| UT-002 | Não convergência | max_cycles atingido, fallback |
| UT-003 | Score melhora entre ciclos | progressão monotônica crescente |
| UT-004 | Variância < threshold | `converged: true` |
| UT-005 | 1 ciclo | funciona como explore simples |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | max_cycles = 0 | `Err(InvalidCycles)` |
| UF-002 | Grafo vazio | `Err(EmptyGraph)` |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado |
|----|---------|-----------|
| FT-001 | Otimização em dataset 10k | Convergência em < 20 ciclos |
| FT-002 | Comparação ACO vs ranking estático | ACO ≥ 15% melhor em score |
| FT-003 | Sob carga concorrente | 20 otimizações simultâneas OK |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Colony → Exploration → Pheromone → Evaporation | Ciclo completo funcional |
| IT-002 | Colony → Pipeline | Pipeline usa resultado otimizado |
| IT-003 | Colony → Métricas | Convergência registrada |

---

## 6. Formato CARE

**Context:** Busca local (top-k simples) não garante ótimo global. Colony Optimization executa múltiplos ciclos ACO completos para convergir em solução global, combinando histórico de feromônio de todas as iterações.

**Assumptions:** 20 ciclos máximos é suficiente para convergência na maioria dos grafos; variância < 0.05 indica convergência; fallback estático é aceitável se ACO não converge; latência total < 500ms é aceitável para otimização global.

**Requirements:** R-001: Ciclo ACO completo | R-002: Convergência medida | R-003: Early stop | R-004: Fallback estático | R-005: Métricas | R-006: Resultado global.

**Evidence:** `tests/colony_optimization_tests.rs` | `reports/aco_vs_static.md`

---

## 7–11. (Resumo)

**Não funcionais:** Latência total < 500ms para 20 ciclos | Convergência em < 20 ciclos (90% dos casos) | Melhoria ≥ 15% vs estático  
**Qualidade:** Convergência 90%+ | Score ≥ 15% melhor | Cobertura ≥ 80%  
**Falha (BLOQUEANTE):** Score pior que estático | Convergência < 50% | Latência > 2s  
**Deps:** Exploration (FT-015), Pheromone (FT-013), Evaporation (FT-014)  
**Rastreabilidade:** FT-016-COLONY-OPTIMIZATION | `src/aco/colony.rs`

**Roadmap:** MVP: 1 ciclo ACO | Iter1: multi-ciclo + convergência + early stop | Iter2: fallback + métricas + benchmark vs estático + docs

---

*Documento gerado em 2026-05-23 — KineContext Engine Product Specification*
