# FT-033 — io_uring Zero-Copy Splice (Storage-to-Network Bypass)

**Módulo:** Storage / Interface | **Versão:** v6.0 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-033-IO-URING-SPLICE | **Atualização:** 2026-05-30

---

## 1. Contexto e Objetivo

No tráfego de busca de alto rendimento e sincronização distribuída, o KCE precisa transmitir grandes volumes de dados (páginas do KineSQL, logs do WAL e blocos de vetores) da persistência em disco diretamente para a rede (Gateway e outros nós da malha). O fluxo convencional envolve ler do disco para um buffer de usuário em memória RAM e depois escrever esse buffer no socket de rede, o que consome ciclos de CPU redundantes e satura a largura de banda da memória.

Como uma extensão da feature **FT-032 (io_uring Bare-Metal)**, o **io_uring Zero-Copy Splice** utiliza a operação `IORING_OP_SPLICE` do kernel Linux. Esta feature permite conectar diretamente o descritor de arquivo físico (KineSQL) ao descritor de rede (TCP/UDP) dentro do próprio kernel através de um buffer pipe intermediário, transmitindo dados com **cópia zero (Zero-Copy)**. Isso elimina completamente o tráfego de dados no espaço de usuário do KCE e libera a CPU para processar buscas e análises cognitivas.

---

## 2. Critérios de Aceite (AC)

### Gerais
- [ ] AC-001: Compilação sem avisos ou erros no ecossistema Rust.
- [ ] AC-002: Liberação automática de recursos (pipes e sockets do kernel) após a conclusão do splice.
- [ ] AC-003: Tolerância a perdas de conexão de rede durante a transferência (descarte de frames parcial com erro limpo gRPC/TCP).

### Específicos
- [ ] AC-010: Registro integrado no anel `io_uring` de descritores de arquivos de banco de dados (KineSQL) e descritores de conexões de sockets ativos (Gateway gRPC/Gossip UDP).
- [ ] AC-011: Execução de chamadas `IORING_OP_SPLICE` de forma assíncrona, ligando o arquivo de origem ao descritor de socket por meio de pipes de kernel pré-alocados.
- [ ] AC-012: Encadeamento de operações: suporte a envio encadeado de cabeçalho gRPC-Web (`write` no anel) seguido do corpo de dados via splice (`splice` no anel) usando `IOSQE_IO_LINK`.
- [ ] AC-013: Otimização de buffer de kernel: redimensionamento dinâmico do tamanho do buffer de pipe interno (`fcntl` F_SETPIPE_SZ) para suportar blocos de dados de até 1MB sem fragmentação.
- [ ] AC-014: Mecanismo de **Fallback Transparente**: caso o kernel não suporte a operação `splice` de arquivos específicos ou o SO não seja compatível, reverte para leitura assíncrona em buffer de usuário (`read`) seguida de escrita no socket (`write`) de maneira invisível para a aplicação.

---

## 3. Definition of Done (DoD)

- [ ] Módulo `IoUringSplicer` implementado e acoplado ao anel principal do `IoUringBackend`.
- [ ] Pool de buffers de pipe gerenciado de forma segura no espaço de kernel.
- [ ] Testes de validação de integridade provando que a transmissão via splice envia bytes idênticos aos gravados originalmente.
- [ ] Benchmarks comparativos sob carga concorrente extrema demonstrando redução superior a 40% no consumo de ciclos de CPU por MB transmitido.
- [ ] Cobertura de testes unitários superior a 80%.

---

## 4. Exemplos de Uso

### Estrutura de Submissão de Splice no Anel (Conceitual em Rust)

