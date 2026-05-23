# FT-013 — Pheromone Routing Engine (ACO Core)

**Módulo:** ACO (Ant Colony Optimization) | **Versão:** v6.0 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-013-PHEROMONE-ROUTING | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O Pheromone Routing Engine é o **núcleo do sistema ACO** do KCE. Cada interação (query, retrieval, execução) deposita um `pheromone_score` no caminho percorrido no grafo semântico. Caminhos mais usados e mais eficientes acumulam feromônio e se tornam preferenciais para futuras buscas. Substitui heurística estática por **aprendizado emergente** — o sistema aprende quais rotas de contexto são mais valiosas sem regras codificadas.

### Valor
- Substitui heurística estática por aprendizado emergente
- Rotas de contexto se auto-otimizam com uso
- Comportamento melhora organicamente sem retraining

---

## 2. Critérios de Aceite (AC)

### Gerais
- [ ] AC-001: Compila sem warnings em `--release`
- [ ] AC-002: Thread-safe via `Arc<RwLock<...>>`
- [ ] AC-003: Interface pública com doc comments

### Específicos
- [ ] AC-010: Cada aresta do grafo possui campo `pheromone_score: f64` (inicializado em `1.0`)
- [ ] AC-011: `deposit(edge, score)` incrementa feromônio proporcionalmente ao sucesso da query
- [ ] AC-012: `route(source, target)` seleciona caminho com probabilidade proporcional ao feromônio
- [ ] AC-013: Seleção probabilística segue fórmula ACO: `P(edge) = τ^α * η^β / Σ(τ^α * η^β)` onde τ=feromônio, η=heurística
- [ ] AC-014: Parâmetros `α` (peso feromônio) e `β` (peso heurística) configuráveis
- [ ] AC-015: Feromônio máximo (`τ_max`) e mínimo (`τ_min`) para evitar estagnação (MMAS — Max-Min Ant System)
- [ ] AC-016: Depósito é proporcional à qualidade do resultado (`1/latency` ou `recall_score`)
- [ ] AC-017: Histórico de depósitos rastreável para auditoria
- [ ] AC-018: Integração com Graph Engine (FT-002) para rotas

---

## 3. Definition of Done (DoD)

- [ ] `deposit(edge, score)` implementado e testado
- [ ] `route(source, target)` com seleção probabilística ACO
- [ ] Fórmula ACO implementada com α, β configuráveis
- [ ] Limites τ_max / τ_min ativos (MMAS)
- [ ] Integração com Graph Engine funcional
- [ ] Testes unitários ≥ 80% cobertura
- [ ] Benchmark de convergência registrado
- [ ] Sem `unwrap()` em código de produção

---

## 4. Exemplos de Uso

### Depósito de feromônio após query bem-sucedida

**Entrada (evento de depósito):**
```json
{
  "path": [
    { "from": 42, "to": 17, "edge_id": "e_42_17" },
    { "from": 17, "to": 88, "edge_id": "e_17_88" },
    { "from": 88, "to": 5, "edge_id": "e_88_5" }
  ],
  "quality_score": 0.92,
  "latency_ms": 12.4,
  "deposit_strategy": "quality_proportional"
}
```

**Estado após depósito:**
```json
{
  "edges_updated": [
    { "edge_id": "e_42_17", "pheromone_before": 1.0, "pheromone_after": 1.92, "delta": 0.92 },
    { "edge_id": "e_17_88", "pheromone_before": 1.5, "pheromone_after": 2.42, "delta": 0.92 },
    { "edge_id": "e_88_5", "pheromone_before": 0.8, "pheromone_after": 1.72, "delta": 0.92 }
  ],
  "total_deposit": 2.76
}
```

### Seleção de rota por feromônio

**Entrada:**
```json
{
  "source_node": 42,
  "target_node": 5,
  "alpha": 1.0,
  "beta": 2.0,
  "num_candidates": 3
}
```

