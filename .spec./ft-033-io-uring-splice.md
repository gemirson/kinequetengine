# FT-033 — io_uring Zero-Copy Splice (Storage-to-Network Bypass)

**Module:** Storage / Interface | **Version:** v6.0 | **Priority:** P0 — Critical  
**Artifact ID:** FT-033-IO-URING-SPLICE | **Update:** 2026-05-30

---

## 1. Context and Objective

In high-throughput search traffic and distributed synchronization, KCE needs to stream large volumes of data (KineSQL pages, WAL logs, and vector blocks) from on-disk persistence directly to the network (Gateway and other mesh nodes). Conventional flow involves reading from disk to a user buffer in RAM and then writing that buffer to the network socket, which consumes redundant CPU cycles and saturates memory bandwidth.

As an extension of the **FT-032 (io_uring Bare-Metal)** feature, **io_uring Zero-Copy Splice** uses the Linux kernel's `IORING_OP_SPLICE` operation. This feature allows you to directly connect the physical file descriptor (KineSQL) to the network descriptor (TCP/UDP) within the kernel itself through an intermediate buffer pipe, transmitting data with **zero-copy**. This completely eliminates data traffic in the KCE user space and frees up the CPU to process searches and cognitive analysis.

---

## 2. Acceptance Criteria (AC)

### General
- [ ] AC-001: Build without warnings or errors in the Rust ecosystem.
- [ ] AC-002: Automatic release of resources (kernel pipes and sockets) after splice completion.
- [ ] AC-003: Tolerance of network connection losses during transfer (partial frame discard with gRPC/TCP clean error).

### Specifics
- [ ] AC-010: Integrated registration in the `io_uring` ring of database file descriptors (KineSQL) and active socket connection descriptors (GRPC/Gossip UDP Gateway).
- [ ] AC-011: Executing `IORING_OP_SPLICE` calls asynchronously, linking the source file to the socket descriptor via pre-allocated kernel pipes.
- [ ] AC-012: Chaining of operations: support chained sending of gRPC-Web header (`write` on the ring) followed by the data body via splice (`splice` on the ring) using `IOSQE_IO_LINK`.
- [ ] AC-013: Kernel Buffer Optimization: Dynamic resizing of internal pipe buffer size (`fcntl` F_SETPIPE_SZ) to support data blocks up to 1MB without fragmentation.
- [ ] AC-014: **Transparent Fallback Mechanism**: if the kernel does not support the `splice` operation of specific files or the OS is not compatible, it reverts to asynchronous reading in the user buffer (`read`) followed by writing to the socket (`write`) in an invisible way for the application.

---

## 3. Definition of Done (DoD)

- [ ] `IoUringSplicer` module implemented and coupled to the `IoUringBackend` main ring.
- [ ] Securely managed pipe buffer pool in kernel space.
- [ ] Integrity validation tests proving that transmission via splice sends bytes identical to those originally recorded.
- [ ] Comparative benchmarks under extreme concurrent load demonstrating greater than 40% reduction in CPU cycle consumption per MB transmitted.
- [ ] Unit test coverage greater than 80%.

---

## 4. Usage Examples

### Ring Splice Submission Structure (Conceptual in Rust)

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

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Entry | Expected Output |
|----|------|---------|----------------|
| UT-001 | Pipe Allocation | Kernel pipe creation request | Valid read/write data returned by the OS |
| UT-002 | Linked Splice | SQE chaining | Sequential execution of the two splice steps in CQE |
| UT-003 | Invalid Fd Error | Null or closed destination FD | CQE return with invalid file error without crash |
| UT-004 | Pipe Resizing | Resize pipe to 512KB | fcntl operation successfully accepted by OS |

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | Full Dataset Streaming | Transferring a 100MB vector file directly to the gRPC socket without allocating memory in KCE user space |
| FT-002 | Congested Buffer Recovery | If the network socket becomes congested, the pipe must hold the write and the CQE of the second step must wait without blocking the thread |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Splice → Gossip (FT-029) | Gossip engine uses splice to transfer heavy graph deltas between nodes |
| IT-002 | Splice → Gateway (FT-031) | The Gateway sends search responses containing raw vectors via splice directly to the client |

---

## 6. CARE Format

**Context:** Routing and streaming high-throughput database data across distributed KCE nodes to eliminate RAM bus bandwidth bottlenecks.

**Assumptions:** The Linux kernel supports splice operation between the file system where KineSQL is mounted and active network sockets. The host network subsystem supports asynchronous batch transmissions.

**Requirements:** R-001: Direct routing via `IORING_OP_SPLICE` | R-002: Chaining of SQEs | R-003: Stable graceful fallback.

**Evidence:** Running stress tests on reading and sending data comparing throughput and CPU consumption against traditional user-buffered architecture.

---

## 7. Non-Functional Criteria

| Appearance | Metric | Target |
|---------|---------|------|
| CPU Load Reduction | Process CPU usage in heavy streaming | > 35% reduction compared to flow with copy |
| Flow Start Latency | Time to commit thread to ring | < 0.2ms |
| Maximum Network Flow | Bank file transmission throughput | Saturation of the physical limit of the card (ex: 10 Gbps) |

---

## 8. Quality and Metrics

**Success:**
- Zero copies made in user space of the database process when transmitting data pages.
- Synchronization and integrity of transferred bytes in 100% of tests.
- Unit test coverage greater than 80%.

**Failure (BLOCKING):**
- Chronic leakage of file descriptors from pipes allocated in the kernel.
- Partial corruption or packet disorder in data transmitted via splice.

---

## 9. Compatibility and Dependencies

| Crate | Version | Purpose |
|-------|--------|-----------|
| `io-uring` | ^0.6 | Ring `Splice` instruction support |
| `libc` | ^0.2 | Native pipe and scaling calls |

---

## 10. Traceability

| Type | ID | Description |
|------|----|-----------|
| Spec | FT-033-IO-URING-SPLICE | This specification |
| Code | `crates/kce-storage/src/io_uring_splicer.rs` | Kernel splice implementer |

---

## 11. Roadmap

### MVP (Phase 1)
Basic router implementation via POSIX synchronous `splice` system call for conceptual validation of channel performance.

### Iteration 1 (Phase 2)
Ring-based asynchronous implementation of `io_uring` with two-step pipeline submission chaining (File $\rightarrow$ Pipe $\rightarrow$ Socket) and CQE error handling.

### Iteration 2 (Phase 3)
Dynamic pooling of pipe descriptors persisted in kernel memory for instant reuse, auto-tuning buffer size based on network MTU, and full integration with the gRPC gateway.