```rust
// Exemplo conceitual de encadeamento de splice para transmitir dados de banco direto para o socket TCP
use io_uring::{opcode, squeue, types};

pub fn submit_zero_copy_splice(
    ring: &mut io_uring::IoUring,
    fd_in: types::Fd,  // Arquivo KineSQL
    off_in: i64,
    fd_out: types::Fd, // Socket TCP de rede
    len: u32,
    pipe_write_fd: types::Fd, // Ponta de escrita do pipe interno do kernel
    pipe_read_fd: types::Fd,  // Ponta de leitura do pipe interno do kernel
) -> Result<(), KceError> {
    // 1. Splice do Arquivo de Origem para o Pipe (Escrita no Pipe)
    let splice_to_pipe = opcode::Splice::new(fd_in, off_in, pipe_write_fd, -1, len)
        .build()
        .flags(squeue::Flags::IO_LINK) // Encadeia com o próximo passo
        .user_data(0x2001);

    // 2. Splice do Pipe para o Socket de Destino (Leitura do Pipe)
    let splice_to_socket = opcode::Splice::new(pipe_read_fd, -1, fd_out, -1, len)
        .build()
        .user_data(0x2002);

    // 3. Submeter ambas as operações encadeadas
    unsafe {
        let mut sq = ring.submission();
        sq.push(&splice_to_pipe).map_err(|_| KceError::SplicePushFailed)?;
        sq.push(&splice_to_socket).map_err(|_| KceError::SplicePushFailed)?;
    }
    ring.submit()?;
    Ok(())
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Entrada | Saída Esperada |
|----|------|---------|----------------|
| UT-001 | Alocação de Pipe | Solicitação de criação de pipe de kernel | Fds válidos de leitura/escrita retornados pelo OS |
| UT-002 | Splice Linkado | Encadeamento SQE | Execução sequencial dos dois passos de splice no CQE |
| UT-003 | Erro de Fd Inválido | Fd de destino nulo ou fechado | Retorno de CQE com erro de arquivo inválido sem crash |
| UT-004 | Redimensionamento de Pipe | Redimensionar pipe para 512KB | Operação de fcntl aceita com sucesso pelo SO |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | Streaming de Dataset Completo | Transferência de um arquivo vetorial de 100MB diretamente para o socket gRPC sem alocar memória no espaço de usuário do KCE |
| FT-002 | Recuperação de Buffer Congestionado | Se o socket de rede congestionar, o pipe deve reter a escrita e a CQE do segundo passo deve aguardar sem travar a thread |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Splice → Gossip (FT-029) | O motor do Gossip utiliza splice para transferir deltas de grafos pesados entre nós |
| IT-002 | Splice → Gateway (FT-031) | O Gateway envia respostas de busca contendo vetores brutos via splice direto ao cliente |

---

## 6. Formato CARE

**Context:** Roteamento e streaming de dados de banco de altíssima vazão em nós distribuídos do KCE, visando eliminar gargalos de largura de banda de barramento de RAM.

**Assumptions:** O kernel Linux suporta a operação de splice entre o sistema de arquivos onde o KineSQL está montado e sockets de rede ativos. O subsistema de rede do host suporta transmissões assíncronas em lote.

**Requirements:** R-001: Roteamento direto via `IORING_OP_SPLICE` | R-002: Encadeamento de SQEs | R-003: Fallback gracioso estável.

**Evidence:** Execução de testes de estresse de leitura e envio de dados comparando a taxa de vazão (Throughput) e consumo de CPU contra a arquitetura tradicional com buffer de usuário.

---

## 7. Critérios Não Funcionais

| Aspecto | Métrica | Alvo |
|---------|---------|------|
| Redução de Carga de CPU | Uso de CPU do processo em transmissão pesada | > 35% de redução frente ao fluxo com cópia |
| Latência de Início de Fluxo | Tempo para submeter o encadeamento no anel | < 0.2ms |
| Vazão Máxima de Rede | Throughput de transmissão de arquivos de banco | Saturation do limite físico da placa (ex: 10 Gbps) |

---

## 8. Qualidade e Métricas

**Sucesso:**
- Zero cópias feitas no espaço de usuário do processo do banco durante a transmissão de páginas de dados.
- Sincronização e integridade dos bytes transferidos em 100% dos testes.
- Cobertura de testes unitários superior a 80%.

**Falha (BLOQUEANTE):**
- Vazamento crônico de file descriptors de pipes alocados no kernel.
- Corrupção parcial ou desordem de pacotes nos dados transmitidos via splice.

---

## 9. Compatibilidade e Dependências

| Crate | Versão | Propósito |
|-------|--------|-----------|
| `io-uring` | ^0.6 | Suporte às instruções `Splice` do anel |
| `libc` | ^0.2 | Chamadas nativas de pipe e dimensionamento |

---

## 10. Rastreabilidade

| Tipo | ID | Descrição |
|------|----|-----------|
| Spec | FT-033-IO-URING-SPLICE | Esta especificação |
| Código | `crates/kce-storage/src/io_uring_splicer.rs` | Implementador do splice de kernel |

---

## 11. Roadmap

### MVP (Fase 1)
Implementação de roteador básico via chamada de sistema síncrona `splice` de POSIX para validação conceitual de performance do canal.

### Iteração 1 (Fase 2)
Implementação assíncrona baseada no anel do `io_uring` com encadeamento de submissão do pipeline de duas etapas (File $\rightarrow$ Pipe $\rightarrow$ Socket) e tratamento de erros do CQE.

### Iteração 2 (Fase 3)
Pool dinâmico de descritores de pipes persistidos na memória de kernel para reuso instantâneo, auto-ajuste de tamanho de buffer baseado na MTU de rede e integração completa com o gateway gRPC.
