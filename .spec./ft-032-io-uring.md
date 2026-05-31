# FT-032 — io_uring Bare-Metal Storage I/O

**Module:** Storage / Infrastructure | **Version:** v6.0 | **Priority:** P0 — Critical  
**Artifact ID:** FT-032-IO-URING | **Update:** 2026-05-30

---

## 1. Context and Objective

To achieve consistent P99 latencies of 3 to 10 ms under extreme load writing to the Write-Ahead Log (WAL) and reading pages to disk, KCE cannot suffer from the overhead of blocking synchronous system calls (`write`, `read`, `fsync`) or OS thread contention.

**io_uring Bare-Metal Storage I/O** introduces a high-performance asynchronous I/O interface native to the Linux kernel. Using shared Submission Queue - SQ and Completion Queue - CQ rings mapped directly into user space memory, KCE submits WAL writes and page reads with zero copies and zero system calls in the main loop (using Kernel Polling). This allows performance close to pure hardware (*bare-metal*).

---

## 2. Acceptance Criteria (AC)

### General
- [ ] AC-001: Clean compilation without warnings in the Rust compiler.
- [ ] AC-002: Zero leaks of file descriptors during ring lifecycle (`io_uring`).
- [ ] AC-003: Safe handling of system errors (e.g., lack of memory in the kernel or limit of open files).

### Specifics
- [ ] AC-010: Initialization and configuration of the `io_uring` ring using pre-registered buffers (`io_uring_register`) to avoid memory mapping overhead at write time.
- [ ] AC-011: Asynchronous writing to WAL with `O_DIRECT` support to bypass the kernel's Page Cache, ensuring that data is written directly to the SSD/NVMe without redundant buffering.
- [ ] AC-012: Chaining of operations: support for married writing submission followed by a physical barrier (`IOSQE_IO_LINK` + `fsync`), allowing physical persistence to occur in a single ring transaction.
- [ ] AC-013: **SQPOLL (Submission Queue Polling)** mode configurable: a dedicated kernel thread monitors the submission queue, allowing asynchronous I/O with zero system calls (`syscall-free`) on the KCE execution thread.
- [ ] AC-014: **Automatic Fallback Engine**: If KCE runs on non-Linux systems or kernels lower than 5.1, the engine must disable the ring and transparently migrate to a fast synchronous threadpool-based I/O backend (such as `tokio::fs` or file-per-thread with direct writes) without corruption or service interruption.

---

## 3. Definition of Done (DoD)

- [ ] Struct `IoUringBackend` that initializes the ring and manages read and write submissions.
- [ ] Implementation of the CQ (Completion Queue) asynchronous event harvesting worker.
- [ ] Robust synchronous fallback mechanism validated in unit tests.
- [ ] Stress tests with concurrent recording in WAL proving zero page corruption.
- [ ] Comparative benchmarks proving at least a 30% reduction in P99 write latency compared to conventional synchronous writes.
- [ ] Unit test coverage greater than 80%.

---

## 4. Usage Examples

### Writing Submission Cycle in WAL (Conceptual in Rust)

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

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Entry | Expected Output |
|----|------|---------|----------------|
| UT-001 | Ring Boot | Default queue configuration (size: 256) | Successful instantiation of the ring descriptor file |
| UT-002 | CQE Writing and Confirmation | Byte block submission | CQE returning successfully and `user_data` identical |
| UT-003 | Fallback activation | Execution in an environment without `io_uring` support | The system loads the default synchronous driver and issues a telemetry warning |
| UT-004 | Buffer Register | 4KB buffer pre-allocation | Record accepted without physical memory page leak |

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | WAL Write Load Test | Recording 100k sequential entries with `io_uring` active and checksum validation at the end to ensure 0% corruption |
| FT-002 | Kernel Crash Recovery | Forced interruption of CQE harvesting should not corrupt data already physically persisted |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | `io_uring` → KineSQL (FT-005) | KineSQL page writing routines redirect data writing to the `io_uring` driver |
| IT-002 | `io_uring` → Observability (FT-007) | Monitoring occupancy metrics for SQ/CQ queues integrated into the dashboard |

---

## 6. CARE Format

**Context:** Ultra-fast transactional persistence writing with predictable latencies and maximum hardware usage optimization in bare-metal Linux server environments.

**Assumptions:** The target Linux kernel adequately supports the `io_uring` API (ideally Kernel $\ge$ 5.6 for stable SQPOLL support). The underlying storage is low latency SSD/NVMe compatible with direct-access writes (`O_DIRECT`).

**Requirements:** R-001: Memory-mapped SQ/CQ queues | R-002: Kernel Polling SQPOLL optional | R-003: Transparent Automatic Fallback | R-004: Chaining of physical barriers (linked ops).

**Evidence:** Executing comparative benchmark scripts under intense contention with tail latency statistics recorded in telemetry.

---

## 7. Non-Functional Criteria

| Appearance | Metric | Target |
|---------|---------|------|
| Syscall overhead | Reduced system calls in the main loop | > 90% Syscall savings under high concurrency |
| WAL P99 Latency | Physical write latency + synchronization | < 3ms |
| Maximum Flow | Pages written per second on NVMe | > 500 MB/s sustained |

---

## 8. Quality and Metrics

**Success:**
- Successful completion of all WAL writes in concurrency tests with 0 page corruptions.
- Proven reduction in latency standard deviation in high writing pressure scenarios.
- Unit test coverage greater than 80%.

**Failure (BLOCKING):**
- Page corruption or data loss in direct writes.
- Deadlock or infinite lock in the CQE harvest queue.

---

## 9. Compatibility and Dependencies

| Crate | Version | Purpose |
|-------|--------|-----------|
| `io-uring` | ^0.6 | Native Rust interface for Linux io_uring |
| `nix` | ^0.27 | POSIX operations and `O_DIRECT` flags |

---

## 10. Traceability

| Type | ID | Description |
|------|----|-----------|
| Spec | FT-032-IO-URING | This specification |
| Code | `crates/kce-storage/src/io_uring_backend.rs` | Asynchronous I/O implementation |

---

## 11. Roadmap

### MVP (Phase 1)
Simple asynchronous driver based on basic write ring. No pre-registration of buffers or SQPOLL.

### Iteration 1 (Phase 2)
Implementation of chained writing (Linked Write + Linked Fsync) with support for `O_DIRECT` to bypass the system cache. Stable dynamic fallback.

### Iteration 2 (Phase 3)
Activation of Kernel Polling (SQPOLL) at runtime for zero-syscall operations in the main loop and dynamic pooling of registered buffers.
