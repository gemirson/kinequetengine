# FT-023 — Optimal Transport Context Distance (Wasserstein)

**Módulo:** Core Engine — Distance Metrics | **Versão:** v6.0 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-023-OPTIMAL-TRANSPORT | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O Optimal Transport (OT) Context Distance calcula a **distância entre dois contextos como o custo mínimo de transformar uma distribuição semântica na outra**, usando a métrica de Wasserstein. Diferente de cosine similarity (que mede ângulo), OT captura **estrutura distribuída**, multimodalidade, deslocamento semântico e evolução de contexto.

### Formulação

```
W_p(μ, ν) = ( inf_{γ ∈ Π(μ,ν)} ∫ ‖x - y‖^p dγ(x, y) )^(1/p)
```

| Conceito Matemático | KCE Mapping |
|---------------------|-------------|
| μ, ν | CARE contexts (distribuições) |
| γ | Plano de transporte ótimo |
| custo | Diferença semântica real |

### Valor
- **Retrieval upgrade brutal:** cosine → filtro rápido, OT → ranking final
- **ECMA evolução real:** mede quanto um contexto mudou ao longo do tempo
- **Immune System:** detecta context drift e anomalias reais (não só outliers)
- **ACO:** custo de aresta = Wasserstein distance → otimização global realista
- Captura multimodalidade, deslocamento semântico e evolução de contexto

### Estratégia de Performance

OT puro é O(n³). Solução para produção:

1. **Sinkhorn Distance** (regularizado) — rápido + aproximado
2. **Entropic Regularization** — controle de custo computacional
3. **Batching + GPU** (futuro) — escala massiva

---

## 2. Critérios de Aceite (AC)

### Gerais
- [ ] AC-001: Compila sem warnings em `--release`
- [ ] AC-002: Thread-safe via `Arc<RwLock<...>>`
- [ ] AC-003: Interface pública com doc comments

### Específicos — Propriedades Matemáticas
- [ ] AC-010: Distância simétrica: `W(A, B) == W(B, A)`
- [ ] AC-011: Distância = 0 se contextos iguais: `W(A, A) == 0.0`
- [ ] AC-012: Desigualdade triangular: `W(A, C) <= W(A, B) + W(B, C)`
- [ ] AC-013: Monotonicidade preservada: contextos mais diferentes → distância maior
- [ ] AC-014: Não-negatividade: `W(A, B) >= 0.0` sempre

### Específicos — Implementação
- [ ] AC-020: `wasserstein(ctx_a, ctx_b)` retorna distância f64
- [ ] AC-021: `sinkhorn(ctx_a, ctx_b, epsilon)` retorna distância aproximada (Sinkhorn)
- [ ] AC-022: Sinkhorn com entropic regularization (ε configurável, default: 0.01)
- [ ] AC-023: Convergência Sinkhorn em < 100 iterações (threshold: 1e-6)
- [ ] AC-024: Fallback automático para cosine se Sinkhorn não converge
- [ ] AC-025: Cost matrix configurável (euclidean, cosine, custom)
- [ ] AC-026: Suporte a distribuições de dimensões diferentes (padding com zeros)
- [ ] AC-027: Latência < 20ms (versão Sinkhorn aproximada, dim ≤ 128)
- [ ] AC-028: Integração como métrica no Retrieval Engine (FT-001)
- [ ] AC-029: Integração como custo de aresta no ACO (FT-013)

---

## 3. Definition of Done (DoD)

- [ ] Wasserstein distance exata implementada (baseline/referência)
- [ ] Sinkhorn distance (aproximada) implementada para produção
- [ ] Entropic regularization com ε configurável
- [ ] Propriedades matemáticas validadas (simetria, identidade, triângulo)
- [ ] Fallback para cosine funcional
- [ ] Integração Retrieval (cosine → OT ranking)
- [ ] Integração ACO (custo de aresta)
- [ ] Integração ECMA (context drift measurement)
- [ ] Integração Immune (anomaly via distribution shift)
- [ ] Benchmark: Sinkhorn < 20ms para dim ≤ 128
- [ ] Testes unitários ≥ 80% cobertura
- [ ] Sem `unwrap()` em código de produção

