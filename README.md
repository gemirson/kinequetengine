# KineQuetEngine (KCE)

**Bio-Inspired Context Engine for AI/ML in Rust**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-258%20passing-brightgreen)]()
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)]()
[![CI](https://github.com/kinecontext/kce/actions/workflows/ci.yml/badge.svg)]()

KCE is a modular cognitive engine that processes semantic context through bio-inspired algorithms. It combines vector search, semantic graph expansion, entity lifecycle management, and mRNA-style executable context generation -- enhanced by Ant Colony Optimization (ACO) for emergent routing and an Artificial Immune System (AIS) for anomaly detection and self-regulation.

---

## Architecture

```
                         ┌──────────────────────────┐
                         │       kce-server          │
                         │    (binary entry point)    │
                         └────────────┬─────────────┘
                                      │
                         ┌────────────▼─────────────┐
                         │        kce-api            │
                         │  Axum REST: /query        │
                         │  /encode, /action         │
                         │  /metrics, /health        │
                         │  /admin/*                 │
                         └────────────┬─────────────┘
                                      │
              ┌───────────────────────┼───────────────────────┐
              │                       │                       │
   ┌──────────▼──────────┐ ┌─────────▼──────────┐ ┌──────────▼──────────┐
   │    kce-pipeline      │ │     kce-aco         │ │     kce-ais          │
   │  10-stage cognitive  │ │  Ant Colony         │ │  Artificial Immune   │
   │  orchestrator        │ │  Optimization       │ │  System              │
   │                      │ │  - pheromone route  │ │  - anomaly detect    │
   │                      │ │  - evaporation      │ │  - antigen memory    │
   │                      │ │  - ant exploration  │ │  - self/non-self     │
   │                      │ │  - colony optim.    │ │  - network regulate  │
   └──────────┬───────────┘ └─────────┬──────────┘ └──────────┬───────────┘
              │                       │                       │
              └───────────────────────┼───────────────────────┘
                                      │
         ┌────────────────────────────┼────────────────────────────┐
         │                            │                            │
┌────────▼────────┐  ┌───────────────▼──────────┐  ┌─────────────▼─────────┐
│   kce-retrieval  │  │      kce-graph           │  │      kce-ecma         │
│  Vector search   │  │  Semantic graph          │  │  Entity lifecycle     │
│  (cosine, SIMD)  │  │  expansion & BFS/DFS     │  │  (Stem->Spec->        │
│                  │  │                          │  │   Mature->Apoptosis)  │
└────────┬─────────┘  └───────────┬──────────────┘  └─────────┬─────────────┘
         │                        │                            │
         │               ┌────────▼────────┐         ┌────────▼────────┐
         │               │    kce-mce      │         │   kce-metrics    │
         │               │  mRNA Cognitive  │         │  cosine, Wasser- │
         │               │  Engine          │         │  stein, Sinkhorn │
         │               │  encode/decode/  │         │  latency p99     │
         │               │  execute         │         │                  │
         │               └─────────────────┘         └──────────────────┘
         │
┌────────▼─────────┐
│   kce-storage     │
│  KineSQL embedded │
│  - WAL + fsync    │
│  - mmap reader    │
│  - CRC32 checksum │
│  - page manager   │
└──────────────────┘
```

### Crate Dependency Overview

| Crate | Purpose |
|---|---|
| `kce-core` | Shared types, errors, traits |
| `kce-retrieval` | Hybrid vector search (cosine SIMD, prime GCD) |
| `kce-graph` | Semantic graph expansion (BFS/DFS, edge weights) |
| `kce-ecma` | Entity lifecycle state machine (Stem -> Specialized -> Mature -> Apoptosis) |
| `kce-mce` | mRNA Cognitive Engine (encode context, decode intent, execute action) |
| `kce-storage` | KineSQL embedded storage with WAL, mmap, CRC32 checksums |
| `kce-pipeline` | 10-stage cognitive orchestrator |
| `kce-api` | Axum REST API with OpenAPI/Swagger |
| `kce-aco` | Ant Colony Optimization (pheromone routing, evaporation, exploration) |
| `kce-ais` | Artificial Immune System (detection, classification, regulation) |
| `kce-metrics` | Distance metrics (cosine, Wasserstein, Sinkhorn) + latency histogram |
| `kce-server` | Binary entry point |

---

## Quickstart

```bash
# Build release binary
cargo build --release

# Run the server
./target/release/kce-server

# Run with tracing
RUST_LOG=info ./target/release/kce-server
```

### Docker

```bash
docker compose up --build
```

---

## API

The server exposes a REST API on the configured port (default: 3000).

### Health Check

```bash
curl http://localhost:3000/health
# => {"status":"ok","version":"6.0.0","uptime_seconds":42}
```

### Query (Semantic Search)

```bash
curl -X POST http://localhost:3000/query \
  -H "Content-Type: application/json" \
  -d '{
    "text": "rust memory safety patterns",
    "top_k": 5,
    "filters": {}
  }'
```

### Encode (Generate Context)

```bash
curl -X POST http://localhost:3000/encode \
  -H "Content-Type: application/json" \
  -d '{
    "text": "distributed systems consensus",
    "ttl": 3600,
    "priority": 0.8
  }'
```

### Action (Execute Intent)

```bash
curl -X POST http://localhost:3000/action \
  -H "Content-Type: application/json" \
  -d '{
    "context_id": "ctx-abc123",
    "intent": "summarize",
    "params": {}
  }'
```

### Metrics

```bash
curl http://localhost:3000/metrics
```

### Admin

```bash
# List entities
curl http://localhost:3000/admin/entities

# Get entity detail
curl http://localhost:3000/admin/entities/{id}

# System status
curl http://localhost:3000/admin/status
```

---

## Features (FT-001 through FT-024)

### Core Engine

| ID | Feature | Description |
|---|---|---|
| FT-001 | Retrieval Engine | Hybrid vector search with cosine SIMD and prime GCD |
| FT-002 | Graph Engine | Semantic graph for context expansion |
| FT-003 | ECMA Engine | Entity lifecycle: Stem -> Specialized -> Mature -> Apoptosis |
| FT-004 | MCE Engine | Context to executable action (mRNA) |
| FT-005 | KineSQL Storage | Embedded storage with WAL, fsync, checksum |
| FT-006 | API Gateway | REST API with Axum, OpenAPI, Swagger UI |
| FT-011 | Pipeline Robust | 10-stage cognitive pipeline orchestration |

### Operations

| ID | Feature | Description |
|---|---|---|
| FT-007 | Observability | Tracing + Prometheus metrics |
| FT-008 | Security | API Key auth, multi-tenant, input validation |
| FT-009 | Resilience | Retry, timeout, circuit breaker, backpressure |
| FT-010 | Concurrency | Thread safety with Arc<RwLock>, parking_lot |
| FT-012 | Deploy Infrastructure | Docker, 12-factor config, healthcheck |

### ACO -- Ant Colony Optimization

| ID | Feature | Description |
|---|---|---|
| FT-013 | Pheromone Routing | Context routing via digital pheromones |
| FT-014 | Evaporation | Temporal pheromone decay |
| FT-015 | Ant Exploration | Parallel queries exploring divergent paths |
| FT-016 | Colony Optimization | Global context optimization via multi-cycle |

### AIS -- Artificial Immune System

| ID | Feature | Description |
|---|---|---|
| FT-017 | Immune Detection | Anomaly detection inspired by antigen recognition |
| FT-018 | Antigen Memory | Persistent memory of critical patterns |
| FT-019 | Response Amplifier | Priority amplification in critical contexts |
| FT-020 | Self/Non-Self | Classification of trusted vs suspicious context |
| FT-021 | Adaptive Mutation | Controlled embedding mutation for exploration |
| FT-022 | Network Regulation | Global equilibrium controller (homeostasis) |

### Distance Metrics

| ID | Feature | Description |
|---|---|---|
| FT-023 | Optimal Transport | Wasserstein/Sinkhorn context distance |
| FT-024 | Latency p99 | Latency histogram with percentile tracking |

---

## Specifications

Full feature specifications are in the [`.spec./`](.spec./) directory:

- [FT-001: Retrieval Engine](.spec./ft-001-retrieval-engine.md)
- [FT-002: Graph Engine](.spec./ft-002-graph-engine.md)
- [FT-003: ECMA Engine](.spec./ft-003-ecma-engine.md)
- [FT-004: MCE Engine](.spec./ft-004-mce-engine.md)
- [FT-005: KineSQL Storage](.spec./ft-005-kinesql-storage.md)
- [FT-006: API Gateway](.spec./ft-006-api-gateway.md)
- [FT-007: Observability](.spec./ft-007-observabilidade.md)
- [FT-008: Security](.spec./ft-008-seguranca.md)
- [FT-009: Resilience](.spec./ft-009-resiliencia.md)
- [FT-010: Concurrency](.spec./ft-010-concorrencia.md)
- [FT-011: Pipeline Robust](.spec./ft-011-pipeline-robusto.md)
- [FT-012: Deploy Infrastructure](.spec./ft-012-deploy-infra.md)
- [FT-013: Pheromone Routing](.spec./ft-013-pheromone-routing.md)
- [FT-014: Evaporation](.spec./ft-014-evaporation.md)
- [FT-015: Ant Exploration](.spec./ft-015-ant-exploration.md)
- [FT-016: Colony Optimization](.spec./ft-016-colony-optimization.md)
- [FT-017: Immune Detection](.spec./ft-017-immune-detection.md)
- [FT-018: Antigen Memory](.spec./ft-018-antigen-memory.md)
- [FT-019: Response Amplifier](.spec./ft-019-response-amplifier.md)
- [FT-020: Self/Non-Self](.spec./ft-020-self-nonself.md)
- [FT-021: Adaptive Mutation](.spec./ft-021-adaptive-mutation.md)
- [FT-022: Network Regulation](.spec./ft-022-network-regulation.md)
- [FT-023: Optimal Transport](.spec./ft-023-optimal-transport.md)
- [FT-024: Latency p99](.spec./ft-024-latency-p99.md)

---

## Development

```bash
# Run all tests
cargo test --workspace

# Run clippy
cargo clippy -- -D warnings

# Check formatting
cargo fmt --all -- --check

# Run benchmarks
cargo bench
```

---

## License

This project is licensed under the [MIT License](LICENSE).
