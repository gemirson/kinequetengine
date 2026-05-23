# FT-005 — KineSQL (Storage Engine)

**Módulo:** Storage | **Versão:** v5.9 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-005-KINESQL-STORAGE | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

O KineSQL é o **storage engine embedded** do KCE. Fornece persistência confiável com WAL (Write-Ahead Log) real, paginação, mmap e checksums por página. É o alicerce de confiabilidade: se o KineSQL falha, o sistema todo perde dados. Deve sobreviver a crashes sem corrupção.

---

## 2. Critérios de Aceite (AC)

- [ ] AC-010: WAL persistente em disco (não apenas memória)
- [ ] AC-011: `fsync` ativo em todas as escritas críticas (WAL + data)
- [ ] AC-012: Atomic commit: WAL → fsync WAL → apply data → fsync data
- [ ] AC-013: Recovery funcional: replay WAL após crash sem perda
- [ ] AC-014: Checksum CRC32 por página — detecta corrupção
- [ ] AC-015: Paginação implementada com tamanho configurável
- [ ] AC-016: Leitura consistente após restart
- [ ] AC-017: Suporte a mmap para leitura rápida
- [ ] AC-018: Concorrência via `Arc<RwLock<KineSQL>>`

---

## 3. Definition of Done (DoD)

- [ ] WAL persistente com fsync
- [ ] Recovery funcional testado com crash simulado
- [ ] Checksum CRC32 validado por página
- [ ] Paginação implementada
- [ ] Atomic commit implementado
- [ ] Testes de crash/recovery passando
- [ ] Cobertura ≥ 80%

---

## 4. Exemplos de Uso

### Escrita + Recovery

**Operação de escrita:**
```json
{
  "operation": "INSERT",
  "table": "vectors",
  "data": { "id": 42, "vector": [0.12, 0.85, 0.33, 0.67], "metadata": {"label": "credit_risk"} }
}
```

**WAL entry (formato interno):**
```
1716500000|INSERT|vectors|{"id":42,"vector":[0.12,0.85,0.33,0.67]}|CRC:0xA3F2B1C4
```

**Após crash + recovery:**
```json
{
  "recovery": {
    "wal_entries_replayed": 47,
    "pages_recovered": 12,
    "checksum_failures": 0,
    "data_integrity": "VALID"
  }
}
```

### Erro — checksum inválido
```json
{
  "error": "CHECKSUM_FAILURE",
  "message": "Page 7 CRC mismatch: expected 0xA3F2B1C4, got 0x00000000",
  "code": 500,
  "action": "PAGE_MARKED_CORRUPT"
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Entrada | Saída Esperada |
|----|------|---------|----------------|
| UT-001 | Checksum válido | página com dados | `checksum == expected` |
| UT-002 | Checksum inválido | página corrompida | `Err(ChecksumFailure)` |
| UT-003 | WAL append | operação INSERT | entrada no arquivo WAL |
| UT-004 | WAL replay | arquivo WAL com 10 entries | 10 operações aplicadas |
| UT-005 | Página cheia → nova página | inserção excede tamanho | nova página alocada |
| UT-006 | Leitura por ID | `get(42)` | registro correto |

**Falhas esperadas:**

| ID | Caso | Comportamento |
|----|------|---------------|
| UF-001 | WAL corrompida | `Err(WalCorrupted)` com posição do erro |
| UF-002 | Disco cheio | `Err(DiskFull)` — sem corrupção parcial |
| UF-003 | Registro inexistente | `Err(NotFound)` |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | Insert → kill → restart → read | Dados íntegros após recovery |
| FT-002 | 10k inserts → checksum all pages | Todos CRC32 válidos |
| FT-003 | Concurrent reads + writes | Sem corrupção, leituras consistentes |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | KineSQL → Retrieval | Vetores lidos corretamente do storage |
| IT-002 | KineSQL → Graph | Grafo persiste e restaura |
| IT-003 | KineSQL → ECMA | Estados dos nós persistem |

---

## 6. Formato CARE

**Context:** Storage engine embedded que é o alicerce de confiabilidade do KCE; deve sobreviver a crashes sem perda/corrupção de dados usando WAL + fsync + checksum.

**Assumptions:** Filesystem suporta fsync; disco tem espaço suficiente; WAL é single-writer (write lock); mmap disponível no OS alvo; CRC32 é suficiente para detecção de corrupção (não para segurança criptográfica).

**Requirements:** R-001: WAL persistente | R-002: fsync em todas escritas | R-003: Atomic commit | R-004: Recovery sem perda | R-005: CRC32 por página | R-006: Paginação | R-007: mmap read.

**Evidence:** `tests/kinesql_crash_tests.rs` | `tests/kinesql_recovery.rs`

---

## 7. Critérios Não Funcionais

| Aspecto | Alvo |
|---------|------|
| Write latency (fsync) | < 10ms |
| Read latency (mmap) | < 1ms |
| Recovery time (1k entries) | < 500ms |
| Max data size | ≥ 10GB |
| Durabilidade | 0 perda em crash simulado |

---

## 8. Qualidade e Métricas

**Sucesso:** 0 perda em crash | CRC32 100% válidos | Recovery automático | Cobertura ≥ 80%  
**Falha (BLOQUEANTE):** Perda de dados em crash | Corrupção silenciosa | Recovery falha

---

## 9. Compatibilidade e Dependências

| Crate | Versão | Propósito |
|-------|--------|-----------|
| `crc32fast` | ^1.3 | Checksum |
| `parking_lot` | ^0.12 | RwLock |
| `memmap2` | ^0.9 | Memory-mapped I/O |

**OS:** Linux (prod) — fsync garantido | **Filesystem:** ext4, xfs (recomendado)

---

## 10. Rastreabilidade

| Tipo | ID |
|------|----|
| Spec | FT-005-KINESQL-STORAGE |
| Código | `src/storage/mod.rs`, `src/storage/wal.rs`, `src/storage/page.rs` |
| Teste | `tests/kinesql_crash_tests.rs` |

---

## 11. Roadmap MVP

### MVP (Semana 1-2)
WAL em disco (append) | Leitura/escrita básica | Checksum CRC32 | 5+ testes unitários

### Iteração 1 (Semana 3-4)
fsync real | Atomic commit | Recovery (WAL replay) | Paginação | Crash tests

### Iteração 2 (Semana 5-6)
mmap para leitura | Concorrência RwLock | Integração com todos módulos | Stress test 10k ops | Docs
