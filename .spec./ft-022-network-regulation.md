# FT-022 — Immune Network Regulation

**Módulo:** AIS (Artificial Immune System) | **Versão:** v6.0 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-022-NETWORK-REGULATION | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O Immune Network Regulation é o **controlador de equilíbrio global** do sistema KCE. Regula a interação entre todos os módulos AIS e ACO, evitando hiperatividade (overfitting), sub-atividade (perda de sensibilidade) e oscilações descontroladas. Inspirado na teoria da rede imunológica de Jerne — anticorpos regulam uns aos outros para manter homeostase.

### Valor
- Estabilidade sistêmica — previne oscilações e cascatas
- Evita hiperatividade (overfitting / excesso de amplificação)
- Regula interação entre módulos automaticamente
- Homeostase: sistema mantém equilíbrio sem intervenção manual

---

## 2. Critérios de Aceite (AC)

### Gerais
- [ ] AC-001: Compila sem warnings em `--release`
- [ ] AC-002: Thread-safe para monitoramento concorrente

### Específicos
- [ ] AC-010: `regulate()` monitora métricas de todos módulos AIS/ACO e aplica ajustes
- [ ] AC-011: Detecta hiperatividade: amplification_rate > threshold → reduz sensitivity
- [ ] AC-012: Detecta sub-atividade: detection_rate < min → aumenta sensitivity
- [ ] AC-013: Regula taxa de mutação: convergência alta → aumenta rate; divergência → diminui
- [ ] AC-014: Regula evaporação: stagnation → aumenta ρ; volatilidade → diminui ρ
- [ ] AC-015: Feedback loop: métricas → regulação → ajuste → nova medição
- [ ] AC-016: Intervalo de regulação configurável (default: 30s)
- [ ] AC-017: Limites de regulação (guardrails) — nenhum parâmetro sai de range safe
- [ ] AC-018: Dashboard de status de regulação (JSON endpoint `/regulation/status`)
- [ ] AC-019: Log detalhado de cada ajuste para auditoria

### Guardrails (limites safe)
| Parâmetro | Min | Max | Default |
|-----------|-----|-----|---------|
| detection_threshold | 0.3 | 0.95 | 0.7 |
| amplification_cap | 2.0 | 10.0 | 5.0 |
| mutation_rate | 0.01 | 0.3 | 0.05 |
| evaporation_rho | 0.01 | 0.5 | 0.1 |
| ant_count | 2 | 20 | 5 |

---

## 3. Definition of Done (DoD)

- [ ] `regulate()` funcional com monitoramento de métricas
- [ ] Detecção de hiperatividade e sub-atividade
- [ ] Ajuste automático de 5+ parâmetros
- [ ] Guardrails implementados e validados
- [ ] Feedback loop funcional
- [ ] Endpoint `/regulation/status`
- [ ] Testes de estabilidade
- [ ] Cobertura ≥ 80%

---

## 4. Exemplos de Uso

### Ciclo de regulação

**Métricas de entrada (coletadas):**
```json
{
  "metrics": {
    "anomaly_detection_rate": 0.35,
    "false_positive_rate": 0.12,
    "amplification_rate": 0.28,
    "mutation_success_rate": 0.08,
    "aco_convergence_cycles": 5,
    "aco_stagnation_detected": false,
    "pheromone_avg": 3.2,
    "pheromone_variance": 0.8
  }
}
```

**Regulação aplicada:**
```json
{
  "adjustments": [
    {
      "parameter": "detection_threshold",
      "from": 0.7,
      "to": 0.65,
      "reason": "FPR (0.12) above target (0.05); lowering threshold to be more selective",
      "guardrail_check": "WITHIN_BOUNDS"
    },
    {
      "parameter": "mutation_rate",
      "from": 0.05,
      "to": 0.08,
      "reason": "Mutation success rate (0.08) low; increasing exploration",
      "guardrail_check": "WITHIN_BOUNDS"
    },
    {
      "parameter": "evaporation_rho",
      "from": 0.1,
      "to": 0.1,
      "reason": "ACO not stagnated; no adjustment needed",
      "guardrail_check": "NO_CHANGE"
    }
  ],
  "system_status": "BALANCED",
  "regulation_cycle": 42,
  "next_regulation_in_seconds": 30
}
```

