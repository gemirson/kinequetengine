# FT-025 — MCP Server (Model Context Protocol)

**Módulo:** Interface | **Versão:** v6.0 | **Prioridade:** P1 — Importante  
**Artefato ID:** FT-025-MCP-SERVER | **Atualização:** 2026-05-30

---

## 1. Contexto e Objetivo

O **MCP Server** atua como a interface direta do KCE para agentes de inteligência artificial (como Cursor, Claude Desktop e outros). Ele implementa o protocolo JSON-RPC 2.0 através de comunicação bidirecional por entrada e saída padrão (`stdin`/`stdout`), expondo as capacidades essenciais do KCE como ferramentas automatizadas. Isso permite que LLMs façam buscas vetoriais, calibração de anomalias por IA e cálculos matemáticos de transporte ótimo diretamente sob demanda.

---

## 2. Critérios de Aceite (AC)

### Gerais
- [ ] AC-001: Compila sem warnings em `--release`
- [ ] AC-002: Zero alocações redundantes no loop de eventos de E/S
- [ ] AC-003: Cobertura de documentação pública completa (/// doc comments)

### Específicos
- [ ] AC-010: Handshake inicial via chamada de método `initialize` retornando a versão do protocolo (`2024-11-05`) e capacidades do servidor.
- [ ] AC-011: Listagem dinâmica de ferramentas disponíveis via chamada de método `tools/list` com esquemas JSON válidos.
- [ ] AC-012: Suporte à execução de ferramentas com o método `tools/call`.
- [ ] AC-013: Ferramenta `kce_retrieve`: aceita vetor de consulta e conjunto de dados, executando a busca híbrida do KCE.
- [ ] AC-014: Ferramenta `kce_classify`: aceita vetor e padrões conhecidos de auto-reconhecimento (self), executando o classificador imunológico.
- [ ] AC-015: Ferramenta `kce_sinkhorn`: computa a distância de transporte ótimo Sinkhorn entre duas distribuições.
- [ ] AC-016: Ferramenta `kce_regulate`: executa a avaliação e homeostase de rede com métricas de performance do sistema.
- [ ] AC-017: Respostas em conformidade estrita com a especificação JSON-RPC 2.0 (campos `jsonrpc`, `id`, `result` e `error`).
- [ ] AC-018: Manipulação segura de strings e tratamento de erros do parser de JSON (evitando panics).

---

## 3. Definition of Done (DoD)

- [ ] Parser de JSON-RPC 2.0 validado e robusto
- [ ] 4 ferramentas core registradas e testadas via mock E/S
- [ ] Tratamento de erros estruturado para chamadas inválidas e erros internos
- [ ] Testes unitários de serialização de payloads
- [ ] Testes funcionais com simulação de interações por linha de comando
- [ ] Sem chamadas de `unwrap()` em caminhos de execução críticos
- [ ] Integração completa com o módulo de logs e métricas do KCE

---

## 4. Exemplos de Uso

### Chamada para Executar a Ferramenta de Busca (`kce_retrieve`)

**Entrada (stdin):**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "kce_retrieve",
    "arguments": {
      "query_vector": [0.1, 0.2, 0.3],
      "dataset_vectors": [
        [0.1, 0.2, 0.3],
        [0.9, 0.8, 0.7]
      ],
      "top_k": 1
    }
  },
  "id": 1
}
```

**Saída (stdout):**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"results\": [\n    {\n      \"id\": 0,\n      \"score\": 1.0,\n      \"cosine_score\": 1.0,\n      \"prime_score\": 1.0\n    }\n  ],\n  \"count\": 1\n}"
      }
    ]
  }
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Entrada | Saída Esperada |
|----|------|---------|----------------|
| UT-001 | Handshake inicial | Método `initialize` com ID numérico | Resposta JSON-RPC com infos do servidor |
| UT-002 | Listagem de ferramentas | Método `tools/list` | JSON listando as 4 ferramentas cadastradas |
| UT-003 | Método inválido | Método `tools/non_existent` | Código de erro `-32601` (Method not found) |
| UT-004 | Parser error | Payload de texto corrompido ou JSON inválido | Código de erro `-32700` (Parse error) |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | Execução do loop principal | O servidor roda continuamente lendo linhas da stdin até o fim do fluxo |
| FT-002 | Teste de similaridade Sinkhorn | Chamada de `kce_sinkhorn` retorna distância computada com sucesso |
| FT-003 | Execução de Homeostase | Chamada de `kce_regulate` gera ajustes dinâmicos com base nas métricas fornecidas |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | MCP → Kce-Retrieval | O vetor de entrada é mapeado de forma idêntica e busca é executada |
| IT-002 | MCP → Kce-Ais | Classificação self/non-self retorna o label correto com nível de confiança |

---

## 6. Formato CARE

**Context:** Canal de comunicação direta com agentes inteligentes para tornar o KCE acionável por sistemas LLM modernos, estendendo o motor cognitivo para fluxos autônomos.

**Assumptions:** As entradas e saídas utilizam exclusivamente UTF-8 e JSON sobre os descritores de arquivo padrão (`stdin`/`stdout`). O servidor não tenta gerenciar sessões concorrentes no mesmo canal físico.

**Requirements:** R-001: conformidade JSON-RPC 2.0 | R-002: conformidade com o Model Context Protocol (MCP) | R-003: exposição de retrieval, classify, sinkhorn e regulation como ferramentas.

**Evidence:** Testes automatizados em `crates/kce-mcp` e integração de ferramentas comprovadas por testes do console.

---

## 7. Critérios Não Funcionais

| Aspecto | Métrica | Alvo |
|---------|---------|------|
| Latência do Dispatcher | Tempo de parsing e roteamento de mensagens | < 1ms |
| Pegada de memória | Consumo de RAM básico em idle | < 15MB |
| Tratamento de erro | Segurança sob dados malformados | Resiliência completa (sem panic) |

---

## 8. Qualidade e Métricas

**Sucesso:** Resposta correta do protocolo a todas as mensagens com conformidade estrita (0 erros de parsing no handshake).

**Falha (BLOQUEANTE):** Queda de conexão abrupta (panic) sob JSON malformado ou interrupção de stdin.

---

## 9. Compatibilidade e Dependências

| Crate | Versão | Propósito |
|-------|--------|-----------|
| `serde_json` | ^1.0 | Serialização e desserialização |
| `kce-retrieval` | Interna | Execução da busca híbrida |
| `kce-ais` | Interna | Classificação e regulação |

---

## 10. Rastreabilidade

| Tipo | ID | Descrição |
|------|----|-----------|
| Spec | FT-025-MCP-SERVER | Esta especificação |
| Código | `crates/kce-mcp/src/main.rs` | Implementação do servidor de E/S |

---

## 11. Roadmap

### MVP (Fase Atual)
Interface JSON-RPC 2.0 básica com suporte aos 3 métodos padrão e exposição de retrieval e classify.

### Iteração 1
Integração completa com as métricas do KCE (/metrics) e suporte à execução em tempo real da regulação por homeostase.

### Iteração 2
Suporte a canais de transporte adicionais (gRPC-Web / SSE) para conexões Web.
