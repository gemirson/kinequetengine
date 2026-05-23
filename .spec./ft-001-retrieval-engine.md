# FT-001 — Retrieval Engine (Busca Híbrida)

**Módulo:** Core Engine | **Versão:** v5.9 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-001-RETRIEVAL-ENGINE | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O Retrieval Engine é o primeiro estágio do pipeline cognitivo do KCE. Realiza buscas híbridas combinando **cosine similarity** (SIMD/AVX2) e **prime similarity** (GCD). Recebe query vetorial e retorna candidatos relevantes para Graph Engine e ECMA. Deve operar com datasets de até 1M+ vetores em produção.

---

## 2. Critérios de Aceite (AC)

### Gerais
- [ ] AC-001: Compila sem warnings em `--release`
- [ ] AC-002: Thread-safe via `Arc<RwLock<...>>` / `parking_lot`
- [ ] AC-003: Interface pública com doc comments

### Específicos
- [ ] AC-010: Cosine similarity: `1.0` para vetores idênticos, `0.0` para ortogonais, `-1.0` para opostos
- [ ] AC-011: Prime similarity via GCD normalizado
- [ ] AC-012: Busca híbrida com pesos configuráveis (`cosine_weight`, `prime_weight`)
- [ ] AC-013: SIMD (AVX2) ativo com fallback seguro
- [ ] AC-014: Early pruning com threshold mínimo parametrizável
- [ ] AC-015: Ordenação determinística (score desc, id asc como tiebreaker)
- [ ] AC-016: Rayon com `max_threads` configurável
- [ ] AC-017: Recall@10 ≥ 0.85 (dataset sintético)
- [ ] AC-018: p95 < 50ms para 100k vetores

---

## 3. Definition of Done (DoD)

- [ ] SIMD ativo (AVX2 ou fallback)
- [ ] Prime similarity implementado e testado
- [ ] Paralelismo Rayon funcional e configurável
- [ ] Early pruning funcional
- [ ] Determinismo garantido (variação < 1%)
- [ ] Testes unitários ≥ 80% cobertura
- [ ] Testes funcionais passando em CI
- [ ] Benchmark baseline registrado
- [ ] Sem `unwrap()` em código de produção
- [ ] Code review aprovado

---

## 4. Exemplos de Uso

### Busca híbrida top-5

**Entrada (JSON):**
```json
{
  "query_vector": [0.12, 0.85, 0.33, 0.67],
  "top_k": 5,
  "similarity": "hybrid",
  "weights": { "cosine": 0.7, "prime": 0.3 },
  "threshold": 0.1
}
```

**Saída:**
```json
{
  "results": [
    { "id": 42, "score": 0.9523, "cosine_score": 0.9712, "prime_score": 0.9082 },
    { "id": 17, "score": 0.8891, "cosine_score": 0.9001, "prime_score": 0.8635 }
  ],
  "latency_ms": 12.4,
  "total_candidates": 100000,
  "pruned_candidates": 87234
}
```

### Erro — dimensão incompatível
```json
{ "error": "DIMENSION_MISMATCH", "message": "Query dim (3) != dataset dim (4)", "code": 400 }
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Entrada | Saída Esperada |
|----|------|---------|----------------|
| UT-001 | Cosine — vetores idênticos | `[1,0], [1,0]` | `1.0` |
| UT-002 | Cosine — ortogonais | `[1,0], [0,1]` | `0.0` |
| UT-003 | Cosine — opostos | `[1,0], [-1,0]` | `-1.0` |
| UT-004 | Cosine — vetor zero | `[0,0], [1,0]` | `0.0` (sem panic) |
| UT-005 | Prime — GCD máximo | `12, 12` | `1.0` |
| UT-006 | Hybrid — peso 50/50 | cos=0.8, prime=0.6 | `0.7` |
| UT-007 | Top-k > dataset | dataset=3, k=10 | `3 resultados` |
| UT-008 | Ordering — scores iguais | score=0.5, score=0.5 | `menor id primeiro` |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | Dimensão incompatível | `Err(DimensionMismatch)` |
| UF-002 | Vetor vazio | `Err(EmptyVector)` |
| UF-003 | Top-k = 0 | `Err(InvalidK)` |
| UF-004 | NaN em vetor | `Err(InvalidVector)` |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | Inserir 10k vetores + query | Top-10 consistente, scores decrescentes |
| FT-002 | 50 queries concorrentes | Todos retornam sem erro |
| FT-003 | Performance 100k vetores | p95 < 50ms |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Retrieval → Graph | Candidatos expandidos com vizinhos |
| IT-002 | Retrieval → ECMA | Maturidade dos nós atualizada |
| IT-003 | API → Retrieval | HTTP 200 com JSON válido |

---

## 6. Formato CARE

**Context:** Primeiro estágio do pipeline cognitivo; busca vetorial híbrida para retrieval semântico de alta qualidade em datasets de até 1M+ vetores.

**Assumptions:** CPU alvo suporta AVX2 (fallback existe); vetores normalizados L2; dataset cabe em memória/mmap; concorrência via `Arc<RwLock>`.

**Requirements:** R-001: SIMD cosine | R-002: Prime via GCD | R-003: Hybrid com pesos | R-004: p95 < 50ms (100k) | R-005: Recall@10 ≥ 0.85 | R-006: Determinismo < 1% variação.

**Evidence:** `benches/retrieval_bench.rs` | `testdata/synth_100k_128d.bin` | `reports/recall_analysis.md`

---

## 7. Critérios Não Funcionais

| Aspecto | Métrica | Alvo |
|---------|---------|------|
| Latência p50 | 100k vetores | < 20ms |
| Latência p95 | 100k vetores | < 50ms |
| Throughput | queries/s | ≥ 500 |
| Memória | vs tamanho dataset | < 2x |

---

## 8. Qualidade e Métricas

**Sucesso:** Recall@10 ≥ 0.85 | Precision@10 ≥ 0.70 | Variação < 1% | Cobertura ≥ 80%

**Falha (BLOQUEANTE):** Recall@10 < 0.70 | p95 > 100ms | Crash em teste funcional | Variação > 5%

---

## 9. Compatibilidade e Dependências

| Crate | Versão | Propósito |
|-------|--------|-----------|
| `rayon` | ^1.8 | Paralelismo |
| `parking_lot` | ^0.12 | RwLock otimizado |

**Deps internas:** KineSQL (leitura vetores), Graph Engine (consumo candidatos), ECMA (maturidade)  
**Rust:** ≥ 1.75 | **OS:** Linux (prod), macOS (dev) | **CPU:** x86_64 AVX2 (preferencial)

---

## 10. Rastreabilidade

| Tipo | ID | Descrição |
|------|----|-----------|
| Spec | FT-001-RETRIEVAL-ENGINE | Esta especificação |
| Código | `src/retrieval/mod.rs` | Implementação |
| Bench | `benches/retrieval_bench.rs` | Benchmark |
| Teste | `tests/retrieval_integration.rs` | Integração |

---

## 11. Roadmap MVP

### MVP (Semana 1-2)
Cosine similarity scalar | Top-k retrieval | 5+ testes unitários | API interna `retrieve(query, dataset, k)`

### Iteração 1 (Semana 3-4)
SIMD AVX2 + fallback | Prime similarity | Hybrid scoring | Rayon | Benchmark baseline | p95 < 50ms

### Iteração 2 (Semana 5-6)
Early pruning | Determinismo validado | Error handling completo | Integração KineSQL + Graph | Testes de carga 500 qps | Docs completos
