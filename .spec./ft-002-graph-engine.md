# FT-002 — Graph Engine (Grafo Semântico)

**Módulo:** Core Engine | **Versão:** v5.9 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-002-GRAPH-ENGINE | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O Graph Engine gerencia o grafo semântico do KCE, responsável pela **expansão contextual** dos candidatos retornados pelo Retrieval Engine. Conecta conceitos via arestas ponderadas e expande o contexto de busca para além da similaridade direta, permitindo ao ECMA e MCE operar com visão relacional do conhecimento.

---

## 2. Critérios de Aceite (AC)

### Gerais
- [ ] AC-001: Compila sem warnings em `--release`
- [ ] AC-002: Thread-safe para leitura concorrente

### Específicos
- [ ] AC-010: `add_edge(a, b, weight)` cria aresta bidirecional
- [ ] AC-011: `neighbors(node_id)` retorna vizinhos diretos com pesos
- [ ] AC-012: `expand(nodes, depth)` retorna subgrafo expandido até profundidade `depth`
- [ ] AC-013: Expansão retorna ≤ 2x nós de entrada
- [ ] AC-014: Sem duplicação de nós no resultado da expansão
- [ ] AC-015: Latência de expansão < 10ms
- [ ] AC-016: Expansão determinística (mesma entrada → mesma saída)
- [ ] AC-017: Remoção de arestas funcional (`remove_edge`)
- [ ] AC-018: Grafo suporta ≥ 100k nós sem degradação

---

## 3. Definition of Done (DoD)

- [ ] `add_edge` funcional e testado
- [ ] `neighbors` consistente com adição/remoção
- [ ] Expansão determinística implementada
- [ ] Sem duplicação de nós
- [ ] Testes unitários ≥ 80% cobertura
- [ ] Testes funcionais passando
- [ ] Documentação da API pública

---

## 4. Exemplos de Uso

### Expansão contextual
**Entrada:**
```json
{
  "seed_nodes": [42, 17, 88],
  "expansion_depth": 2,
  "max_nodes": 20
}
```

**Saída:**
```json
{
  "expanded_nodes": [42, 17, 88, 5, 12, 33, 91, 7],
  "edges": [
    { "from": 42, "to": 5, "weight": 0.85 },
    { "from": 17, "to": 12, "weight": 0.72 },
    { "from": 88, "to": 33, "weight": 0.68 }
  ],
  "depth_reached": 2,
  "total_nodes": 8
}
```

### Erro — nó inexistente
```json
{ "error": "NODE_NOT_FOUND", "message": "Node 9999 does not exist in graph", "code": 404 }
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Entrada | Saída Esperada |
|----|------|---------|----------------|
| UT-001 | add_edge bidirecional | `add_edge(1, 2, 0.5)` | `neighbors(1)` contém 2 e vice-versa |
| UT-002 | neighbors — nó isolado | `neighbors(99)` | `[]` |
| UT-003 | expand depth=1 | seed=[1], depth=1 | nó 1 + vizinhos diretos |
| UT-004 | expand sem duplicatas | ciclo 1→2→3→1 | cada nó aparece 1x |
| UT-005 | remove_edge | `remove_edge(1,2)` | `neighbors(1)` não contém 2 |
| UT-006 | expand max_nodes | 100 vizinhos, max=5 | exatamente 5 nós |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | Nó inexistente | `Err(NodeNotFound)` |
| UF-002 | Depth negativo | `Err(InvalidDepth)` |
| UF-003 | Aresta duplicada | Ignora silenciosamente (idempotente) |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | Criar grafo 1k nós + expandir | Subgrafo correto em < 10ms |
| FT-002 | Expansão concorrente | 50 expansions simultâneas sem erro |
| FT-003 | Grafo após persistência | Reload do KineSQL mantém estrutura |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Retrieval → Graph → ECMA | Candidatos expandidos alimentam ECMA |
| IT-002 | Graph → KineSQL | Grafo persiste e recupera corretamente |

---

## 6. Formato CARE

**Context:** Grafo semântico para expansão contextual de candidatos do Retrieval Engine; permite relações além de similaridade direta.

**Assumptions:** Grafo é esparso (avg degree < 10); cabe em memória; arestas são bidirecionais com peso; concorrência de leitura é mais frequente que escrita.

**Requirements:** R-001: add/remove_edge | R-002: neighbors O(1) | R-003: expand com profundidade | R-004: ≤ 2x nós de entrada | R-005: latência < 10ms | R-006: determinismo.

**Evidence:** `tests/graph_tests.rs` | `benches/graph_bench.rs`

---

## 7. Critérios Não Funcionais

| Aspecto | Alvo |
|---------|------|
| Latência expansão | < 10ms |
| Memória (100k nós) | < 100MB |
| Concorrência | 50 leituras simultâneas sem degradação |

---

## 8. Qualidade e Métricas

**Sucesso:** Sem duplicatas em expansão | Determinismo 100% | Cobertura ≥ 80%  
**Falha (BLOQUEANTE):** Duplicatas no resultado | Latência > 50ms | Crash em concorrência

---

## 9. Compatibilidade e Dependências

**Deps internas:** Retrieval Engine (entrada), ECMA (saída), KineSQL (persistência)  
**Rust:** ≥ 1.75 | **Crates:** `parking_lot` ^0.12

---

## 10. Rastreabilidade

| Tipo | ID |
|------|----|
| Spec | FT-002-GRAPH-ENGINE |
| Código | `src/graph/mod.rs` |
| Teste | `tests/graph_integration.rs` |

---

## 11. Roadmap MVP

### MVP (Semana 1-2)
`add_edge` + `neighbors` | 3+ testes unitários | Estrutura adjacency list

### Iteração 1 (Semana 3-4)
`expand` com profundidade | Deduplicação | `remove_edge` | Determinismo | Benchmark

### Iteração 2 (Semana 5-6)
Integração KineSQL (persistência) | Integração Retrieval→Graph→ECMA | Testes de carga | Docs
