# FT-030 — Distributed Consensus (Raft-Lite & CRDTs)

**Módulo:** Infraestrutura | **Versão:** v6.0 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-030-DISTRIBUTED-CONSENSUS | **Atualização:** 2026-05-30

---

## 1. Contexto e Objetivo

Sistemas distribuídos eficientes exigem um equilíbrio entre performance e consistência de dados. Tentar impor consistência forte em todas as operações de busca e telemetria vetorial degrada severamente a latência. Esta especificação define o **Consenso Híbrido** do KCE:
1. **Consistência Eventual (via CRDTs)** no caminho quente de dados (feromônios, conexões semânticas, logs de anomalia) para garantir respostas rápidas.
2. **Consistência Forte (via Raft-Lite)** no caminho de controle (configurações de tenants, chaves de acesso, atribuições fixas de shards e tabela de membros do cluster) para garantir integridade.

---

## 2. Critérios de Aceite (AC)

### Gerais
- [ ] AC-001: Compilação limpa em Rust.
- [ ] AC-002: Sem contenção de bloqueios globais nos caminhos de CRDT.
- [ ] AC-003: Tratamento de partições de rede com resolução determinística de conflitos.

### Específicos
- [ ] AC-010: Implementação de CRDTs do tipo *State-based* e *Delta-based* para sincronização eventual de grafos cognitivos e intensidades de feromônios.
- [ ] AC-011: Resolução de conflitos LWW-Element-Graph (Last-Write-Wins) baseada em watermarks temporais causais com tiebreaker determinístico por ID do nó.
- [ ] AC-012: O protocolo **Raft-Lite** deve gerenciar a eleição de líder e replicação de logs com quorum simples ($N/2 + 1$) para escrita de configurações críticas.
- [ ] AC-013: Escrita persistente do log do Raft-Lite no KineSQL WAL (FT-005) antes da confirmação da transação de controle (Commit atômico distribuído).
- [ ] AC-014: Detecção automática de perda de líder Raft-Lite e re-eleição em menos de 1,5 segundos.
- [ ] AC-015: Sincronização e reintegração automática de nós recuperados após partição de rede, aplicando deltas em background.

---

## 3. Definition of Done (DoD)

- [ ] Implementação de estruturas de CRDT (`CrdtGraph` e `CrdtPheromones`).
- [ ] Implementação da máquina de estados finitamente regulada para o protocolo Raft-Lite.
- [ ] Testes automatizados de eleição sob cenários de queda de nós em clusters de 3 e 5 nós.
- [ ] Integração com o KineSQL para gravação de logs de transação.
- [ ] Cobertura de testes unitários superior a 80%.

---

## 4. Exemplos de Uso

### Mensagem de Proposta do Raft-Lite (JSON)

**Proposta de inserção de novo tenant enviada pelo líder:**
```json
{
  "term": 3,
  "leader_id": "node-1-uuid",
  "prev_log_index": 142,
  "prev_log_term": 3,
  "entries": [
    {
      "index": 143,
      "term": 3,
      "command": "CREATE_TENANT",
      "payload": { "tenant_id": 4, "secret_hash": "e3b0c442..." }
    }
  ],
  "leader_commit": 142
}
```

### Resposta de Resolução de Conflitos CRDT (Graph Delta)

**Resultado de merge determinístico de grafos concorrentes:**
```json
{
  "merged_nodes": 12,
  "conflicts_resolved": 3,
  "strategy": "LWW_CAUSAL",
  "local_timestamp": 1780000000000,
  "remote_timestamp": 1780000000050,
  "applied_deltas": ["edge_10->12_updated"]
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Entrada | Saída Esperada |
|----|------|---------|----------------|
| UT-001 | Merge CRDT Pheromones | Local $\tau=1.5$, Remoto $\tau=2.5$ | Convergência matemática determinística |
| UT-002 | LWW-Element-Graph tiebreaker | Timestamps idênticos, IDs diferentes | O nó com maior ID alfanumérico vence |
| UT-003 | Raft-Lite Eleição Inicial | 3 nós ativos, sem líder | Um nó declara candidatura e recebe votos majoritários |
| UT-004 | Rejeição de Log Menor | Líder com termo desatualizado envia log | Retorna erro; proposta rejeitada |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | Partição de Rede Simulada | Split-brain clássico: a partição minoritária (2 nós de um cluster de 5) bloqueia escritas fortes, enquanto a majoritária (3 nós) elege novo líder e mantém as operações ativas |
| FT-002 | Sincronização em Lote | Um nó offline por 5 minutos recebe apenas a diferença incremental de grafos e feromônios, recuperando o alinhamento com a malha |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Raft-Lite → KineSQL (FT-005) | Logs do Raft são gravados e descarregados fisicamente com fsync |
| IT-002 | CRDT → Graph (FT-002) | Nós e arestas do grafo semântico são instanciados e sincronizados via CRDT |

---

## 6. Formato CARE

**Context:** Consistência e integridade transacional de dados e metadados no KCE operando em ambientes multi-nós sujeitos a instabilidades de infraestrutura de nuvem.

**Assumptions:** A maioria dos nós do cluster ($N/2 + 1$) está online e acessível. Relógios de hardware dos servidores são razoavelmente sincronizados via NTP.

**Requirements:** R-001: Consenso Raft-Lite para controle | R-002: Consistência eventual CRDT para dados quentes | R-003: Persistência no KineSQL WAL.

**Evidence:** Execução de testes de caos de rede Jepsen simplificados validando linearidade para escritas e convergência final para dados.

---

## 7. Critérios Não Funcionais

| Aspecto | Métrica | Alvo |
|---------|---------|------|
| Overhead de Merge CRDT | Tempo para mesclar estados de grafos de 10k nós | < 5ms |
| Latência de Eleição Raft | Tempo para eleger novo líder | < 1.5s |
| Tamanho do Log Raft-Lite | Compactação e expurgo periódico de logs confirmados | Limite de 50MB antes do snapshot |

---

## 8. Qualidade e Métricas

**Sucesso:**
- 100% de convergência determinística de grafos em todos os cenários de recuperação de partições de rede.
- Zero ocorrências de split-brain persistente após resolução da partição física.
- Cobertura de testes unitários superior a 80%.

**Falha (BLOQUEANTE):**
- Perda de linearidade nas escritas fortes (ex: dois líderes aceitando escritas simultâneas no mesmo termo).
- Corrupção do grafo semântico por falha no merge de CRDT.

---

## 9. Compatibilidade e Dependências

| Crate | Versão | Propósito |
|-------|--------|-----------|
| `parking_lot` | ^0.12 | Locks concorrentes rápidos |
| `serde` | ^1.0 | Serialização de mensagens e estados |

---

## 10. Rastreabilidade

| Tipo | ID | Descrição |
|------|----|-----------|
| Spec | FT-030-DISTRIBUTED-CONSENSUS | Esta especificação |
| Design | `docs/kce_distributed_architecture.md` | Distributed Consensus Design |

---

## 11. Roadmap

### MVP (Fase 1)
Consistência eventual simples baseada em Last-Write-Wins puramente em memória. Sem suporte a eleições automáticas.

### Iteração 1 (Fase 2)
Implementação completa da máquina de estados do Raft-Lite com eleição de líder e quorum. Gravação das configurações dos Tenants e Shards persistida no KineSQL.

### Iteração 2 (Fase 3)
Sincronização delta-based otimizada de CRDTs com compressão de payloads e integração total com as hipermutações controladas do AIS.
