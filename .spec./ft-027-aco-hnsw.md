# FT-027 — ACO-HNSW (Bio-Inspired Vector Indexing)

**Módulo:** Core Engine / ACO | **Versão:** v6.0 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-027-ACO-HNSW | **Atualização:** 2026-05-30

---

## 1. Contexto e Objetivo

Os índices de busca vetorial modernos baseados em grafos, como o HNSW (Hierarchical Navigable Small World), utilizam travessia gulosa (greedy search) e conexões geométricas estáticas. Embora eficientes, esses métodos não se adaptam aos padrões reais de uso das consultas.

O **ACO-HNSW** substitui a busca puramente geométrica por uma travessia dinâmica baseada em **Colônias de Formigas (Ant Colony Optimization)**. O grafo multicamadas armazena não apenas as distâncias euclidianas das arestas, mas também um peso dinâmico de **feromônio ($\tau$)**. Consultas frequentes e semanticamente semelhantes criam e reforçam "rodovias cognitivas" que guiam as formigas de busca até os vizinhos mais relevantes, reduzindo o número total de nós visitados e estabilizando a latência de cauda (P99) sob alta carga de trabalho.

---

## 2. Critérios de Aceite (AC)

### Gerais
- [ ] AC-001: Compilação sem warnings no ecossistema Rust workspace.
- [ ] AC-002: Acesso concorrente thread-safe para busca simultânea e escrita de feromônios via `parking_lot`.
- [ ] AC-003: Documentação completa da API pública no código.

### Específicos
- [ ] AC-010: O grafo de busca deve conter múltiplas camadas. Cada aresta deve mapear a distância vetorial e um nível de feromônio $\tau \ge 0.0$ (inicializado com um valor padrão $\tau_0$).
- [ ] AC-011: As buscas no índice devem suportar a travessia probabilística guiada pela fórmula de transição de feromônio:
  $$P_{ij} = \frac{[\tau_{ij}]^\alpha \cdot [\eta_{ij}]^\beta}{\sum_{k \in \mathcal{N}(i)} [\tau_{ik}]^\alpha \cdot [\eta_{ik}]^\beta}$$
  onde $\tau_{ij}$ representa a força do feromônio, $\eta_{ij}$ é a similaridade inversa da distância do nó vizinho $j$ em relação ao vetor de consulta, $\mathcal{N}(i)$ é o conjunto de vizinhos do nó atual, e $\alpha, \beta$ são os pesos configuráveis de controle.
- [ ] AC-012: Mecanismo de **Reforço de Caminho**: ao final de uma busca com sucesso, o caminho percorrido pelas formigas que encontraram os melhores candidatos (Top-K) recebe um incremento de feromônio:
  $$\Delta \tau_{ij} = Q \cdot \text{Score}_{\text{Similarity}}$$
- [ ] AC-013: Mecanismo de **Evaporação Temporal**: a cada intervalo de tempo ou número de consultas, o feromônio das arestas evapora seguindo a constante de decaimento $\rho$ definida na feature FT-014:
  $$\tau_{ij} \leftarrow (1 - \rho) \cdot \tau_{ij}$$
- [ ] AC-014: Suporte a fallback dinâmico: se o regulador de rede (FT-022) indicar instabilidade ou esgotamento de memória, o índice desativa o cálculo probabilístico de ACO e opera no modo de busca clássica HNSW gulosa.
- [ ] AC-015: Latência de cauda P99 $\le 8\text{ ms}$ para datasets de até 100k vetores (128d) a 1000 QPS.

---

## 3. Definition of Done (DoD)

- [ ] Estrutura de dados de grafo multicamadas com vetores e feromônios implementada.
- [ ] Roteamento estocástico de formigas no grafo usando geradores de números pseudo-aleatórios eficientes por thread.
- [ ] Threads de busca paralelizadas com Rayon para as formigas explorarem caminhos concorrentemente.
- [ ] Sistema de evaporação assíncrono acoplado ao temporizador geral do motor de ACO.
- [ ] Testes unitários com no mínimo 85% de cobertura de código nos caminhos quentes de busca.
- [ ] Benchmarks comparativos mostrando redução de saltos (hops) no grafo para buscas semânticas repetidas em comparação ao HNSW clássico.

---

## 4. Exemplos de Uso

### Payload de Busca no Índice ACO-HNSW (JSON)

**Entrada (POST /query/aco-hnsw):**
```json
{
  "query_vector": [0.05, 0.91, -0.12, 0.43],
  "top_k": 5,
  "aco_params": {
    "alpha": 1.2,
    "beta": 2.0,
    "num_ants": 8,
    "pheromone_deposit": 0.5
  }
}
```

