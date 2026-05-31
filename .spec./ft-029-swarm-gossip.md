# FT-029 — Swarm Gossip Protocol

**Module:** Infrastructure | **Version:** v6.0 | **Priority:** P0 — Critical  
**Artifact ID:** FT-029-SWARM-GOSSIP | **Update:** 2026-05-30

---

## 1. Context and Objective

Synchronizing metadata, optimized search paths, and immune threat profiles across a distributed network cannot rely on expensive, persistent TCP connections. **Swarm Gossip Protocol** defines a low-overhead asynchronous peer-to-peer (P2P) communication protocol based on compressed binary packets transmitted via **UDP**. It is responsible for synchronizing ACO pheromones, incremental updates of the cognitive graph and propagating immune threat signatures (Herd Immunity) at the cluster level in a few milliseconds.

---

## 2. Acceptance Criteria (AC)

### General
- [ ] AC-001: Clean build without unnecessary OS dependencies.
- [ ] AC-002: Secure parsing of binary packets (zero buffer overflows, explicit bounds checking).
- [ ] AC-003: Zero memory allocation in main UDP listening loop.

### Specifics
- [ ] AC-010: Strict binary packet format, containing Magic Byte (`0x03`), Node ID (16 bytes), Sequence Number (4 bytes), Message Type (1 byte), Causal Watermark (8 bytes) and Variable Payload.
- [ ] AC-011: Supported message types:
  * `0x01` (Pheromone Sync): edge pheromone weight propagation.
  * `0x02` (Antigen Sync): transmission of threat signatures.
  * `0x03` (Graph Delta Sync): sending graph structural deltas.
- [ ] AC-012: **Herd Immunity** Mechanism: upon receiving an Antigen Sync packet, the node must update its immunological memory immediately and filter future requests with the received signature.
- [ ] AC-013: Packet clutter handling: use of Vector Clocks / Causal Watermarks to discard outdated packets.
- [ ] AC-014: Limited epidemic broadcast: each node forwards the received information to $K$ random neighbors (default: $K=3$) to avoid broadcast storms.

---

## 3. Definition of Done (DoD)

- [ ] Implementation of asynchronous UDP listening socket and message dispatcher in Tokyo.
- [ ] Manual binary serializer and deserializer (zero-copy parsing).
- [ ] Contention-free concurrency control mechanism for writing received messages.
- [ ] Unit tests with clutter and packet loss simulation.
- [ ] Unit test coverage greater than 80%.

---

## 4. Usage Examples

### Antigen Sync Binary Payload (Conceptual Representation)

**Binary packet sent over the network (represented in Hex):**
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

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Entry | Expected Output |
|----|------|---------|----------------|
| UT-001 | Message Serialization | Filled Structure | Strict Binary Byte Vector |
| UT-002 | Deserialization and Validation | Corrupt byte vector (missing bytes) | Returns `Err(InvalidPacket)` without panic |
| UT-003 | Causal Obsolete Watermark | Message received with watermark < location | Message silently discarded |
| UT-004 | Random Neighbor Selection | List of 10 nodes, $K=3$ | Returns exactly 3 distinct nodes stochastically |

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | Threat Propagation | Threat injection on Node 1 propagates and blocks similar queries on Node 3 in less than 100ms |
| FT-002 | Network Partition Recovery | After isolated node reconnection, accumulated messages with larger watermarks update the local state |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Gossip → AIS Memory (FT-018) | Package of type `0x02` inserts an active record into Antigen Memory |
| IT-002 | Gossip → ACO Evaporation (FT-014) | Synchronizes pheromone decays on shared edges |

---

## 6. CARE Format

**Context:** High-performance, asynchronous and resilient to physical failure communication of inter-datacenter network links in KCE clusters.

**Assumptions:** The physical network allows UDP traffic on the configured port (default: 9000). Occasional loss of individual telemetry packets (such as pheromones) is tolerated and corrected in subsequent cycles.

**Requirements:** R-001: UDP-based communication | R-002: Secure manual binary parser | R-003: Immediate Herd Immunity | R-004: Causal control by watermarks.

**Evidence:** Execution of network chaos tests (artificial latency + 20% packet loss) proving final data convergence.

---

## 7. Non-Functional Criteria

| Appearance | Metric | Target |
|---------|---------|------|
| Dissemination Latency | Time for 90% of nodes to receive a threat update | < 50ms |
| Network Overhead | Average bandwidth consumption per node at idle | < 10 KB/s |
| Package Security | Preventing replay attacks | Disposal by watermark + lightweight cryptographic signature |

---

## 8. Quality and Metrics

**Success:**
- Complete convergence of pheromones after network stabilization.
- Zero memory leaks or panics in the packet reception loop.
- Unit test coverage greater than 80%.

**Failure (BLOCKING):**
- Infinite relay loops (broadcast storms) that saturate the network.
- Buffer overflow or memory panic when reading corrupted payloads.

---

## 9. Compatibility and Dependencies

| Crate/Tool | Version | Purpose |
|--------------------|--------|-----------|
| `tokio` | ^1.35 | UDP asynchronous sockets |
| `crc32fast` | ^1.3 | Packet validation checksum |

---

## 10. Traceability

| Type | ID | Description |
|------|----|-----------|
| Spec | FT-029-SWARM-GOSSIP | This specification |
| Design | `docs/kce_distributed_architecture.md` | Swarm Gossip Protocol |

---

## 11. Roadmap

### MVP (Phase 1)
Basic sending of heartbeats via UDP for simple detection of the presence of neighboring nodes. No encryption or watermarks.

### Iteration 1 (Phase 2)
Implementation of Pheromone Sync and Antigen Sync with causal control based on watermarks and restricted epidemic pass-through.

### Iteration 2 (Phase 3)
Optional symmetric encryption on payloads, complete control of structural graph deltas via CRDTs and integration with distributed homeostasis.
