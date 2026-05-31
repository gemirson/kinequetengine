# FT-032 — io_uring Bare-Metal Storage I/O

**Módulo:** Storage / Infraestrutura | **Versão:** v6.0 | **Prioridade:** P0 — Crítico  
**Artefato ID:** FT-032-IO-URING | **Atualização:** 2026-05-30

---

## 1. Contexto e Objetivo

Para atingir latências P99 consistentes de 3 a 10ms sob extrema carga de escrita no Write-Ahead Log (WAL) e leitura de páginas em disco, o KCE não pode sofrer com o overhead de chamadas de sistema síncronas bloqueantes (`write`, `read`, `fsync`) ou contenção de threads do sistema operacional.

O **io_uring Bare-Metal Storage I/O** introduz uma interface de I/O assíncrona de alto desempenho nativa do kernel Linux. Utilizando anéis compartilhados de submissão (Submission Queue - SQ) e conclusão (Completion Queue - CQ) mapeados diretamente na memória do espaço de usuário, o KCE submete escritas no WAL e leituras de páginas com zero cópias e zero chamadas de sistema no loop principal (usando Kernel Polling). Isso permite desempenho próximo ao hardware puro (*bare-metal*).

---

## 2. Critérios de Aceite (AC)

### Gerais
- [ ] AC-001: Compilação limpa sem warnings no compilador Rust.
- [ ] AC-002: Zero vazamentos de file descriptors durante o ciclo de vida do anel (`io_uring`).
- [ ] AC-003: Tratamento seguro de erros de sistema (ex: falta de memória no kernel ou limite de arquivos abertos).

### Específicos
- [ ] AC-010: Inicialização e configuração do anel `io_uring` utilizando buffers pré-registrados (`io_uring_register`) para evitar overhead de mapeamento de memória em tempo de escrita.
- [ ] AC-011: Escrita assíncrona no WAL com suporte a `O_DIRECT` para bypass do Page Cache do kernel, garantindo que o dado seja gravado diretamente no SSD/NVMe sem buffering redundante.
- [ ] AC-012: Encadeamento de operações: suporte à submissão casada de escrita seguida de barreira física (`IOSQE_IO_LINK` + `fsync`), permitindo que a persistência física ocorra em uma única transação do anel.
- [ ] AC-013: Modo **SQPOLL (Submission Queue Polling)** configurável: uma thread dedicada do kernel monitora a fila de submissão, permitindo I/O assíncrono com zero chamadas de sistema (`syscall-free`) na thread de execução do KCE.
- [ ] AC-014: Mecanismo de **Fallback Automático**: caso o KCE seja executado em sistemas não-Linux ou kernels inferiores a 5.1, o motor deve desativar o anel e migrar de forma transparente para um backend de I/O baseado em threadpool síncrono rápido (como `tokio::fs` ou file-per-thread com direct writes) sem corrupção ou interrupção do serviço.

---

## 3. Definition of Done (DoD)

- [ ] Struct `IoUringBackend` que inicializa o anel e gerencia as submissões de leitura e escrita.
- [ ] Implementação do worker de colheita assíncrona de eventos da CQ (Completion Queue).
- [ ] Mecanismo de fallback síncrono robusto validado em testes unitários.
- [ ] Testes de estresse com gravação concorrente no WAL comprovando zero corrupção de páginas.
- [ ] Benchmarks comparativos provando redução de pelo menos 30% na latência de escrita P99 em relação a gravações síncronas convencionais.
- [ ] Cobertura de testes unitários superior a 80%.

---

## 4. Exemplos de Uso

### Ciclo de Submissão de Escrita no WAL (Conceitual em Rust)

```rust
// Exemplo conceitual da submissão assíncrona de um bloco de dados no WAL usando a crate `io-uring`
use io_uring::{opcode, squeue, types};

pub fn submit_wal_write(
    ring: &mut io_uring::IoUring,
    fd: types::Fd,
    offset: u64,
    data: *const u8,
    len: u32,
) -> Result<(), KceError> {
    // 1. Criar a entrada de submissão (SQE) para escrita direta
    let write_op = opcode::Write::new(fd, data, len)
        .offset(offset)
        .build()
        .user_data(0x1024); // Identificador único da transação

    // 2. Enviar a SQE para a fila de submissão
    unsafe {
        ring.submission()
            .push(&write_op)
            .map_err(|_| KceError::IoUringSubmissionFailed)?;
    }

    // 3. Submeter ao kernel (se SQPOLL estiver inativo, executa chamada de sistema leve; se ativo, é instantâneo)
    ring.submit()?;
    Ok(())
}
```