**Saída:**
```json
{
  "results": [
    { "id": 1024, "score": 0.985, "hops": 3 },
    { "id": 2048, "score": 0.912, "hops": 4 },
    { "id": 512, "score": 0.887, "hops": 5 }
  ],
  "metrics": {
    "total_hops_visited": 12,
    "avg_hops_per_ant": 4.1,
    "latency_ms": 3.42,
    "pheromone_highways_utilized": ["1024->2048"]
  }
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Entrada | Saída Esperada |
|----|------|---------|----------------|
| UT-001 | Fórmula de Transição | $\tau_{ij}=1.0$, $\eta_{ij}=0.8$, $\alpha=1.0$, $\beta=1.0$ | Probabilidade uniforme entre vizinhos com pesos idênticos |
| UT-002 | Reforço de Aresta | Aresta visitada com score 0.9 | Pheromone $\tau$ incrementado conforme a constante $Q$ |
| UT-003 | Evaporação de Arestas | $\tau=2.0$, $\rho=0.1$ | Após evaporação, $\tau$ deve ser $1.8$ |
| UT-004 | Clamping de Pheromone | Reduções repetidas por evaporação | $\tau$ não deve decair abaixo do limite mínimo definido |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | Carga Semântica Repetida | 100 queries repetidas devem usar caminhos idênticos com menor número de saltos à medida que o feromônio acumula |
| FT-002 | Mudança de Distribuição (Drift) | Quando o assunto das consultas muda abruptamente, as estradas de feromônio antigas devem evaporar e novas conexões devem ser reforçadas |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | ACO-HNSW → Evaporação (FT-014) | Ciclos periódicos de evaporação do módulo de ACO limpam as arestas do HNSW |
| IT-002 | ACO-HNSW → Regulação (FT-022) | O regulador reduz o número de formigas se a latência p95 ultrapassar 10ms |

---

## 6. Formato CARE

**Context:** Recuperação vetorial aproximada de baixíssima latência para o pipeline cognitivo de produção do KCE, operando sob cargas de trabalho reais dinâmicas.

**Assumptions:** A memória RAM é suficiente para manter os vetores e a matriz esparsa de feromônios das arestas de busca. Os caminhos de busca repetidos seguem leis de potência comuns em aplicações reais de busca semântica.

**Requirements:** R-001: Roteamento baseado em probabilidade ACO | R-002: Integração com multi-camadas de HNSW | R-003: Feedback dinâmico de leitura e reforço.

**Evidence:** Benchmarks localizados em `benches/aco_hnsw_bench.rs` demonstrando taxas de acerto e contagem de saltos.

---

## 7. Critérios Não Funcionais

| Aspecto | Métrica | Alvo |
|---------|---------|------|
| Latência de Busca | p95 para base de 100k vetores | < 5ms |
| Latência de Escrita | Tempo para atualizar o feromônio | < 0.5ms (assíncrono) |
| Overhead de Memória | Espaço adicional para o float de feromônio por aresta | < 12% em relação ao grafo HNSW puro |

---

## 8. Qualidade e Métricas

**Sucesso:**
- Recall@10 superior a 0.90 em testes de carga sintética.
- Redução de pelo menos 20% no número de nós visitados em consultas repetidas versus busca HNSW clássica.
- Zero vazamentos ou panics na concorrência de escrita de feromônios.

**Falha (BLOQUEANTE):**
- Recall@10 abaixo de 0.80.
- Travamento por deadlocks nas escritas de feromônios.

---

## 9. Compatibilidade e Dependências

| Crate | Versão | Propósito |
|-------|--------|-----------|
| `rayon` | ^1.8 | Paralelismo de formigas exploradoras |
| `rand` | ^0.8 | Roteamento probabilístico estocástico |
| `parking_lot` | ^0.12 | Locks rápidos sem contenção de kernel |

---

## 10. Rastreabilidade

| Tipo | ID | Descrição |
|------|----|-----------|
| Spec | FT-027-ACO-HNSW | Esta especificação |
| Código | `crates/kce-retrieval/src/aco_hnsw.rs` | Implementação do índice |
| Bench | `benches/aco_hnsw_bench.rs` | Teste comparativo de performance |

---

## 11. Roadmap

### MVP (Fase 1)
Implementação de grafo de camada única navegável utilizando busca por formiga única determinística.

### Iteração 1 (Fase 2)
Implementação de grafo multicamadas (HNSW completo) com busca estocástica por múltiplas formigas paralelas com Rayon.

### Iteração 2 (Fase 3)
Integração de hardware acelerado, auto-ajuste de $\alpha$ e $\beta$ pelo regulador global e persistência do estado dos feromônios no KineSQL.
