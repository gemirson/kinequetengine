# FT-017 — Immune Detection Engine (Anomaly Recognition)

**Módulo:** AIS (Artificial Immune System) | **Versão:** v6.0 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-017-IMMUNE-DETECTION | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O Immune Detection Engine implementa **detecção de anomalias** inspirada no sistema imunológico biológico. Compara padrões de entrada com uma baseline de comportamento "normal" (self). Padrões que divergem significativamente são classificados como "antígenos" (anomalias). Fornece antifraude natural e detecção de comportamento fora do padrão sem regras fixas codificadas.

### Valor
- Antifraude natural — detecta padrões anômalos sem regras hardcoded
- Detecção de comportamento fora do padrão em tempo real
- Aprendizado contínuo — baseline evolui com o tempo

---

## 2. Critérios de Aceite (AC)

- [ ] AC-010: `detect(input)` retorna `AnomalyResult { is_anomaly: bool, score: f64, details: Vec<Deviation> }`
- [ ] AC-011: Baseline construída a partir de N primeiras interações (warm-up configurável)
- [ ] AC-012: Anomaly score = distância normalizada da baseline (0.0 = normal, 1.0 = anômalo)
- [ ] AC-013: Threshold de detecção configurável (default: 0.7)
- [ ] AC-014: Múltiplas dimensões monitoradas: latência, distribuição de features, frequência, padrão temporal
- [ ] AC-015: False positive rate < 5% após warm-up
- [ ] AC-016: Detecção em tempo real (< 2ms overhead por request)
- [ ] AC-017: Baseline atualizada incrementalmente (sliding window)
- [ ] AC-018: Anomalias geram evento para Antigen Memory (FT-018)

---

## 3. Definition of Done (DoD)

- [ ] Algoritmo de detecção implementado (distance-based ou negative selection)
- [ ] Baseline com warm-up e sliding window
- [ ] Threshold configurável
- [ ] Integração com pipeline para detecção inline
- [ ] False positive rate validado < 5%
- [ ] Testes unitários ≥ 80%

---

## 4. Exemplos de Uso

### Detecção de anomalia

**Input normal:**
```json
{
  "request": {
    "query_vector": [0.12, 0.85, 0.33, 0.67],
    "tenant_id": "bank_01",
    "source_ip": "10.0.1.50",
    "request_frequency": 12
  }
}
```

**Resultado (normal):**
```json
{
  "is_anomaly": false,
  "anomaly_score": 0.15,
  "threshold": 0.7,
  "deviations": []
}
```

**Input anômalo (fraude potencial):**
```json
{
  "request": {
    "query_vector": [0.99, 0.01, 0.99, 0.01],
    "tenant_id": "bank_01",
    "source_ip": "185.220.101.1",
    "request_frequency": 500
  }
}
```

**Resultado (anomalia detectada):**
```json
{
  "is_anomaly": true,
  "anomaly_score": 0.89,
  "threshold": 0.7,
  "deviations": [
    { "dimension": "query_vector", "deviation": 0.82, "detail": "Distribution shift detected" },
    { "dimension": "request_frequency", "deviation": 0.95, "detail": "41x above baseline mean" },
    { "dimension": "source_ip", "deviation": 0.70, "detail": "New IP, known Tor exit node" }
  ],
  "action": "FLAG_FOR_REVIEW",
  "antigen_created": true
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Saída Esperada |
|----|------|----------------|
| UT-001 | Input normal | `is_anomaly: false`, score < 0.7 |
| UT-002 | Input anômalo (frequência alta) | `is_anomaly: true`, score > 0.7 |
| UT-003 | Input anômalo (vetor distante) | `is_anomaly: true` |
| UT-004 | Warm-up period | Detecção desabilitada durante warm-up |
| UT-005 | Baseline sliding window | Baseline atualiza com novos dados |
| UT-006 | Threshold edge case | score = threshold → configurable behavior |
| UT-007 | Múltiplas dimensões | Deviations por dimensão corretas |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | Baseline vazia (pré warm-up) | `Ok(skip)` — sem detecção |
| UF-002 | Threshold > 1.0 | `Err(InvalidThreshold)` |
| UF-003 | Input com NaN | `Err(InvalidInput)` |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado |
|----|---------|-----------|
| FT-001 | 1000 requests normais → 10 anômalos | 10 detectados, FPR < 5% |
| FT-002 | Detecção inline no pipeline | Overhead < 2ms |
| FT-003 | Baseline evolui com 10k requests | FPR decresce ao longo do tempo |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Detection → Antigen Memory (FT-018) | Anomalia cria antigen |
| IT-002 | Detection → Pipeline | Pipeline flagga request anômalo |
| IT-003 | Detection → Métricas | `kce_anomalies_detected` registrado |
| IT-004 | Detection → Response Amplifier (FT-019) | Anomalia amplifica prioridade |

---

## 6. Formato CARE

**Context:** Sistemas de produção financeira precisam detectar fraude e comportamento anômalo em tempo real. Regras fixas são frágeis e facilmente contornadas. Detecção imunológica compara com baseline aprendida, adaptando-se continuamente.

**Assumptions:** Baseline é construída com warm-up de 1000+ requests; sliding window de 10k requests; anomaly score é distância normalizada; múltiplas dimensões são monitoradas independentemente; false positive rate alvo < 5%.

**Requirements:** R-001: detect() inline | R-002: Baseline sliding window | R-003: Multi-dimensional | R-004: Threshold configurável | R-005: FPR < 5% | R-006: < 2ms overhead | R-007: Integração Antigen Memory.

**Evidence:** `tests/immune_detection_tests.rs` | `reports/fpr_analysis.md`

---

## 7–11. (Resumo)

**Não funcionais:** Overhead < 2ms | FPR < 5% | True Positive Rate > 90% | Memória baseline < 50MB  
**Qualidade:** FPR < 5% | TPR > 90% | Baseline estável após warm-up | Cobertura ≥ 80%  
**Falha (BLOQUEANTE):** FPR > 10% | TPR < 70% | Overhead > 10ms | Baseline corrompe  
**Deps:** Pipeline (FT-011), Antigen Memory (FT-018), Métricas (FT-007)  
**Rastreabilidade:** FT-017-IMMUNE-DETECTION | `src/ais/detection.rs`

**Roadmap:** MVP: detect() com distância euclidiana + threshold fixo | Iter1: Multi-dimensional + sliding window + baseline auto | Iter2: Integração Antigen Memory + pipeline inline + FPR tuning

---

*Documento gerado em 2026-05-23 — KineContext Engine Product Specification*
