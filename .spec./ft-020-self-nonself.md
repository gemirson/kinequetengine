# FT-020 — Self vs Non-Self Context Classifier

**Módulo:** AIS (Artificial Immune System) | **Versão:** v6.0 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-020-SELF-NONSELF | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O Self vs Non-Self Classifier **separa contexto confiável (self) de contexto externo/suspeito (non-self)** usando classificação binária + probabilística. Inspirado na capacidade do sistema imunológico de distinguir células próprias de invasores. Fornece segurança adaptativa sem regras fixas — o sistema aprende o que é "self" a partir do comportamento normal.

### Valor
- Segurança sem regras fixas — aprende padrões confiáveis
- Classificação probabilística (não binária absoluta)
- Adaptação contínua a novos padrões legítimos

---

## 2. Critérios de Aceite (AC)

- [ ] AC-010: `classify(input)` retorna `Classification { label: Self|NonSelf, confidence: f64, features: Vec<FeatureScore> }`
- [ ] AC-011: Self profile construído via negative selection algorithm (NSA)
- [ ] AC-012: Classificação probabilística (confidence 0.0..1.0), não apenas binária
- [ ] AC-013: Self profile atualizado incrementalmente com novos padrões válidos
- [ ] AC-014: Non-self threshold configurável (default: 0.6)
- [ ] AC-015: Features monitoradas: vector distribution, temporal pattern, tenant behavior, API usage
- [ ] AC-016: Classificação < 2ms por request
- [ ] AC-017: Self profile persistido no KineSQL
- [ ] AC-018: Integração com Immune Detection (FT-017) para cross-validation

---

## 3. Definition of Done (DoD)

- [ ] Negative selection algorithm implementado
- [ ] Self profile com aprendizado incremental
- [ ] Classificação probabilística funcional
- [ ] Persistência KineSQL
- [ ] Integração FT-017
- [ ] Cobertura ≥ 80%

---

## 4. Exemplos de Uso

### Classificação Self
```json
{
  "input": { "vector": [0.12, 0.85, 0.33], "tenant": "bank_01", "hour": 14, "endpoint": "/query" },
  "result": {
    "label": "SELF",
    "confidence": 0.92,
    "features": [
      { "name": "vector_distribution", "score": 0.95, "status": "NORMAL" },
      { "name": "temporal_pattern", "score": 0.88, "status": "NORMAL" },
      { "name": "tenant_behavior", "score": 0.93, "status": "NORMAL" }
    ]
  }
}
```

### Classificação Non-Self
```json
{
  "input": { "vector": [0.99, 0.01, 0.99], "tenant": "bank_01", "hour": 3, "endpoint": "/action" },
  "result": {
    "label": "NON_SELF",
    "confidence": 0.81,
    "features": [
      { "name": "vector_distribution", "score": 0.25, "status": "ANOMALOUS" },
      { "name": "temporal_pattern", "score": 0.35, "status": "ANOMALOUS" },
      { "name": "tenant_behavior", "score": 0.78, "status": "NORMAL" }
    ],
    "recommended_action": "ESCALATE_TO_IMMUNE_DETECTION"
  }
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Saída Esperada |
|----|------|----------------|
| UT-001 | Input normal → Self | `label: SELF`, confidence > 0.8 |
| UT-002 | Input anômalo → Non-Self | `label: NON_SELF`, confidence > 0.6 |
| UT-003 | Confidence range | sempre 0.0..1.0 |
| UT-004 | Self profile update | novo padrão válido incorporado |
| UT-005 | NSA — detectors gerados | detectors não matcheiam self |
| UT-006 | Threshold edge case | score = threshold → configurable |
| UT-007 | Multi-feature scoring | scores por feature corretos |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | Self profile vazio | `Err(NoSelfProfile)` |
| UF-002 | Input com dims incompatíveis | `Err(DimensionMismatch)` |
| UF-003 | Threshold > 1.0 | `Err(InvalidThreshold)` |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado |
|----|---------|-----------|
| FT-001 | 1000 self + 50 non-self | Accuracy > 90% |
| FT-002 | Self profile evolui | Novos padrões legítimos aceitos |
| FT-003 | Classificação inline | Overhead < 2ms |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Classifier → Detection (FT-017) | Non-self escalado para detecção |
| IT-002 | Classifier → Pipeline | Pipeline adapta fluxo por classificação |
| IT-003 | Classifier → KineSQL | Self profile persiste |

---

## 6. Formato CARE

**Context:** Regras de segurança fixas são frágeis. O classificador Self/Non-Self aprende automaticamente padrões confiáveis e identifica contexto externo/suspeito de forma probabilística, adaptando-se continuamente.

**Assumptions:** Negative Selection Algorithm é adequado para MVP; self profile construído em warm-up (1000+ samples); classificação é probabilística; features são independentes; self evolui incrementalmente.

**Requirements:** R-001: classify() probabilístico | R-002: NSA para self profile | R-003: Incremental learning | R-004: < 2ms | R-005: Persistência | R-006: Multi-feature | R-007: Integração FT-017.

**Evidence:** `tests/self_nonself_tests.rs` | `reports/classifier_accuracy.md`

---

## 7–11. (Resumo)

**Não funcionais:** Classificação < 2ms | Accuracy > 90% | Memória self profile < 20MB  
**Qualidade:** Accuracy > 90% | 0 false negatives para ameaças conhecidas | Cobertura ≥ 80%  
**Falha (BLOQUEANTE):** Accuracy < 75% | Overhead > 10ms | Self profile corrompe  
**Deps:** Detection (FT-017), KineSQL (FT-005), Pipeline (FT-011)  
**Rastreabilidade:** FT-020-SELF-NONSELF | `src/ais/classifier.rs`

**Roadmap:** MVP: classify() com threshold fixo + euclidean distance | Iter1: NSA + multi-feature + incremental | Iter2: Persistência + integração + accuracy tuning

---

*Documento gerado em 2026-05-23 — KineContext Engine Product Specification*