---

## 4. Exemplos de Uso

### 4.1 Distância básica entre contextos

**Entrada:**
```json
{
  "ctx_a": [0.2, 0.8],
  "ctx_b": [0.6, 0.4],
  "method": "sinkhorn",
  "epsilon": 0.01
}
```

**Saída:**
```json
{
  "wasserstein_distance": 0.32,
  "method": "sinkhorn",
  "iterations": 23,
  "converged": true,
  "computation_ms": 1.2
}
```

### 4.2 Contextos idênticos

**Entrada:**
```json
{
  "ctx_a": [0.5, 0.3, 0.2],
  "ctx_b": [0.5, 0.3, 0.2]
}
```

**Saída:**
```json
{
  "wasserstein_distance": 0.0,
  "method": "sinkhorn",
  "iterations": 1,
  "converged": true
}
```

### 4.3 Retrieval híbrido (cosine → OT ranking)

**Pipeline:**
```json
{
  "query": [0.12, 0.85, 0.33, 0.67],
  "stage_1": {
    "method": "cosine",
    "top_k": 50,
    "latency_ms": 8.2,
    "purpose": "fast pre-filter"
  },
  "stage_2": {
    "method": "wasserstein_sinkhorn",
    "top_k": 10,
    "latency_ms": 14.5,
    "purpose": "precise re-ranking"
  },
  "final_results": [
    { "id": 42, "cosine_score": 0.91, "ot_distance": 0.08, "final_rank": 1 },
    { "id": 17, "cosine_score": 0.95, "ot_distance": 0.15, "final_rank": 2 }
  ],
  "note": "id=17 had higher cosine but worse OT → OT captured structural difference"
}
```

### 4.4 ECMA — Context Drift Measurement

**Entrada:**
```json
{
  "node_id": 42,
  "context_t0": [0.2, 0.8, 0.0],
  "context_t1": [0.3, 0.6, 0.1],
  "context_t2": [0.5, 0.3, 0.2]
}
```

**Saída:**
```json
{
  "drift": [
    { "from": "t0", "to": "t1", "distance": 0.18 },
    { "from": "t1", "to": "t2", "distance": 0.27 },
    { "from": "t0", "to": "t2", "distance": 0.42 }
  ],
  "drift_rate": 0.21,
  "trend": "DIVERGING",
  "maturity_impact": -0.05
}
```

### 4.5 Immune — Anomaly via Distribution Shift

**Entrada:**
```json
{
  "baseline_distribution": [0.2, 0.5, 0.3],
  "current_distribution": [0.8, 0.1, 0.1],
  "threshold": 0.4
}
```

**Saída:**
```json
{
  "ot_distance": 0.52,
  "is_anomaly": true,
  "detail": "Distribution shift exceeds threshold (0.52 > 0.40)",
  "action": "ESCALATE_TO_IMMUNE_DETECTION"
}
```

### 4.6 Erro — Não convergência → fallback

