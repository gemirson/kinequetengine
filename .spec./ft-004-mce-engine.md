# FT-004 — MCE (mRNA Cognitive Engine)

**Módulo:** Core Engine | **Versão:** v5.9 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-004-MCE-ENGINE | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O MCE transforma **contexto semântico em instruções executáveis** (mRNA). Recebe nós enriquecidos do ECMA, codifica-os em payloads compactos com intent, prioridade e TTL, e executa a ação correspondente. É o estágio final do pipeline cognitivo — onde conhecimento vira ação.

---

## 2. Critérios de Aceite (AC)

- [ ] AC-010: Encoding transforma nós ECMA em struct `mRNA` com `intent`, `priority`, `ttl`, `payload`
- [ ] AC-011: Redução de payload > 50% vs input bruto
- [ ] AC-012: Execução de mRNA < 5ms
- [ ] AC-013: TTL é respeitado — mRNA expirado não executa
- [ ] AC-014: Prioridade calculada automaticamente baseada em maturidade do nó
- [ ] AC-015: Consistência de ação: mesma entrada → mesma ação
- [ ] AC-016: Feedback de resultado retroalimenta ECMA
- [ ] AC-017: Suporte a múltiplos intents (`risk_eval`, `data_enrich`, `alert`, `classify`)

---

## 3. Definition of Done (DoD)

- [ ] Encoding funcional com redução > 50%
- [ ] Execução funcional < 5ms
- [ ] TTL aplicado e validado
- [ ] Prioridade calculada automaticamente
- [ ] Feedback loop → ECMA
- [ ] Testes unitários ≥ 80%
- [ ] Sem `unwrap()` em produção

---

## 4. Exemplos de Uso

### Encoding + Execução

**Entrada (contexto ECMA):**
```json
{
  "nodes": [
    { "id": 42, "state": "Specialized", "maturity": 0.87, "data": { "type": "credit_risk", "value": 0.72 } }
  ],
  "context": "loan_evaluation"
}
```

**mRNA gerado:**
```json
{
  "intent": "risk_eval",
  "priority": 8,
  "ttl_ms": 5000,
  "payload": "0x03A2F1...",
  "payload_size_bytes": 64,
  "original_size_bytes": 156,
  "compression_ratio": 0.59
}
```

**Resultado da execução:**
```json
{
  "action": "risk_eval",
  "result": { "risk_score": 0.72, "recommendation": "APPROVE_WITH_CONDITIONS" },
  "execution_ms": 2.3,
  "feedback": { "success": true, "maturity_delta": 0.02 }
}
```

### Erro — mRNA expirado
```json
{ "error": "MRNA_EXPIRED", "message": "TTL exceeded (5000ms)", "code": 408 }
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Entrada | Saída Esperada |
|----|------|---------|----------------|
| UT-001 | Encoding gera intent correto | context="risk" | `mrna.intent == "risk_eval"` |
| UT-002 | Compression ratio > 50% | payload 156 bytes | output < 78 bytes |
| UT-003 | TTL válido | ttl=5000ms, age=1000ms | executa normalmente |
| UT-004 | TTL expirado | ttl=100ms, age=200ms | `Err(MrnaExpired)` |
| UT-005 | Prioridade por maturidade | maturity=0.87 | `priority >= 7` |
| UT-006 | Determinismo | mesma entrada 2x | mesma saída |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | Nó sem dados | `Err(EmptyPayload)` |
| UF-002 | Intent desconhecido | `Err(UnknownIntent)` |
| UF-003 | TTL = 0 | `Err(InvalidTTL)` |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | Pipeline completo: encode → execute | Ação final correta |
| FT-002 | Batch de 100 mRNAs | Todos executam < 5ms |
| FT-003 | mRNA com TTL curto sob carga | Expirados rejeitados corretamente |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | ECMA → MCE | Nós Specialized geram mRNA válido |
| IT-002 | MCE → ECMA (feedback) | Resultado retroalimenta maturidade |
| IT-003 | API → MCE | POST /encode + POST /action funcionais |

---

## 6. Formato CARE

**Context:** Último estágio do pipeline cognitivo; transforma conhecimento processado em ações executáveis via encoding compacto inspirado em biologia molecular (mRNA).

**Assumptions:** Nós de entrada são validados pelo ECMA; intents são finitos e conhecidos; TTL é definido pelo caller ou calculado por prioridade; execução é síncrona.

**Requirements:** R-001: Encoding com redução > 50% | R-002: Execução < 5ms | R-003: TTL enforcement | R-004: Priority automática | R-005: Feedback → ECMA | R-006: Determinismo.

**Evidence:** `tests/mce_tests.rs` | `benches/mce_bench.rs`

---

## 7. Critérios Não Funcionais

| Aspecto | Alvo |
|---------|------|
| Latência encoding | < 2ms |
| Latência execução | < 5ms |
| Compression ratio | > 50% |
| Throughput | ≥ 1000 mRNA/s |

---

## 8. Qualidade e Métricas

**Sucesso:** Compression > 50% | Execução < 5ms | Determinismo 100% | Cobertura ≥ 80%  
**Falha (BLOQUEANTE):** Compression < 30% | Execução > 20ms | Ação inconsistente para mesma entrada

---

## 9. Compatibilidade e Dependências

**Deps internas:** ECMA (entrada), KineSQL (persistência resultado), API (endpoints /encode, /action)  
**Rust:** ≥ 1.75

---

## 10. Rastreabilidade

| Tipo | ID |
|------|----|
| Spec | FT-004-MCE-ENGINE |
| Código | `src/mce/mod.rs` |
| Teste | `tests/mce_integration.rs` |

---

## 11. Roadmap MVP

### MVP (Semana 1-2)
Encoding básico | Execução síncrona | 1 intent (`risk_eval`) | 5+ testes

### Iteração 1 (Semana 3-4)
TTL enforcement | Priority automática | Múltiplos intents | Compression otimizada | Benchmark

### Iteração 2 (Semana 5-6)
Feedback loop ECMA | Integração API | Batch execution | Testes de carga | Docs