### Status endpoint (GET /regulation/status)
```json
{
  "status": "BALANCED",
  "modules": {
    "immune_detection": { "health": "OK", "sensitivity": "NORMAL" },
    "antigen_memory": { "health": "OK", "antigens_active": 23 },
    "response_amplifier": { "health": "OK", "amplification_rate": "NORMAL" },
    "self_classifier": { "health": "OK", "accuracy": 0.93 },
    "mutation_engine": { "health": "TUNING", "rate_adjusted": true },
    "aco_colony": { "health": "OK", "convergence": "STABLE" }
  },
  "guardrails": { "violations": 0, "warnings": 1 },
  "last_regulation": "2026-05-23T14:00:00Z"
}
```

### Hiperatividade detectada
```json
{
  "alert": "HYPERACTIVITY_DETECTED",
  "module": "response_amplifier",
  "metric": "amplification_rate",
  "value": 0.45,
  "threshold": 0.30,
  "action": "REDUCE_SENSITIVITY",
  "adjustment": { "amplification_cap": { "from": 5.0, "to": 3.0 } }
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Saída Esperada |
|----|------|----------------|
| UT-001 | FPR alto → reduce threshold | threshold diminui |
| UT-002 | Detection rate baixo → increase sensitivity | threshold diminui |
| UT-003 | Stagnation ACO → increase ρ | evaporation_rho aumenta |
| UT-004 | Mutation success baixo → increase rate | mutation_rate aumenta |
| UT-005 | Guardrail min violado | parâmetro clamped a min |
| UT-006 | Guardrail max violado | parâmetro clamped a max |
| UT-007 | Sistema balanceado | sem ajustes aplicados |
| UT-008 | Feedback loop 10 ciclos | sistema converge para equilíbrio |
| UT-009 | Hiperatividade detectada | alert gerado + sensitivity reduzida |
| UT-010 | Sub-atividade detectada | sensitivity aumentada |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | Métricas indisponíveis | `Ok(skip)` — regulação adiada |
| UF-002 | Interval negativo | `Err(InvalidInterval)` |
| UF-003 | Guardrail invertido (min > max) | `Err(InvalidGuardrail)` |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado |
|----|---------|-----------|
| FT-001 | 100 ciclos de regulação | Sistema mantém equilíbrio |
| FT-002 | Injeção de anomalias massivas | Regulador estabiliza em < 10 ciclos |
| FT-003 | Endpoint /regulation/status | JSON válido com status de todos módulos |
| FT-004 | Guardrails nunca violados | 0 violations em 1000 ciclos |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Regulation → Detection (FT-017) | Threshold ajustado |
| IT-002 | Regulation → Amplifier (FT-019) | Cap ajustado |
| IT-003 | Regulation → Mutation (FT-021) | Rate ajustada |
| IT-004 | Regulation → Evaporation (FT-014) | ρ ajustado |
| IT-005 | Regulation → ACO (FT-015) | Ant count ajustado |
| IT-006 | Regulation → API | Endpoint /regulation/status funcional |
| IT-007 | Regulation → Métricas | `kce_regulation_adjustments` registrado |

---

## 6. Formato CARE

**Context:** Sem regulação global, módulos AIS e ACO podem entrar em ciclos viciosos: hiperatividade (tudo é anomalia), sub-atividade (nada detecta), ou oscilação (flip-flop entre estados). O Network Regulation implementa homeostase inspirada na teoria de rede imunológica de Jerne — módulos regulam uns aos outros para equilíbrio sistêmico.

**Assumptions:**
- Métricas de todos módulos AIS/ACO disponíveis via API interna
- Regulação periódica (30s) é suficiente — não precisa ser real-time
- Guardrails são limites hard — nunca violados
- Feedback loop converge em < 10 ciclos na maioria dos cenários
- Hiperatividade é mais perigosa que sub-atividade (false positives custam mais)

**Requirements:**
- R-001: regulate() com monitoramento global
- R-002: Detecção de hiperatividade e sub-atividade
- R-003: Ajuste automático de 5+ parâmetros
- R-004: Guardrails com limites hard
- R-005: Feedback loop convergente
- R-006: Endpoint /regulation/status
- R-007: Audit log de ajustes
- R-008: Integração com todos módulos AIS + ACO

**Evidence:**
- `tests/regulation_tests.rs`
- `tests/stability_tests.rs` — testes de convergência de feedback loop
- `reports/regulation_dynamics.md` — análise de estabilidade

---

## 7. Critérios de Aceitação Não Funcionais

| Aspecto | Alvo |
|---------|------|
| Latência regulate() | < 10ms |
| Overhead no sistema | < 1% throughput |
| Convergência feedback loop | < 10 ciclos |
| Guardrail violations | 0 (absoluto) |
| Memória adicional | < 5MB |
| Disponibilidade /regulation/status | 100% |

---

## 8. Critérios de Qualidade e Métricas

**Sucesso:**
- 0 guardrail violations em todos testes
- Feedback loop converge em < 10 ciclos (90%+ dos cenários)
- Sistema mantém equilíbrio por 7+ dias sob carga
- Cobertura ≥ 80%

**Falha (BLOQUEANTE):**
- Guardrail violado → sistema instável → **BLOQUEANTE**
- Feedback loop não converge em 50 ciclos → **BLOQUEANTE**
- Oscilação contínua (flip-flop > 20 ciclos) → **BLOQUEANTE**
- Endpoint /regulation/status indisponível → **BLOQUEANTE**

---

## 9. Compatibilidade e Dependências

### Dependências Internas (todos módulos regulados)
| Módulo | Parâmetros Regulados |
|--------|----------------------|
| Detection (FT-017) | detection_threshold |
| Antigen (FT-018) | expiry_days |
| Amplifier (FT-019) | amplification_cap |
| Classifier (FT-020) | nonself_threshold |
| Mutation (FT-021) | mutation_rate |
| Evaporation (FT-014) | evaporation_rho |
| Exploration (FT-015) | ant_count |

### Crates
| Crate | Versão | Propósito |
|-------|--------|-----------|
| `tokio` | ^1.35 | Timer periódico |
| `parking_lot` | ^0.12 | RwLock |
| `serde_json` | ^1.0 | Status endpoint |

---

## 10. Rastreabilidade

| Tipo | ID |
|------|----|
| Spec | FT-022-NETWORK-REGULATION |
| Código | `src/ais/regulation.rs`, `src/ais/guardrails.rs` |
| Teste | `tests/regulation_tests.rs`, `tests/stability_tests.rs` |
| API | `GET /regulation/status` |
| Deps | FT-014, FT-015, FT-017, FT-018, FT-019, FT-020, FT-021 |

---

## 11. Entrega e Critérios de Aceitação do MVP — Roadmap

### MVP (Semana 1-2)
| Item | Critério de Aceite |
|------|--------------------|
| `regulate()` básico | Monitora 2 métricas (FPR, detection_rate) |
| Guardrails hardcoded | 5 parâmetros com limites |
| Log de ajustes | Console output |
| 5+ testes unitários | Passando |

**Saída:** Regulação básica de 2 parâmetros com guardrails.

---

### Iteração 1 (Semana 3-4)
| Item | Critério de Aceite |
|------|--------------------|
| Monitoramento de 5+ métricas | Todos módulos AIS |
| Ajuste automático de 5 parâmetros | Detection, Amplifier, Mutation, Evaporation, Exploration |
| Feedback loop | Convergência em < 10 ciclos |
| Detecção hiper/sub-atividade | Alertas gerados |
| Endpoint /regulation/status | JSON funcional |

**Saída:** Regulação completa com feedback loop e status endpoint.

---

### Iteração 2 (Semana 5-6)
| Item | Critério de Aceite |
|------|--------------------|
| Integração com todos 7 módulos | Parâmetros regulados end-to-end |
| Testes de estabilidade 7 dias | Equilíbrio mantido |
| Guardrails dinâmicos | Ajustáveis por config |
| Audit trail completo | Histórico de ajustes |
| Métricas Prometheus | `kce_regulation_*` exportadas |
| Documentação | API docs + runbook operacional |

**Saída:** Network Regulation production-ready — homeostase sistêmica autônoma.

---

*Documento gerado em 2026-05-23 — KineContext Engine Product Specification*
