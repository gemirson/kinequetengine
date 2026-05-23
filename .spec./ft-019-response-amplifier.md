# FT-019 — Immune Response Amplifier

**Módulo:** AIS (Artificial Immune System) | **Versão:** v6.0 | **Prioridade:** P1 — Alto  
**Artefato ID:** FT-019-RESPONSE-AMPLIFIER | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O Immune Response Amplifier **aumenta automaticamente o peso e prioridade de decisões críticas**. Quando contexto "perigoso" é detectado (anomalia, antigen match, cenário de alta severidade), o amplifier escala a prioridade de processamento, aloca mais recursos e reduz thresholds de ação. Inspirado na amplificação clonal do sistema imunológico.

### Valor
- Decisões mais rápidas em cenários críticos
- Alocação dinâmica de prioridade sem regras manuais
- Amplificação proporcional à severidade

---

## 2. Critérios de Aceite (AC)

- [ ] AC-010: `amplify(context, trigger)` retorna `AmplifiedContext` com prioridade ajustada
- [ ] AC-011: Amplificação proporcional à severidade: LOW=1.5x, MEDIUM=2x, HIGH=3x, CRITICAL=5x
- [ ] AC-012: Triggers suportados: anomaly_detection, antigen_match, manual_escalation
- [ ] AC-013: Amplificação afeta: prioridade MCE, timeout (estendido), retry count (aumentado)
- [ ] AC-014: Decay de amplificação após resolução (não permanece indefinidamente)
- [ ] AC-015: Máximo de amplificação = 5x (cap para evitar resource starvation)
- [ ] AC-016: Log detalhado de cada amplificação para auditoria
- [ ] AC-017: Métricas: `kce_amplifications_total`, `kce_amplification_avg_factor`

---

## 3. Definition of Done (DoD)

- [ ] `amplify()` funcional com 4 níveis de severidade
- [ ] Triggers integrados (detection, antigen, manual)
- [ ] Decay pós-resolução
- [ ] Cap de amplificação (5x)
- [ ] Métricas e audit log
- [ ] Cobertura ≥ 80%

---

## 4. Exemplos de Uso

### Amplificação por anomalia

**Entrada:**
```json
{
  "context": { "request_id": "req_abc", "priority": 5, "timeout_ms": 100, "retries": 3 },
  "trigger": { "type": "anomaly_detection", "severity": "HIGH", "anomaly_score": 0.89 }
}
```

**Saída (contexto amplificado):**
```json
{
  "amplified_context": {
    "request_id": "req_abc",
    "priority": 15,
    "timeout_ms": 300,
    "retries": 9,
    "amplification_factor": 3.0,
    "trigger": "anomaly_detection",
    "decay_after_ms": 30000
  },
  "audit": {
    "original_priority": 5,
    "amplified_priority": 15,
    "reason": "HIGH severity anomaly (score: 0.89)"
  }
}
```

### Amplificação por antigen match
```json
{
  "trigger": { "type": "antigen_match", "severity": "CRITICAL", "antigen_id": "ag_001" },
  "amplified_context": {
    "priority": 25,
    "amplification_factor": 5.0,
    "action": "BLOCK_AND_ALERT"
  }
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Saída Esperada |
|----|------|----------------|
| UT-001 | Amplificação LOW | factor = 1.5x |
| UT-002 | Amplificação HIGH | factor = 3.0x |
| UT-003 | Amplificação CRITICAL | factor = 5.0x |
| UT-004 | Cap respeitado | factor ≤ 5.0x mesmo com múltiplos triggers |
| UT-005 | Decay após resolução | priority retorna ao original |
| UT-006 | Priority ajustada | `priority * factor` |
| UT-007 | Timeout estendido | `timeout * factor` |
| UT-008 | Retries aumentados | `retries * factor` |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | Severity inválida | `Err(InvalidSeverity)` |
| UF-002 | Trigger desconhecido | `Err(UnknownTrigger)` |
| UF-003 | Contexto nulo | `Err(EmptyContext)` |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado |
|----|---------|-----------|
| FT-001 | Anomalia → amplificação → resposta rápida | Latência de decisão reduzida |
| FT-002 | Múltiplos triggers simultâneos | Cap 5x respeitado |
| FT-003 | Decay após 30s | Priority retorna ao original |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Detection (FT-017) → Amplifier → MCE | MCE processa com prioridade elevada |
| IT-002 | Antigen (FT-018) → Amplifier → Pipeline | Pipeline escala recursos |
| IT-003 | Amplifier → Métricas | Amplificações registradas |

---

## 6. Formato CARE

**Context:** Em cenários críticos (fraude, ataque), resposta normal é lenta demais. O Amplifier escala automaticamente prioridade e recursos, inspirado na amplificação clonal imunológica.

**Assumptions:** 4 níveis de severidade são suficientes; cap de 5x previne resource starvation; decay é temporal (30s default); amplificação afeta priority, timeout e retries.

**Requirements:** R-001: amplify() com 4 severidades | R-002: 3 triggers | R-003: Cap 5x | R-004: Decay | R-005: Métricas + audit | R-006: Integração FT-017/FT-018.

**Evidence:** `tests/amplifier_tests.rs`

---

## 7–11. (Resumo)

**Não funcionais:** Overhead amplificação < 0.5ms | Cap 5x nunca violado | Decay accuracy ± 1s  
**Qualidade:** Amplificação correta 100% | Cap sempre respeitado | Cobertura ≥ 80%  
**Falha (BLOQUEANTE):** Cap violado | Amplificação não decai | Priority overflow  
**Deps:** Detection (FT-017), Antigen (FT-018), MCE (FT-004), Pipeline (FT-011)  
**Rastreabilidade:** FT-019-RESPONSE-AMPLIFIER | `src/ais/amplifier.rs`

**Roadmap:** MVP: amplify() com factor fixo por severity | Iter1: 3 triggers + decay + cap | Iter2: Integração completa + métricas + audit log

---

*Documento gerado em 2026-05-23 — KineContext Engine Product Specification*