```json
{
  "error": "SINKHORN_NOT_CONVERGED",
  "iterations": 100,
  "residual": 0.05,
  "fallback": "cosine",
  "cosine_distance": 0.23,
  "warning": "Sinkhorn did not converge; using cosine fallback"
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Entrada | Saída Esperada |
|----|------|---------|----------------|
| UT-001 | Identidade | `W(a, a)` | `0.0` |
| UT-002 | Simetria | `W(a, b)` vs `W(b, a)` | iguais |
| UT-003 | Não-negatividade | qualquer par | `≥ 0.0` |
| UT-004 | Triângulo | `W(a,c)` vs `W(a,b) + W(b,c)` | `W(a,c) ≤ sum` |
| UT-005 | Monotonicidade | a próximo de b, c distante | `W(a,b) < W(a,c)` |
| UT-006 | Sinkhorn convergência | ε=0.01, dim=4 | converge < 100 iter |
| UT-007 | Sinkhorn vs exato | dim=4 | diferença < 5% |
| UT-008 | Epsilon alto (ε=1.0) | qualquer | mais blur, mais rápido |
| UT-009 | Epsilon baixo (ε=0.001) | qualquer | mais preciso, mais lento |
| UT-010 | Dims diferentes | dim=3 vs dim=5 | padding + resultado válido |
| UT-011 | Distribuições uniformes | [0.25, 0.25, 0.25, 0.25] x2 | `0.0` |
| UT-012 | Distribuições Dirac | [1,0,0] vs [0,0,1] | distância máxima |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | Distribuição com negativos | `Err(InvalidDistribution)` |
| UF-002 | Distribuição não soma 1 | `Err(NotNormalized)` ou auto-normalize |
| UF-003 | Vetor vazio | `Err(EmptyVector)` |
| UF-004 | NaN em distribuição | `Err(InvalidValue)` |
| UF-005 | ε = 0 | `Err(InvalidEpsilon)` — divisão por zero |
| UF-006 | Sinkhorn não converge | Fallback cosine + warning |

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasserstein_identity() {
        let a = vec![0.2, 0.8];
        assert!((wasserstein(&a, &a) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_wasserstein_symmetry() {
        let a = vec![0.2, 0.8];
        let b = vec![0.6, 0.4];
        assert!((wasserstein(&a, &b) - wasserstein(&b, &a)).abs() < 1e-10);
    }

    #[test]
    fn test_triangle_inequality() {
        let a = vec![0.1, 0.9];
        let b = vec![0.5, 0.5];
        let c = vec![0.9, 0.1];
        assert!(wasserstein(&a, &c) <= wasserstein(&a, &b) + wasserstein(&b, &c) + 1e-10);
    }

    #[test]
    fn test_sinkhorn_convergence() {
        let a = vec![0.2, 0.3, 0.5];
        let b = vec![0.4, 0.4, 0.2];
        let result = sinkhorn(&a, &b, 0.01, 100);
        assert!(result.converged);
        assert!(result.iterations < 100);
    }

    #[test]
    fn test_sinkhorn_fallback() {
        let a = vec![0.5, 0.5];
        let b = vec![0.5, 0.5];
        let result = sinkhorn_with_fallback(&a, &b, 0.0001, 5); // max 5 iter
        // Should fallback to cosine if not converged
        assert!(result.distance.is_finite());
    }
}
```

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | Retrieval híbrido: cosine top-50 → OT re-rank top-10 | OT re-ranking melhora precision@10 ≥ 5% |
| FT-002 | Comparar 100 pares de clusters | OT separa clusters melhor que cosine |
| FT-003 | Ordenação semântica validada | Ranking OT correlaciona com human judgment |
| FT-004 | Performance dim=128, 50 pares | Sinkhorn < 20ms total |
| FT-005 | Context drift 1000 timesteps | Drift rate calculado corretamente |
| FT-006 | Fallback sob carga | Cosine ativa quando Sinkhorn timeout |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | OT → Retrieval (FT-001) | Re-ranking pós-cosine funcional |
| IT-002 | OT → ECMA (FT-003) | Context drift mede evolução real |
| IT-003 | OT → Immune Detection (FT-017) | Distribution shift detecta anomalias |
| IT-004 | OT → ACO Pheromone (FT-013) | Custo de aresta = Wasserstein |
| IT-005 | OT → Pipeline (FT-011) | Métrica OT disponível no pipeline |
| IT-006 | OT → Métricas (FT-007) | `kce_ot_latency_ms`, `kce_ot_fallbacks` |

---

## 6. Formato CARE

### Context
A maioria dos sistemas de retrieval usa cosine similarity, que mede apenas o ângulo entre vetores. Isso ignora a **estrutura distribuída** do contexto. Optimal Transport (Wasserstein distance) captura o custo mínimo de transformar uma distribuição semântica em outra, revelando multimodalidade, deslocamento semântico e evolução de contexto que cosine não consegue detectar.

