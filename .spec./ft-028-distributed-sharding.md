# FT-028 — Distributed Sharding (ACTA)

**Módulo:** Infraestrutura / Core Engine | **Versão:** v6.0 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-028-DISTRIBUTED-SHARDING | **Atualização:** 2026-05-30

---

## 1. Contexto e Objetivo

À medida que o KCE escala para múltiplos nós e servidores, manter todo o grafo semântico e o espaço vetorial em um único nó torna-se inviável devido a limitações de CPU e memória. Esta especificação define o **Distributed Sharding**, que fatia os dados com base em chaves de partição e emprega um mecanismo dinâmico inspirado na divisão de trabalho de colônias de formigas: o **Ant-Colony Task Allocation (ACTA)**. O objetivo é distribuir a carga de busca e indexação dinamicamente conforme o uso real, mitigando problemas de shards quentes (*hot shards*).

---

## 2. Critérios de Aceite (AC)

### Gerais
- [ ] AC-001: Compilação limpa sem warnings no compilador Rust.
- [ ] AC-002: Isolamento estrito entre shards de tenants distintos.
- [ ] AC-003: Thread-safety completo para leituras simultâneas e redistribuições assíncronas.

### Específicos
- [ ] AC-010: Definição da chave de partição composta por `tenant_id` (isolamento primário) e `context_hash` (distribuição secundária do grafo).
- [ ] AC-011: Encaminhamento dinâmico de buscas: se um nó $A$ que hospeda o Shard primário estiver sob alta utilização ($>85\%$ CPU/RAM), ele deve delegar a tarefa (query) de forma transparente para réplicas secundárias no nó $B$ menos carregado.
- [ ] AC-012: O motor de homeostase distribui shards de forma a "atrair" dados semanticamente correlacionados para nós geograficamente ou fisicamente próximos, minimizando saltos de rede inter-nós.
- [ ] AC-013: Migração assíncrona de shards: redistribuição de dados sem bloqueio das queries de leitura no hot path.
- [ ] AC-014: Suporte para até 256 partições lógicas mapeadas dinamicamente para os nós físicos disponíveis.

---

## 3. Definition of Done (DoD)

- [ ] Implementação de traits e estruturas de particionamento de dados (`ShardRouter` e `TaskAllocator`).
- [ ] Algoritmo ACTA simulado e validado em cenários de stress com redistribuição em background.
- [ ] Testes de migração sem perda de pacotes ou interrupção de serviço.
- [ ] Testes unitários com no mínimo 80% de cobertura.
- [ ] Coexistência de shards locais (KineSQL) com roteamento de rede.

---

## 4. Exemplos de Uso

### Estrutura de Roteamento de Shard (JSON interno)

**Roteador de Shard configurado no nó:**
```json
{
  "node_id": "8a72-f19b-449e-ba02",
  "assigned_shards": [
    { "shard_id": 12, "tenant_id": 1, "hash_range": [0, 1000] },
    { "shard_id": 13, "tenant_id": 1, "hash_range": [1001, 2000] }
  ],
  "node_load": {
    "cpu_utilization": 0.42,
    "memory_free_bytes": 8589934592,
    "active_delegations": 0
  }
}
```

### Resposta de Roteamento de Delegamento de Tarefa (Query)

**JSON retornado ao delegar busca para o nó vizinho:**
```json
{
  "query_id": "9a1f-82bc",
  "status": "DELEGATED",
  "delegated_to_node": "3c01-d822-411a-ba73",
  "reason": "Source node load above 85%; routing to low-latency replica",
  "latency_overhead_ms": 1.2
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Entrada | Saída Esperada |
|----|------|---------|----------------|
| UT-001 | Determinação de Shard | tenant=1, vector_hash=420 | shard_id consistente (ex: 12) |
| UT-002 | Gatilho de Delegação ACTA | Carga do nó A = 90% | `TaskAllocator::should_delegate` retorna `true` |
| UT-003 | Migração de Shard | Sinal de redistribuição | Dados movidos para o nó destino; checksum validado |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | Simulação de Hot Shard | Carga artificial em 1 nó resulta no redirecionamento automático de 40% das queries para nós secundários em 30 segundos |
| FT-002 | Queda de Nó Primário | Detecção e promoção imediata de um nó secundário para assumir as leituras do shard |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Sharding → KineSQL (FT-005) | Dados migrados são serializados e descarregados no banco em disco no destino |
| IT-002 | Sharding → Homeostase (FT-022) | Homeostase ajusta os thresholds de carga que ativam a delegação de tarefas |

---

## 6. Formato CARE

**Context:** Escalonamento horizontal e mitigação de gargalos de hardware em clusters KCE operando em cenários reais com múltiplos clientes simultâneos.

**Assumptions:** Cada nó físico conhece a topologia de nós vizinhos e as capacidades máximas de hardware declaradas. A rede inter-nós possui latência RTT inferior a 2ms.

**Requirements:** R-001: Roteamento baseado em hash de tenant e dados | R-002: Delegação de tarefas baseada em carga (ACTA) | R-003: Migração assíncrona tolerante a falhas.

**Evidence:** Execução de scripts de carga distribuídos simulando redistribuição dinâmica de shards e validação de latências p95.

---

## 7. Critérios Não Funcionais

| Aspecto | Métrica | Alvo |
|---------|---------|------|
| Overhead de Roteamento | Latência adicional de rede para decisão de redirecionamento | < 0.2ms |
| Tempo de Convergência | Tempo para um novo nó assumir um shard delegado | < 1s |
| Perda de Dados durante Migração | Integridade das queries em andamento | 0 erros (Transicional) |

---

## 8. Qualidade e Métricas

**Sucesso:**
- Desvio padrão da carga de CPU entre os nós do cluster inferior a 15% após estabilização do ACTA.
- Zero perda de dados ou inconsistência durante a migração.
- Cobertura de testes unitários superior a 80%.

**Falha (BLOQUEANTE):**
- Inconsistência de partição (duas chaves idênticas em shards ativos separados sem replicação configurada).
- Vazamento de dados de tenant durante a redistribuição.

---

## 9. Compatibilidade e Dependências

| Crate | Versão | Propósito |
|-------|--------|-----------|
| `uuid` | ^1.6 | IDs únicos de nós e transações |
| `parking_lot` | ^0.12 | Locks concorrentes para tabelas de rotas |

---

## 10. Rastreabilidade

| Tipo | ID | Descrição |
|------|----|-----------|
| Spec | FT-028-DISTRIBUTED-SHARDING | Esta especificação |
| Design | `docs/kce_distributed_architecture.md` | Arquitetura distribuída de swarm |

---

## 11. Roadmap

### MVP (Fase 1)
Particionamento estático em memória com roteamento baseado em tenant_id. Sem migração assíncrona.

### Iteração 1 (Fase 2)
Implementação do ACTA (delegação dinâmica baseada em telemetria de CPU). Suporte a réplicas ativas e redirecionamento de busca transparente.

### Iteração 2 (Fase 3)
Migração dinâmica assíncrona de shards com fsync remoto e integração completa com a homeostase global.
