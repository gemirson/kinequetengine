# FT-031 — Hematoencephalic Gateway

**Module:** Interface | **Version:** v6.0 | **Priority:** P0 — Critical  
**Artifact ID:** FT-031-HEMATOENCEPHALIC-GATEWAY | **Update:** 2026-05-30

---

## 1. Context and Objective

The **Hematoencephalic Gateway** acts as the physical and logical barrier (Blood-Brain Barrier) between the external corporate network and KCE's high-speed distributed fabric. It exposes a unified **gRPC-Web** and **WebSockets** interface that translates client requests into internal Swarm protocol messages, forwards vector search queries directly to nodes hosting active data partitions (shards), and transmits real-time telemetry streams (node ​​evolution, pheromones, and threats) to visualization dashboards.

---

## 2. Acceptance Criteria (AC)

### General
- [ ] AC-001: Clean build in Rust.
- [ ] AC-002: Full support for HTTP/2 calls and HTTP/1.1 fallback via gRPC-Web.
- [ ] AC-003: Integrated protection against denial of service attacks and packet flooding.

### Specifics
- [ ] AC-010: Tenant authentication middleware via API Key at gateway level.
- [ ] AC-011: Intelligent query routing: based on the `tenant_id` key and the vector hash, forwards the search request directly to the corresponding primary node (FT-028) using reused TCP connections (Connection Pooling).
- [ ] AC-012: Support bidirectional streaming via WebSockets for real-time telemetry from:
  * State of homeostasis of the nodes.
  * Activity of ants searching for and evaporating pheromones.
  * Alerts of immunological threats and antigen blocks.
- [ ] AC-013: Transparent fallback of connections: if the destination node fails, redirects the query to the secondary replica node in less than 10ms.
- [ ] AC-014: Optional payload compression via gzip or zstd to reduce mobile bandwidth consumption.
- [ ] AC-015: Maximum latency introduced by the gateway in the vector query is less than 0.5ms.

---

## 3. Definition of Done (DoD)

- [ ] Implementation of the gRPC-Web proxy using `tonic` and `axum`.
- [ ] Connection and maintenance of WebSockets sessions for transmitting monitoring data.
- [ ] Load tests demonstrating throughput of up to 5,000 requests per second per gateway instance.
- [ ] Secure handling of client disconnections and file descriptor leaks.
- [ ] Unit test coverage greater than 80%.

---

## 4. Usage Examples

### gRPC Service Subscription (Protobuf)

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

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Entry | Expected Output |
|----|------|---------|----------------|
| UT-001 | API Key Validation | Request without header `x-api-key` | Returns gRPC error `UNAUTHENTICATED` |
| UT-002 | Gateway Router | tenant=1, vector_hash=123 | Route resolved to the correct IP of the corresponding node |
| UT-003 | Redirection on Failure | Offline target node | Successfully redirects and returns from the replica node |

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | gRPC-Web Load Testing | 100,000 concurrent gRPC calls executed with <0.1% error rate and stable latency |
| FT-002 | Event Streaming | Connected WebSocket client continuously receives telemetry data without message loss |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Gateway → Shard Router (FT-028) | The gateway uses the shard routing table updated by the cluster advisor |
| IT-002 | Gateway → Resilience (FT-009) | The gateway applies backpressure and circuit breakers when nodes are overloaded |

---

## 6. CARE Format

**Context:** Single point of entry and validation of requests from external clients, acting as a security barrier and real-time traffic orchestrator for the KCE distributed fabric.

**Assumptions:** The client supports gRPC-Web or WebSocket traffic. The upstream load balancer directs HTTP/2 connections to the gateway port (default: 8080).

**Requirements:** R-001: Intelligent shard routing | R-002: Telemetry streaming support | R-003: Tenant Authentication.

**Evidence:** Running stress tests with `ghz` (gRPC benchmark tool) proving low latency under extreme concurrent load.

---

## 7. Non-Functional Criteria

| Appearance | Metric | Target |
|---------|---------|------|
| Additional Latency (Proxy Overhead) | Latency added by translation and routing | < 0.5ms |
| Limit Concurrent WebSocket Connections | Active concurrent connections per gateway node | > 10,000 |
| Maximum Telemetry Throughput | Events transmitted per second | > 1,000 events/sec |

---

## 8. Quality and Metrics

**Success:**
- Zero packet losses or unexpected disconnections when testing 24-hour persistent WebSockets connections.
- Latency overhead of less than 0.5ms.
- Unit test coverage greater than 80%.

**Failure (BLOCKING):**
- Resource leaks (memory leaks or file descriptors not closed when disconnecting clients).
- Tenant authentication bypass.

---

## 9. Compatibility and Dependencies

| Crate | Version | Purpose |
|-------|--------|-----------|
| `tonic` | ^0.10 | Implementation of gRPC and gRPC-Web |
| `tokio-tungstenite` | ^0.20 | Asynchronous WebSocket Protocol |

---

## 10. Traceability

| Type | ID | Description |
|------|----|-----------|
| Spec | FT-031-HEMATOENCEPHALIC-GATEWAY | This specification |
| Design | `docs/kce_distributed_architecture.md` | Hematoencephalic Gateway Design |

---

## 11. Roadmap

### MVP (Phase 1)
Simple Axum REST proxy with basic authentication and fixed routing to local nodes. No support for gRPC-Web or WebSockets streaming.

### Iteration 1 (Phase 2)
Implementation of the gRPC-Web server (`tonic`) and dynamic intelligent routing of shards. Basic WebSockets connectivity for simplified metrics.

### Iteration 3 (Phase 3)
Full 3D telemetry streaming, advanced compression, flow control and adaptive rate-limiting integrated with cluster homeostasis.
