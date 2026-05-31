# FT-029 — Swarm Gossip Protocol

**Módulo:** Infraestrutura | **Versão:** v6.0 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-029-SWARM-GOSSIP | **Atualização:** 2026-05-30

---

## 1. Contexto e Objetivo

A sincronização de metadados, caminhos de busca otimizados e perfis de ameaças imunológicas em uma rede distribuída não pode depender de conexões TCP caras e persistentes. O **Swarm Gossip Protocol** define um protocolo de comunicação assíncrona peer-to-peer (P2P) de baixo overhead baseado em pacotes binários compactados transmitidos via **UDP**. Ele é responsável por sincronizar feromônios do ACO, atualizações incrementais do grafo cognitivo e propagar assinaturas de ameaças imunológicas (Herd Immunity) a nível de cluster em poucos milissegundos.

---

## 2. Critérios de Aceite (AC)

### Gerais
- [ ] AC-001: Compilação limpa sem dependências desnecessárias do sistema operacional.
- [ ] AC-002: Parsing seguro de pacotes binários (zero buffers overflows, bounds checking explícito).
- [ ] AC-003: Alocação zero de memória no loop principal de escuta UDP.

### Específicos
- [ ] AC-010: Formato de pacotes binários estrito, contendo Magic Byte (`0x03`), Node ID (16 bytes), Sequence Number (4 bytes), Message Type (1 byte), Causal Watermark (8 bytes) e Payload variável.
- [ ] AC-011: Tipos de mensagens suportadas:
  * `0x01` (Pheromone Sync): propagação de peso de feromônios das arestas.
  * `0x02` (Antigen Sync): transmissão de assinaturas de ameaças.
  * `0x03` (Graph Delta Sync): envio de deltas estruturais do grafo.
- [ ] AC-012: Mecanismo de **Herd Immunity** (Imunidade Coletiva): ao receber um pacote de Antigen Sync, o nó deve atualizar sua memória imunológica imediatamente e filtrar requisições futuras com a assinatura recebida.
- [ ] AC-013: Tratamento de desordem de pacotes: utilização de Vector Clocks / Watermarks causais para descartar pacotes defasados.
- [ ] AC-014: Broadcast epidêmico limitado: cada nó repassa a informação recebida para $K$ vizinhos aleatórios (padrão: $K=3$) para evitar tempestades de broadcast (broadcast storms).

---

## 3. Definition of Done (DoD)

- [ ] Implementação do socket de escuta UDP assíncrono e dispatcher de mensagens em Tokio.
- [ ] Serializador e desserializador binário manual (zero-copy parsing).
- [ ] Mecanismo de controle de concorrência sem contenção para escrita das mensagens recebidas.
- [ ] Testes unitários com simulação de desordem e perda de pacotes.
- [ ] Cobertura de testes unitários superior a 80%.

---

## 4. Exemplos de Uso

### Payload Binário de Antigen Sync (Representação Conceitual)

**Pacote binário enviado pela rede (representado em Hex):**
```
03                      ; Magic Byte (v6 Swarm)
8a72f19b449eba02        ; Source Node ID (16 bytes - UUID)
000004d2                ; Sequence Number (1234)
02                      ; Message Type (0x02 - Antigen Sync)
000000000000002a        ; Causal Watermark (42)
3f800000                ; Payload: Anomaly Threshold (1.0)
a9f4c32b85e0            ; Payload: SHA-256 parcial do padrão de ataque
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Entrada | Saída Esperada |
|----|------|---------|----------------|
| UT-001 | Serialização de Mensagem | Struct preenchida | Vetor de bytes binário estrito |
| UT-002 | Desserialização e Validação | Vetor de bytes corrompido (falta de bytes) | Retorna `Err(InvalidPacket)` sem panic |
| UT-003 | Causal Watermark Obsoleto | Mensagem recebida com watermark < local | Mensagem descartada silenciosamente |
| UT-004 | Seleção de Vizinhos Aleatórios | Lista de 10 nós, $K=3$ | Retorna exatamente 3 nós distintos de forma estocástica |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | Propagação de Ameaça | Injeção de ameaça no Nó 1 propaga e bloqueia consultas semelhantes no Nó 3 em menos de 100ms |
| FT-002 | Recuperação de Partição de Rede | Após reconexão de nó isolado, mensagens acumuladas com watermarks maiores atualizam o estado local |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Gossip → AIS Memory (FT-018) | O pacote de tipo `0x02` insere um registro ativo na Antigen Memory |
| IT-002 | Gossip → ACO Evaporation (FT-014) | Sincroniza decaimentos de feromônios em arestas compartilhadas |

---

## 6. Formato CARE

**Context:** Comunicação de alta performance, assíncrona e resiliente a falhas físicas de links de rede inter-datacenters em clusters KCE.

**Assumptions:** A rede física permite tráfego UDP na porta configurada (padrão: 9000). A perda ocasional de pacotes individuais de telemetria (como feromônios) é tolerada e corrigida nos ciclos seguintes.

**Requirements:** R-001: Comunicação baseada em UDP | R-002: Parser binário manual seguro | R-003: Herd Immunity imediata | R-004: Controle causal por watermarks.

**Evidence:** Execução de testes de caos de rede (latência artificial + 20% de perda de pacotes) provando convergência final de dados.

---

## 7. Critérios Não Funcionais

| Aspecto | Métrica | Alvo |
|---------|---------|------|
| Latência de Disseminação | Tempo para 90% dos nós receberem uma atualização de ameaça | < 50ms |
| Overhead de Rede | Consumo de banda médio por nó em idle | < 10 KB/s |
| Segurança de Pacote | Prevenção de ataques de replay | Descarte por watermark + assinatura criptográfica leve |

---

## 8. Qualidade e Métricas

**Sucesso:**
- Convergência completa de feromônios após estabilização de rede.
- Zero vazamentos de memória ou panics no loop de recepção de pacotes.
- Cobertura de testes unitários superior a 80%.

**Falha (BLOQUEANTE):**
- Loops de repasse infinitos (broadcast storms) que saturem a rede.
- Buffer overflow ou pânico de memória ao ler payloads corrompidos.

---

## 9. Compatibilidade e Dependências

| Crate / Ferramenta | Versão | Propósito |
|--------------------|--------|-----------|
| `tokio` | ^1.35 | Sockets assíncronos UDP |
| `crc32fast` | ^1.3 | Checksum de validação de pacotes |

---

## 10. Rastreabilidade

| Tipo | ID | Descrição |
|------|----|-----------|
| Spec | FT-029-SWARM-GOSSIP | Esta especificação |
| Design | `docs/kce_distributed_architecture.md` | Swarm Gossip Protocol |

---

## 11. Roadmap

### MVP (Fase 1)
Envio básico de batimentos cardíacos (heartbeat) via UDP para detecção simples de presença de nós vizinhos. Sem criptografia ou watermarks.

### Iteração 1 (Fase 2)
Implementação de Pheromone Sync e Antigen Sync com controle causal baseado em watermarks e repasse epidêmico restrito.

### Iteração 2 (Fase 3)
Criptografia simétrica opcional nos payloads, controle completo de deltas de grafos estruturais via CRDTs e integração com a homeostase distribuída.