**Saída (rotas candidatas com probabilidade):**
```json
{
  "routes": [
    { "path": [42, 17, 88, 5], "probability": 0.65, "total_pheromone": 5.06 },
    { "path": [42, 33, 5], "probability": 0.25, "total_pheromone": 3.20 },
    { "path": [42, 91, 12, 5], "probability": 0.10, "total_pheromone": 1.80 }
  ],
  "selected_route": [42, 17, 88, 5],
  "selection_method": "roulette_wheel"
}
```

### Erro — nós desconectados
```json
{ "error": "NO_ROUTE", "message": "No path between node 42 and node 999", "code": 404 }
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Entrada | Saída Esperada |
|----|------|---------|----------------|
| UT-001 | Depósito incrementa feromônio | deposit(edge, 0.5) | pheromone += 0.5 |
| UT-002 | Feromônio inicial = 1.0 | nova aresta | `pheromone == 1.0` |
| UT-003 | τ_max respeitado | depósitos excessivos | `pheromone <= τ_max` |
| UT-004 | τ_min respeitado | após evaporação | `pheromone >= τ_min` |
| UT-005 | Seleção probabilística | 2 rotas, τ=[5.0, 1.0] | rota 1 selecionada ~83% (α=1, β=0) |
| UT-006 | α=0 ignora feromônio | qualquer τ | seleção uniforme |
| UT-007 | Rota determinística seed | seed fixo | mesma rota sempre |
| UT-008 | Deposit quality_proportional | score=0.9 | delta proporcional |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | Nós desconectados | `Err(NoRoute)` |
| UF-002 | Score negativo | `Err(InvalidScore)` |
| UF-003 | α ou β negativos | `Err(InvalidParams)` |
| UF-004 | Edge inexistente | `Err(EdgeNotFound)` |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | 100 queries na mesma rota | Feromônio acumula, rota se torna dominante |
| FT-002 | Rotas alternativas sob carga | Rota melhor converge para >60% seleção |
| FT-003 | Integração com pipeline | Pipeline usa rota ACO ao invés de fixed |
| FT-004 | Concorrência de depósitos | 50 depósitos simultâneos sem corrupção |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Pheromone → Graph Engine | Depósitos refletem no grafo |
| IT-002 | Pipeline → Pheromone → Retrieval | Retrieval usa rotas pheromone-weighted |
| IT-003 | Pheromone → KineSQL | Scores persistem após restart |
| IT-004 | Pheromone → Evaporation (FT-014) | Evaporação reduz scores corretamente |

---

## 6. Formato CARE

**Context:** O KCE utiliza heurísticas estáticas para routing de contexto. O Pheromone Routing substitui isso por aprendizado emergente inspirado em colônias de formigas (ACO). Cada interação deposita feromônio digital nos caminhos do grafo, fazendo com que rotas mais eficientes se reforcem naturalmente.

**Assumptions:**
- O grafo semântico (FT-002) já existe e suporta pesos em arestas
- Feromônio é um float f64 com limites configuráveis
- Seleção probabilística usa roulette wheel selection
- Concorrência de depósitos é gerenciada via RwLock
- α e β default são 1.0 e 2.0 respectivamente (ACO clássico)

**Requirements:**
- R-001: Depósito de feromônio proporcional à qualidade
- R-002: Seleção probabilística ACO (P = τ^α * η^β / Σ)
- R-003: MMAS — limites τ_max, τ_min
- R-004: Parâmetros α, β configuráveis
- R-005: Integração com Graph Engine
- R-006: Persistência via KineSQL
- R-007: Thread-safe para depósitos concorrentes

**Evidence:**
- `benches/pheromone_convergence.rs` — benchmark de convergência
- `tests/pheromone_routing_tests.rs` — testes unitários
- `reports/aco_analysis.md` — análise de convergência vs heurística estática

---

## 7. Critérios de Aceitação Não Funcionais

### Desempenho
| Métrica | Alvo | Método |
|---------|------|--------|
| Latência deposit | < 0.5ms | Benchmark |
| Latência route selection | < 5ms | Benchmark |
| Convergência | < 50 iterações para rota ótima | Dataset sintético |
| Throughput deposits | ≥ 5000/s | `wrk` |

### Segurança
- Scores validados (não-negativos, não-NaN)
- Limites τ_max/τ_min impedem manipulação
- Audit trail de depósitos

### Acessibilidade
- N/A (módulo backend)

---

## 8. Critérios de Qualidade e Métricas

### Métricas de Sucesso
| Métrica | Valor Alvo |
|---------|------------|
| Melhoria de recall vs estático | ≥ 10% |
| Convergência para rota ótima | < 50 iterações |
| Taxa de exploração mantida | ≥ 15% (não fica preso) |
| Cobertura de testes | ≥ 80% |

### Critérios de Falha
- Convergência > 200 iterações → **BLOQUEANTE**
- Feromônio fora de [τ_min, τ_max] → **BLOQUEANTE**
- Stagnation (0% exploração) → **BLOQUEANTE**
- Corrupção em depósito concorrente → **BLOQUEANTE**

---

## 9. Compatibilidade e Dependências

### Dependências Diretas
| Crate | Versão | Propósito |
|-------|--------|-----------|
| `rand` | ^0.8 | Seleção probabilística |
| `parking_lot` | ^0.12 | RwLock otimizado |

### Dependências Internas
- **Graph Engine (FT-002):** estrutura de arestas para depósito
- **Evaporation (FT-014):** decaimento de feromônio
- **KineSQL (FT-005):** persistência de scores
- **Pipeline (FT-011):** integração no fluxo

### Compatibilidade
| Item | Requisito |
|------|-----------|
| Rust | ≥ 1.75 stable |
| OS | Linux (prod), macOS (dev) |

---

## 10. Critérios de Rastreabilidade

### Histórico de Mudanças
| Data | Versão | Descrição | Autor |
|------|--------|-----------|-------|
| 2026-05-23 | v1.0 | Criação da especificação inicial | Product Specialist |

### IDs de Artefatos Relacionados
| Tipo | ID | Descrição |
|------|----|-----------|
| Spec | FT-013-PHEROMONE-ROUTING | Esta especificação |
| Código | `src/aco/pheromone.rs` | Implementação principal |
| Código | `src/aco/routing.rs` | Seleção de rotas |
| Bench | `benches/pheromone_convergence.rs` | Benchmark convergência |
| Teste | `tests/pheromone_routing_tests.rs` | Testes unitários |
| Dep | FT-002, FT-005, FT-014 | Features dependentes |

---

## 11. Entrega e Critérios de Aceitação do MVP — Roadmap

### MVP (Semana 1-2)
| Item | Critério de Aceite |
|------|--------------------|
| `deposit(edge, score)` | Incrementa feromônio corretamente |
| `get_pheromone(edge)` | Retorna score atual |
| Feromônio inicial = 1.0 | Testado |
| 5+ testes unitários | Passando |

**Saída:** Depósito funcional de feromônio em arestas do grafo.

---

### Iteração 1 (Semana 3-4)
| Item | Critério de Aceite |
|------|--------------------|
| `route(source, target)` | Seleção probabilística ACO |
| Fórmula ACO (τ^α * η^β) | Implementada |
| MMAS (τ_max, τ_min) | Limites ativos |
| Integração Graph Engine | Depósitos refletem no grafo |
| Benchmark convergência | Registrado (< 50 iter) |

**Saída:** Routing funcional com seleção ACO integrada ao grafo.

---

### Iteração 2 (Semana 5-6)
| Item | Critério de Aceite |
|------|--------------------|
| Persistência KineSQL | Scores sobrevivem restart |
| Integração Pipeline | Pipeline usa rotas ACO |
| Integração Evaporation (FT-014) | Decaimento funcional |
| Concorrência validada | 50 depósitos simultâneos OK |
| Audit trail | Histórico de depósitos rastreável |
| Testes de carga | 5000 deposits/s sustentado |
| Documentação completa | API docs + exemplos |

**Saída:** Pheromone Routing production-ready integrado ao pipeline KCE.

---

*Documento gerado em 2026-05-23 — KineContext Engine Product Specification*
