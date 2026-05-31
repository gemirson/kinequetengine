# FT-026 — Native Python Bindings (PyO3)

**Módulo:** Interface | **Versão:** v6.0 | **Prioridade:** P1 — Importante  
**Artefato ID:** FT-026-PYTHON-BINDINGS | **Atualização:** 2026-05-30

---

## 1. Contexto e Objetivo

O **Native Python Bindings** expõe as funcionalidades de alta performance do KCE (desenvolvido em Rust) diretamente no ecossistema Python. Utilizando a biblioteca PyO3, este módulo gera um pacote dinâmico compilado (`.so`/`.pyd`) que pode ser importado em scripts Python tradicionais (`import kce_python`). Isso simplifica a adoção do motor cognitivo em projetos de ciência de dados e engenharia de machine learning sem comprometer a performance computacional do backend Rust.

---

## 2. Critérios de Aceite (AC)

### Gerais
- [ ] AC-001: Compilação dinâmica limpa em sistemas Linux (gerando arquivo `.so`)
- [ ] AC-002: Gerenciamento eficiente do Global Interpreter Lock (GIL) do Python nas operações pesadas
- [ ] AC-003: Mapeamento claro de erros do Rust para exceções adequadas do Python (`ValueError`, `RuntimeError`)

### Específicos
- [ ] AC-010: Exposição da classe `PyLatencyHistogram` com métodos `record(us)`, `percentiles()` e `reset()`.
- [ ] AC-011: Exposição da função `retrieve(query_vector, top_k)` retornando uma lista de tuplas `(id, score)`.
- [ ] AC-012: Exposição da função `encode(nodes)` recebendo uma lista de tuplas `(state, maturity)` e retornando o payload mRNA compilado em bytes.
- [ ] AC-013: Exposição da função `classify(vector, self_patterns)` retornando o label de classificação e o score de confiança.
- [ ] AC-014: Exposição da função `sinkhorn_distance(a, b)` retornando o valor da distância de transporte ótimo Sinkhorn.
- [ ] AC-015: Exposição da função `cosine_similarity(a, b)` retornando a similaridade de cosseno de forma rápida.
- [ ] AC-016: Preservação da integridade de memória nas conversões de dados complexos entre CPython e a runtime Rust.

---

## 3. Definition of Done (DoD)

- [ ] Extensão compilada via maturin / cargo-c com sucesso
- [ ] 6 funções nativas expostas e testadas via scripts Python
- [ ] Classe de telemetria `PyLatencyHistogram` funcional e documentada
- [ ] Testes unitários com o framework `pytest` passando em CI
- [ ] Mapeamento e propagação corretos de exceções sem quebras catastróficas (Core Dumps)
- [ ] Documentação de API no padrão Docstring integrada

---

## 4. Exemplos de Uso

### Chamada no Script Python

```python
import kce_python

# 1. Distância Sinkhorn entre duas distribuições
dist = kce_python.sinkhorn_distance([0.2, 0.8], [0.5, 0.5])
print(f"Distância Sinkhorn: {dist}")

# 2. Medir similaridade cosseno
score = kce_python.cosine_similarity([1.0, 0.0], [1.0, 0.0])
assert score == 1.0

# 3. Classificação Imunológica (AIS)
label, confidence = kce_python.classify(
    vector=[0.1, 0.2, 0.3],
    self_patterns=[
        [0.1, 0.2, 0.3],
        [0.11, 0.19, 0.29],
        [0.09, 0.21, 0.31]
    ]
)
print(f"Classificação: {label} (Confiança: {confidence})")

# 4. Histograma de Latência
hist = kce_python.latency_histogram_new()
hist.record(1200) # 1.2ms
hist.record(850)  # 0.85ms
metrics = hist.percentiles()
print(f"P95 latency: {metrics['p95_us']} us")
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários (pytest)

| ID | Caso | Entrada | Saída Esperada |
|----|------|---------|----------------|
| UT-001 | Cosine Similarity | Dois vetores idênticos | Retorna `1.0` |
| UT-002 | Erro de dimensão no cosseno | Vetores com tamanhos diferentes | Lança `ValueError` no Python |
| UT-003 | Classificador AIS | Vetor idêntico a um padrão self | Label `"self"` com alta confiança |
| UT-004 | Histograma percentis | Gravação de amostras | Dicionário contendo count, mean, p50, p95 e p99 |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | Integração com numpy/scipy | Vetores extraídos de arrays numpy são convertidos e processados corretamente |
| FT-002 | Validação de Garbage Collection | Alocação repetida e desalocação de instâncias de histograma não geram vazamentos de memória |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Python Binding → MceEngine | A função `encode` traduz strings de estado ("Stem", "Apoptosis") para enums Rust, retornando bytes válidos |

---

## 6. Formato CARE

**Context:** Disponibilização da biblioteca KCE para cientistas de dados e engenheiros de aprendizado de máquina que utilizam Python, unindo a facilidade da linguagem ao desempenho computacional em Rust.

**Assumptions:** O Python de destino é v3.8 ou superior. Os vetores passados são conversíveis para tipos primitivos (`f64`). O compilador PyO3 gerencia as referências de forma segura sob a GIL.

**Requirements:** R-001: Empacotamento compilado nativo | R-002: Tradução robusta de enums e tipos estruturados | R-003: Coerência matemática nas saídas comparadas com o core Rust.

**Evidence:** Suíte de testes `pytest` localizada em `crates/kce-python/tests/` executada em pipeline local.

---

## 7. Critérios Não Funcionais

| Aspecto | Métrica | Alvo |
|---------|---------|------|
| Overhead de Conversão | Custo extra de passagem de dados Python -> Rust | < 2% sobre o custo da operação nativa |
| Compatibilidade de Tipos | Coerção de tipos numéricos do Python/Numpy | Suporte a inteiros e floats de 64 bits |

---

## 8. Qualidade e Métricas

**Sucesso:** Passagem bem-sucedida de todos os testes unitários via `pytest`, sem ocorrências de falhas catastróficas de segmentação (`segmentation faults`).

**Falha (BLOQUEANTE):** Fugas de memória sistemáticas verificadas no loop de GC do interpretador ou estouro de pilha por conversão cíclica.

---

## 9. Compatibilidade e Dependências

| Crate / Tool | Versão | Propósito |
|--------------|--------|-----------|
| `pyo3` | ^0.21 | Interface Rust-Python |
| `maturin` | ^1.0 | Builder e empacotador da extensão |
| Python | >= 3.8 | Interpretador runtime alvo |

---

## 10. Rastreabilidade

| Tipo | ID | Descrição |
|------|----|-----------|
| Spec | FT-026-PYTHON-BINDINGS | Esta especificação |
| Código | `crates/kce-python/src/lib.rs` | Bindings dinâmicas PyO3 |

---

## 11. Roadmap

### MVP (Fase Atual)
Exportação direta das métricas matemáticas, classificação de anomalias por vetor de similaridade e exportação do histograma de latência.

### Iteração 1
Suporte para carregar datasets inteiros do KineSQL diretamente a partir de caminhos de arquivo no Python.

### Iteração 2
Suporte para paralelismo multi-thread com liberação explícita da GIL (`py.allow_threads`).