### Assumptions
- Contextos são representáveis como distribuições de probabilidade (normalizados, não-negativos)
- Sinkhorn (regularizado) é suficiente para produção — OT exato apenas como baseline/referência
- Entropic regularization com ε=0.01 fornece bom tradeoff precisão/velocidade
- Cost matrix padrão é euclidiana; cosine disponível como alternativa
- Convergência Sinkhorn em < 100 iterações para maioria dos inputs
- Fallback para cosine é aceitável quando Sinkhorn não converge
- GPU acceleration é roadmap futuro (não MVP)

### Requirements
- R-001: Wasserstein distance exata (referência/testes)
- R-002: Sinkhorn distance (produção) com ε configurável
- R-003: Propriedades métricas validadas (simetria, identidade, triângulo)
- R-004: Latência < 20ms (Sinkhorn, dim ≤ 128)
- R-005: Fallback automático para cosine
- R-006: Integração Retrieval (re-ranking)
- R-007: Integração ECMA (context drift)
- R-008: Integração Immune (distribution shift)
- R-009: Integração ACO (edge cost)
- R-010: Cost matrix configurável

### Evidence
- `benches/ot_bench.rs` — benchmark Sinkhorn vs exato vs cosine
- `tests/ot_properties.rs` — validação de propriedades métricas
- `reports/ot_vs_cosine.md` — análise comparativa de precision
- `reports/sinkhorn_convergence.md` — análise de convergência por ε

---

## 7. Critérios de Aceitação Não Funcionais

### Desempenho

| Métrica | Alvo | Método |
|---------|------|--------|
| Sinkhorn latência (dim=4) | < 1ms | Benchmark |
| Sinkhorn latência (dim=128) | < 20ms | Benchmark |
| Sinkhorn latência (dim=512) | < 100ms | Benchmark |
| Exato latência (dim=128) | < 500ms | Benchmark (ref only) |
| Re-ranking 50 candidatos | < 50ms | Benchmark |
| Convergência Sinkhorn | < 100 iterações | Teste |
| Fallback rate | < 5% | Monitoring |

### Segurança
- Inputs validados (não-negativos, finitos, normalizados)
- Sem buffer overflow na cost matrix (bounds checking)
- ε validado (> 0, < 10)

### Acessibilidade
- N/A (módulo backend)

---

## 8. Critérios de Qualidade e Métricas

### Métricas de Sucesso

| Métrica | Valor Alvo |
|---------|------------|
| Precision@10 melhoria vs cosine-only | ≥ 5% |
| Sinkhorn vs exato erro relativo | < 5% (ε=0.01) |
| Propriedades métricas | 100% validadas |
| Context drift correlation com evolução real | ≥ 0.80 |
| Cobertura de testes | ≥ 80% |

### Critérios de Falha
- Propriedade métrica violada (simetria, triângulo) → **BLOQUEANTE**
- Sinkhorn latência > 50ms (dim ≤ 128) → **BLOQUEANTE**
- Precision@10 pior que cosine-only → **BLOQUEANTE**
- Erro Sinkhorn vs exato > 20% → **BLOQUEANTE**
- Fallback rate > 20% → **WARNING**

---

## 9. Compatibilidade e Dependências

### Dependências Diretas

| Crate | Versão | Propósito |
|-------|--------|-----------|
| `ndarray` | ^0.15 | Matrizes para cost matrix e transport plan |
| `parking_lot` | ^0.12 | RwLock |

### Dependências Internas

| Módulo | Tipo de Integração |
|--------|--------------------|
| Retrieval (FT-001) | Re-ranking métrica (cosine → OT) |
| ECMA (FT-003) | Context drift measurement |
| ACO Pheromone (FT-013) | Edge cost = Wasserstein |
| Immune Detection (FT-017) | Distribution shift anomaly |
| Pipeline (FT-011) | Métrica disponível no pipeline |
| Métricas (FT-007) | `kce_ot_*` expostas |

