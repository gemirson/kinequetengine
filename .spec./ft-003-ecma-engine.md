# FT-003 — ECMA (Embryological Cognitive Memory Architecture)

**Módulo:** Core Engine | **Versão:** v5.9 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-003-ECMA-ENGINE | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O ECMA é o motor de **evolução cognitiva** do KCE. Nós de conhecimento evoluem através de estados (Stem → Progenitor → Specialized → Apoptosis) baseado em uso, entropia e conexões. Implementa um ciclo de vida biológico-inspirado que garante que conhecimento relevante amadurece e conhecimento obsoleto é degradado automaticamente.

---

## 2. Critérios de Aceite (AC)

- [ ] AC-010: Estados implementados: `Stem`, `Progenitor`, `Specialized`, `Apoptosis`
- [ ] AC-011: Transições automáticas baseadas em `usage_count`, `entropy`, `connections`
- [ ] AC-012: Nós evoluem para `Specialized` após 10+ interações com alta relevância
- [ ] AC-013: Nós com baixa relevância (entropia > threshold) degradam para `Apoptosis`
- [ ] AC-014: Nenhuma transição inválida ocorre (ex: Apoptosis → Stem)
- [ ] AC-015: Função `maturity(node)` retorna score 0.0..1.0
- [ ] AC-016: Feedback loop: resultado de execução MCE retroalimenta maturidade
- [ ] AC-017: Estado do nó é persistido no KineSQL

---

## 3. Definition of Done (DoD)

- [ ] Todos os 4 estados implementados
- [ ] Máquina de estados com transições validadas
- [ ] Função de maturidade ativa e testada
- [ ] Feedback loop MCE → ECMA funcional
- [ ] Persistência de estados no KineSQL
- [ ] Testes unitários ≥ 80% cobertura
- [ ] Sem transições inválidas possíveis

---

## 4. Exemplos de Uso

### Evolução de nó
**Entrada (estado inicial):**
```json
{
  "node_id": 42,
  "state": "Stem",
  "usage_count": 0,
  "entropy": 0.9,
  "connections": 2,
  "maturity": 0.1
}
```

**Após 15 interações com alta relevância:**
```json
{
  "node_id": 42,
  "state": "Specialized",
  "usage_count": 15,
  "entropy": 0.2,
  "connections": 8,
  "maturity": 0.87
}
```

### Degradação (Apoptosis)
```json
{
  "node_id": 99,
  "state": "Apoptosis",
  "usage_count": 2,
  "entropy": 0.95,
  "connections": 0,
  "maturity": 0.05,
  "reason": "HIGH_ENTROPY_LOW_USAGE"
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Entrada | Saída Esperada |
|----|------|---------|----------------|
| UT-001 | Maturity > 0.5 para nó ativo | usage=10, entropy=0.3 | `maturity > 0.5` |
| UT-002 | Transição Stem → Progenitor | usage=5, entropy=0.5 | `state == Progenitor` |
| UT-003 | Transição → Specialized | usage=15, entropy=0.2 | `state == Specialized` |
| UT-004 | Degradação → Apoptosis | usage=1, entropy=0.95 | `state == Apoptosis` |
| UT-005 | Transição inválida bloqueada | Apoptosis → Stem | `Err(InvalidTransition)` |
| UT-006 | Maturity range | qualquer nó | `0.0 <= maturity <= 1.0` |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | Transição Apoptosis → Stem | `Err(InvalidTransition)` |
| UF-002 | Entropy negativa | `Err(InvalidEntropy)` |
| UF-003 | Nó inexistente para update | `Err(NodeNotFound)` |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | Simular 20 interações | Nó evolui Stem → Progenitor → Specialized |
| FT-002 | Simular abandono | Nó degrada para Apoptosis |
| FT-003 | Feedback loop MCE | Resultado positivo aumenta maturidade |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Retrieval → ECMA | Nós retornados têm maturidade atualizada |
| IT-002 | ECMA → KineSQL | Estados persistem após restart |
| IT-003 | MCE → ECMA | Feedback de execução retroalimenta |

---

## 6. Formato CARE

**Context:** Motor de evolução cognitiva que implementa ciclo de vida biológico para nós de conhecimento; garante que informação relevante amadurece e obsoleta é removida.

**Assumptions:** Threshold de transição é configurável; entropia é calculada externamente; usage_count é incrementado pelo Retrieval Engine; persistência via KineSQL.

**Requirements:** R-001: 4 estados | R-002: transições automáticas | R-003: maturity 0..1 | R-004: feedback loop MCE | R-005: persistência | R-006: sem transições inválidas.

**Evidence:** `tests/ecma_tests.rs` | `src/ecma/state_machine.rs`

---

## 7. Critérios Não Funcionais

| Aspecto | Alvo |
|---------|------|
| Latência update | < 1ms por nó |
| Memória por nó | < 256 bytes |
| Transições/s | ≥ 10k |

---

## 8. Qualidade e Métricas

**Sucesso:** 0 transições inválidas | Maturity sempre em [0,1] | Cobertura ≥ 80%  
**Falha (BLOQUEANTE):** Transição inválida ocorre | Maturity fora de range | Estado corrompido após restart

---

## 9. Compatibilidade e Dependências

**Deps internas:** Retrieval (usage_count), Graph (connections), MCE (feedback), KineSQL (persistência)  
**Rust:** ≥ 1.75

---

## 10. Rastreabilidade

| Tipo | ID |
|------|----|
| Spec | FT-003-ECMA-ENGINE |
| Código | `src/ecma/mod.rs`, `src/ecma/state_machine.rs` |
| Teste | `tests/ecma_integration.rs` |

---

## 11. Roadmap MVP

### MVP (Semana 1-2)
4 estados | Transições manuais | `maturity()` | 5+ testes unitários

### Iteração 1 (Semana 3-4)
Transições automáticas por threshold | Feedback loop MCE | Validação de transição | Benchmark

### Iteração 2 (Semana 5-6)
Persistência KineSQL | Integração completa pipeline | Testes de carga | Docs
