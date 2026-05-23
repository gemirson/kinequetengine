# FT-018 — Antigen Memory System

**Módulo:** AIS (Artificial Immune System) | **Versão:** v6.0 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-018-ANTIGEN-MEMORY | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O Antigen Memory System **memoriza padrões críticos** (ataques, fraudes, eventos raros) de forma persistente, permitindo resposta mais rápida em ocorrências futuras. Inspirado nas células B de memória do sistema imunológico — após primeira exposição a um antígeno, o sistema "lembra" e responde exponencialmente mais rápido na reexposição.

### Valor
- Aprendizado contínuo real — sistema nunca esquece ameaças
- Resposta mais rápida no futuro (O(1) lookup vs O(n) detecção)
- Base de conhecimento de anomalias cresce organicamente

---

## 2. Critérios de Aceite (AC)

- [ ] AC-010: `store_antigen(pattern, metadata)` persiste padrão anômalo
- [ ] AC-011: `match_antigen(input)` verifica se input é similar a antígeno conhecido
- [ ] AC-012: Match usa similarity threshold configurável (default: 0.85)
- [ ] AC-013: Lookup em < 1ms (hash-based ou index)
- [ ] AC-014: Antigens persistidos no KineSQL — sobrevivem restart
- [ ] AC-015: Cada antigen possui: `pattern`, `severity`, `first_seen`, `last_seen`, `match_count`, `metadata`
- [ ] AC-016: Antigen expiry configurável (default: 90 dias, 0 = permanente)
- [ ] AC-017: Antigen memory exportável (JSON/CSV) para auditoria
- [ ] AC-018: Integração com Immune Detection (FT-017) — anomalias viram antigens

---

## 3. Definition of Done (DoD)

- [ ] store/match funcional e testado
- [ ] Persistência KineSQL
- [ ] Lookup < 1ms
- [ ] Expiry funcional
- [ ] Export JSON/CSV
- [ ] Integração FT-017
- [ ] Cobertura ≥ 80%

---

## 4. Exemplos de Uso

### Armazenamento de antígeno

**Entrada (anomalia detectada):**
```json
{
  "pattern": {
    "vector_signature": [0.99, 0.01, 0.99, 0.01],
    "frequency_range": [400, 600],
    "ip_class": "tor_exit"
  },
  "severity": "HIGH",
  "metadata": { "source": "immune_detection", "incident_id": "INC-2026-0042" }
}
```

**Antigen criado:**
```json
{
  "antigen_id": "ag_001",
  "pattern": { "vector_signature": [0.99, 0.01, 0.99, 0.01], "frequency_range": [400, 600] },
  "severity": "HIGH",
  "first_seen": "2026-05-23T10:00:00Z",
  "last_seen": "2026-05-23T10:00:00Z",
  "match_count": 0,
  "expiry_days": 90,
  "status": "ACTIVE"
}
```

### Match de antígeno conhecido
**Input semelhante a antígeno:**
```json
{ "query_vector": [0.98, 0.02, 0.97, 0.03], "request_frequency": 450 }
```

**Resultado:**
```json
{
  "antigen_match": true,
  "matched_antigen": "ag_001",
  "similarity": 0.97,
  "severity": "HIGH",
  "response_time_ms": 0.3,
  "action": "BLOCK_IMMEDIATELY",
  "match_count_updated": 1
}
```

### Export (CSV)
```csv
antigen_id,severity,first_seen,last_seen,match_count,status
ag_001,HIGH,2026-05-23T10:00:00Z,2026-05-23T14:30:00Z,12,ACTIVE
ag_002,MEDIUM,2026-05-20T08:00:00Z,2026-05-22T16:00:00Z,3,ACTIVE
ag_003,LOW,2026-03-01T12:00:00Z,2026-03-01T12:00:00Z,0,EXPIRED
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Saída Esperada |
|----|------|----------------|
| UT-001 | Store antigen | antigen_id gerado, persistido |
| UT-002 | Match — similar (0.97) | `antigen_match: true` |
| UT-003 | Match — dissimilar (0.30) | `antigen_match: false` |
| UT-004 | Match count incrementa | `match_count += 1` |
| UT-005 | Expiry — antigen expirado | não retornado em match |
| UT-006 | Lookup performance | < 1ms para 1000 antigens |
| UT-007 | Export JSON | formato válido |
| UT-008 | Export CSV | formato válido |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | Pattern vazio | `Err(EmptyPattern)` |
| UF-002 | Severity inválida | `Err(InvalidSeverity)` |
| UF-003 | Antigen duplicado exato | Atualiza existing (idempotente) |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado |
|----|---------|-----------|
| FT-001 | Store 100 antigens → match | Todos matcheiam corretamente |
| FT-002 | Antigen survives restart | Persiste via KineSQL |
| FT-003 | Expiry após 90 dias (simulado) | Antigen removido de matches |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Detection (FT-017) → Antigen Memory | Anomalia cria antigen automaticamente |
| IT-002 | Antigen → Response Amplifier (FT-019) | Antigen match amplifica resposta |
| IT-003 | Antigen → KineSQL | Persistência funcional |

---

## 6. Formato CARE

**Context:** Detecção de anomalias é cara (O(n)). Memorizar padrões críticos permite resposta O(1) em reexposição. Células B de memória biológicas inspiram persistência de longo prazo para ameaças conhecidas.

**Assumptions:** Antigens são vetoriais com similarity match; 1000 antigens é ceiling prático para MVP; lookup hash-based para < 1ms; persistência KineSQL; expiry configurável.

**Requirements:** R-001: store/match | R-002: Persistência | R-003: < 1ms lookup | R-004: Expiry | R-005: Export JSON/CSV | R-006: Integração FT-017 | R-007: Idempotência.

**Evidence:** `tests/antigen_memory_tests.rs` | `reports/antigen_response_time.md`

---

## 7–11. (Resumo)

**Não funcionais:** Lookup < 1ms | Storage < 10MB para 1000 antigens | Expiry accuracy ± 1 hora  
**Qualidade:** Match accuracy > 95% | 0 false negatives para exact match | Cobertura ≥ 80%  
**Falha (BLOQUEANTE):** Lookup > 10ms | Antigen perdido após restart | False negative > 5%  
**Deps:** KineSQL (FT-005), Immune Detection (FT-017), Response Amplifier (FT-019)  
**Rastreabilidade:** FT-018-ANTIGEN-MEMORY | `src/ais/antigen.rs`

**Roadmap:** MVP: store/match com hash lookup | Iter1: KineSQL persistência + expiry + CSV export | Iter2: Similarity match + FT-017 integração + audit trail

---

*Documento gerado em 2026-05-23 — KineContext Engine Product Specification*
