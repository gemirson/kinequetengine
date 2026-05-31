# FT-031 — Hematoencephalic Gateway

**Módulo:** Interface | **Versão:** v6.0 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-031-HEMATOENCEPHALIC-GATEWAY | **Atualização:** 2026-05-30

---

## 1. Contexto e Objetivo

O **Hematoencephalic Gateway** atua como a barreira física e lógica (Blood-Brain Barrier) entre a rede corporativa externa e a malha distribuída de alta velocidade do KCE. Ele expõe uma interface **gRPC-Web** e **WebSockets** unificada que traduz requisições de clientes em mensagens do protocolo interno Swarm, encaminha as consultas de busca vetorial diretamente aos nós que hospedam as partições ativas de dados (shards) e transmite fluxos de telemetria em tempo real (evolução de nós, feromônios e ameaças) para dashboards de visualização.

---

## 2. Critérios de Aceite (AC)

### Gerais
- [ ] AC-001: Compilação limpa em Rust.
- [ ] AC-002: Suporte completo a chamadas HTTP/2 e fallback HTTP/1.1 via gRPC-Web.
- [ ] AC-003: Proteção integrada contra ataques de negação de serviço e inundações de pacotes.

### Específicos
- [ ] AC-010: Middleware de autenticação de Tenants via API Key em nível de gateway.
- [ ] AC-011: Roteamento inteligente de consultas: com base na chave `tenant_id` e no hash do vetor, encaminha a requisição de busca diretamente ao nó primário correspondente (FT-028) usando conexões TCP reaproveitadas (Connection Pooling).
- [ ] AC-012: Suporte a streaming bidirecional via WebSockets para telemetria em tempo real de:
  * Estado de homeostase dos nós.
  * Atividade das formigas de busca e evaporação de feromônios.
  * Alertas de ameaças imunológicas e bloqueios de antígenos.
- [ ] AC-013: Fallback transparente de conexões: se o nó de destino falhar, redireciona a query para o nó replica secundária em menos de 10ms.
- [ ] AC-014: Compressão opcional de payloads via gzip ou zstd para reduzir consumo de banda móvel.
- [ ] AC-015: Latência máxima introduzida pelo gateway na query vetorial inferior a 0.5ms.

---

## 3. Definition of Done (DoD)

- [ ] Implementação do proxy gRPC-Web utilizando `tonic` e `axum`.
- [ ] Conexão e manutenção de sessões WebSockets para transmissão de dados de monitoramento.
- [ ] Testes de carga demonstrando vazão de até 5.000 requisições por segundo por instância de gateway.
- [ ] Tratamento seguro de desconexões de clientes e vazamento de file descriptors.
- [ ] Cobertura de testes unitários superior a 80%.

---

## 4. Exemplos de Uso

### Assinatura do Serviço gRPC (Protobuf)

```protobuf
syntax = "proto3";
package kce.gateway;

service KceGateway {
  // Executa busca vetorial distribuída
  rpc Search (SearchRequest) returns (SearchResponse);
  
  // Stream de telemetria em tempo real para o dashboard
  rpc StreamTelemetry (TelemetryRequest) returns (stream TelemetryEvent);
}

message SearchRequest {
  uint64 tenant_id = 1;
  repeated double query_vector = 2;
  uint32 top_k = 3;
}

message SearchResponse {
  repeated SearchResult results = 1;
  double latency_ms = 2;
}

message SearchResult {
  uint64 id = 1;
  double score = 2;
}

message TelemetryRequest {
  uint64 tenant_id = 1;
}

message TelemetryEvent {
  string timestamp = 1;
  string component = 2; // e.g. "ACO", "AIS", "Core"
  string event_json = 3; // Estado estruturado serializado
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Entrada | Saída Esperada |
|----|------|---------|----------------|
| UT-001 | Validação de Chave de API | Requisição sem cabeçalho `x-api-key` | Retorna erro gRPC `UNAUTHENTICATED` |
| UT-002 | Roteador do Gateway | tenant=1, vector_hash=123 | Rota resolvida para o IP correto do nó correspondente |
| UT-003 | Redirecionamento sob Falha | Nó de destino offline | Redireciona e retorna com sucesso do nó réplica |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | Teste de Carga gRPC-Web | 100.000 chamadas gRPC concorrentes executadas com taxa de erro < 0.1% e latência estável |
| FT-002 | Streaming de Eventos | Cliente WebSocket conectado recebe continuamente dados de telemetria sem perda de mensagens |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Gateway → Shard Router (FT-028) | O gateway utiliza a tabela de roteamento de shards atualizada pelo conselheiro do cluster |
| IT-002 | Gateway → Resiliência (FT-009) | O gateway aplica backpressure e circuit breakers diante de sobrecarga nos nós |

---

## 6. Formato CARE

**Context:** Ponto único de entrada e validação de requisições de clientes externos, agindo como barreira de segurança e orquestrador de tráfego em tempo real para a malha distribuída do KCE.

**Assumptions:** O cliente suporta tráfego gRPC-Web ou WebSocket. O balanceador de carga upstream direciona conexões HTTP/2 para a porta do gateway (padrão: 8080).

**Requirements:** R-001: Roteamento inteligente de shards | R-002: Suporte a streaming de telemetria | R-003: Autenticação de Tenants.

**Evidence:** Execução de testes de estresse com `ghz` (ferramenta de benchmark gRPC) comprovando baixa latência sob carga concorrente extrema.

---

## 7. Critérios Não Funcionais

| Aspecto | Métrica | Alvo |
|---------|---------|------|
| Latência Adicional (Proxy Overhead) | Latência adicionada pela tradução e roteamento | < 0.5ms |
| Limite de Conexões WebSocket Concorrentes | Conexões simultâneas ativas por nó de gateway | > 10.000 |
| Vazão Máxima de Telemetria | Eventos transmitidos por segundo | > 1.000 events/sec |

---

## 8. Qualidade e Métricas

**Sucesso:**
- Zero perdas de pacotes ou desconexões inesperadas em testes de conexões WebSockets persistentes de 24 horas.
- Overhead de latência inferior a 0.5ms.
- Cobertura de testes unitários superior a 80%.

**Falha (BLOQUEANTE):**
- Vazamento de recursos (leaks de memória ou file descriptors não fechados na desconexão de clientes).
- Bypass de autenticação de tenant.

---

## 9. Compatibilidade e Dependências

| Crate | Versão | Propósito |
|-------|--------|-----------|
| `tonic` | ^0.10 | Implementação do gRPC e gRPC-Web |
| `tokio-tungstenite` | ^0.20 | Protocolo WebSocket assíncrono |

---

## 10. Rastreabilidade

| Tipo | ID | Descrição |
|------|----|-----------|
| Spec | FT-031-HEMATOENCEPHALIC-GATEWAY | Esta especificação |
| Design | `docs/kce_distributed_architecture.md` | Hematoencephalic Gateway Design |

---

## 11. Roadmap

### MVP (Fase 1)
Proxy REST Axum simples com autenticação básica e roteamento fixo para nós locais. Sem suporte a gRPC-Web ou streaming WebSockets.

### Iteração 1 (Fase 2)
Implementação do servidor gRPC-Web (`tonic`) e roteamento inteligente dinâmico de shards. Conectividade básica WebSockets para métricas simplificadas.

### Iteração 3 (Fase 3)
Streaming completo de telemetria 3D, compressão avançada, controle de fluxo e rate-limiting adaptativo integrado com a homeostase do cluster.