### Compatibilidade

| Item | Requisito |
|------|-----------|
| Rust | ≥ 1.75 stable |
| OS | Linux (prod), macOS (dev) |
| CPU | x86_64 (SIMD para matrix ops futuro) |

---

## 10. Critérios de Rastreabilidade

### Histórico de Mudanças

| Data | Versão | Descrição | Autor |
|------|--------|-----------|-------|
| 2026-05-23 | v1.0 | Criação da especificação | Product Specialist |

### IDs de Artefatos Relacionados

| Tipo | ID | Descrição |
|------|----|-----------|
| Spec | FT-023-OPTIMAL-TRANSPORT | Esta especificação |
| Código | `src/metrics/optimal_transport.rs` | Implementação OT |
| Código | `src/metrics/sinkhorn.rs` | Sinkhorn regularizado |
| Código | `src/metrics/cost_matrix.rs` | Matrizes de custo |
| Bench | `benches/ot_bench.rs` | Benchmark |
| Teste | `tests/ot_properties.rs` | Propriedades métricas |
| Teste | `tests/ot_integration.rs` | Integração |
| Deps | FT-001, FT-003, FT-013, FT-017 | Features integradas |

---

## 11. Entrega e Critérios de Aceitação do MVP — Roadmap

### MVP (Semana 1-2)

| Item | Critério de Aceite |
|------|--------------------|
| Wasserstein 1D exato | Distância exata para distribuições 1D (sorting-based, O(n log n)) |
| Sinkhorn distance | Implementação básica com ε fixo (0.01) |
| Cost matrix euclidiana | Funcional para dims ≤ 128 |
| Propriedades métricas | Simetria, identidade, não-negatividade validadas |
| 8+ testes unitários | Passando (incluindo propriedades) |
| API interna | `fn wasserstein(a, b) -> f64` + `fn sinkhorn(a, b, eps) -> SinkhornResult` |

**Saída:** OT funcional como biblioteca interna de métricas.

---

### Iteração 1 (Semana 3-4)

| Item | Critério de Aceite |
|------|--------------------|
| Entropic regularization | ε configurável com validação |
| Convergência adaptativa | Threshold + max_iter configuráveis |
| Fallback para cosine | Automático quando Sinkhorn não converge |
| Cost matrix customizável | Euclidean, cosine, custom fn |
| Integração Retrieval | cosine top-50 → OT re-rank top-10 |
| Benchmark | Sinkhorn < 20ms (dim ≤ 128) |
| Desigualdade triangular | Validada numericamente |

**Saída:** OT integrado no Retrieval como re-ranking metric.

---

### Iteração 2 (Semana 5-6)

| Item | Critério de Aceite |
|------|--------------------|
| Integração ECMA | Context drift measurement funcional |
| Integração Immune | Distribution shift → anomaly detection |
| Integração ACO | Edge cost = Wasserstein |
| Batch computation | Múltiplos pares em paralelo (Rayon) |
| Métricas Prometheus | `kce_ot_latency_ms`, `kce_ot_fallbacks`, `kce_ot_convergence_iter` |
| Precision@10 benchmark | ≥ 5% melhoria vs cosine-only |
| Dims ≤ 512 | Sinkhorn < 100ms |
| Documentação completa | API docs + mathematical reference + examples |

**Saída:** OT production-ready integrado em 4 módulos core do KCE.

---

### Futuro (Backlog)

| Item | Descrição |
|------|-----------|
| GPU acceleration | cuML / CUDA Sinkhorn para dims altas |
| Sliced Wasserstein | Projeção 1D para approximação ultra-rápida |
| Wasserstein barycenter | Centro de massa de múltiplos contextos |
| Online OT | Streaming Sinkhorn para updates incrementais |

---

*Documento gerado em 2026-05-23 — KineContext Engine Product Specification*