---

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Entrada | Saída Esperada |
|----|------|---------|----------------|
| UT-001 | Inicialização do Anel | Configuração padrão de fila (tamanho: 256) | Instanciação com sucesso do file descriptor do anel |
| UT-002 | Escrita e Confirmação CQE | Submissão de bloco de bytes | CQE retornando com sucesso e `user_data` idêntico |
| UT-003 | Ativação do Fallback | Execução em ambiente sem suporte a `io_uring` | O sistema carrega o driver síncrono padrão e emite um warning de telemetria |
| UT-004 | Registro de Buffers | Pré-alocação de buffer de 4KB | Registro aceito sem vazamento de página física de memória |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado Esperado |
|----|---------|---------------------|
| FT-001 | Teste de Carga de Escritas no WAL | Gravação de 100k entradas sequenciais com `io_uring` ativa e validação por checksum no final para atestar 0% de corrupção |
| FT-002 | Recuperação sob Queda no Kernel | Interrupção forçada da colheita de CQEs não deve corromper os dados já persistidos fisicamente |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | `io_uring` → KineSQL (FT-005) | As rotinas de gravação de página do KineSQL redirecionam a escrita de dados para o driver do `io_uring` |
| IT-002 | `io_uring` → Observabilidade (FT-007) | Monitoramento das métricas de ocupação das filas SQ/CQ integradas no dashboard |

---

## 6. Formato CARE

**Context:** Escrita ultra-rápida de persistência transacional com latências previsíveis e otimização máxima de uso de hardware em ambientes de servidores Linux bare-metal.

**Assumptions:** O kernel Linux de destino oferece suporte adequado à API `io_uring` (idealmente Kernel $\ge$ 5.6 para suporte a SQPOLL estável). O storage subjacente é SSD/NVMe de baixa latência compatível com escritas direct-access (`O_DIRECT`).

**Requirements:** R-001: Filas SQ/CQ mapeadas em memória | R-002: Kernel Polling SQPOLL opcional | R-003: Fallback automático transparente | R-004: Encadeamento de barreiras físicas (linked ops).

**Evidence:** Execução de scripts de benchmark comparativos sob concorrência intensa com as estatísticas de latência de cauda gravadas na telemetria.

---

## 7. Critérios Não Funcionais

| Aspecto | Métrica | Alvo |
|---------|---------|------|
| Overhead de Syscall | Redução de chamadas de sistema no loop principal | > 90% de economia de Syscalls sob alta concorrência |
| Latência P99 do WAL | Latência física de gravação + sincronização | < 3ms |
| Vazão Máxima | Páginas escritas por segundo em NVMe | > 500 MB/s sustentados |

---

## 8. Qualidade e Métricas

**Sucesso:**
- Conclusão com sucesso de todas as escritas no WAL em testes de concorrência com 0 corrupções de páginas.
- Redução comprovada do desvio padrão de latência em cenários de alta pressão de escrita.
- Cobertura de testes unitários superior a 80%.

**Falha (BLOQUEANTE):**
- Corrupção de páginas ou perda de dados em escritas diretas.
- Deadlock ou travamento infinito na fila de colheita CQE.

---

## 9. Compatibilidade e Dependências

| Crate | Versão | Propósito |
|-------|--------|-----------|
| `io-uring` | ^0.6 | Interface nativa Rust para io_uring do Linux |
| `nix` | ^0.27 | Operações POSIX e flags `O_DIRECT` |

---

## 10. Rastreabilidade

| Tipo | ID | Descrição |
|------|----|-----------|
| Spec | FT-032-IO-URING | Esta especificação |
| Código | `crates/kce-storage/src/io_uring_backend.rs` | Implementação do I/O assíncrono |

---

## 11. Roadmap

### MVP (Fase 1)
Driver assíncrono simples baseado em anel de escrita básica. Sem pré-registro de buffers ou SQPOLL.

### Iteração 1 (Fase 2)
Implementação de escrita encadeada (Linked Write + Linked Fsync) com suporte a `O_DIRECT` para bypass do cache do sistema. Fallback dinâmico estável.

### Iteração 2 (Fase 3)
Ativação de Kernel Polling (SQPOLL) em tempo de execução para operações zero-syscall no loop principal e pooling dinâmico de buffers registrados.
