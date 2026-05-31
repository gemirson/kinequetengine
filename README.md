# KineQuetEngine (KCE)

**Bio-Inspired Context Engine for AI/ML in Rust**

[![License: CC BY-NC 4.0](https://img.shields.io/badge/License-CC_BY--NC_4.0-lightgrey.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-258%20passing-brightgreen)]()
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)]()
[![CI](https://github.com/kinecontext/kce/actions/workflows/ci.yml/badge.svg)]()

KCE is a modular cognitive engine that processes semantic context through bio-inspired algorithms. It combines vector search, semantic graph expansion, entity lifecycle management, and mRNA-style executable context generation -- enhanced by Ant Colony Optimization (ACO) for emergent routing and an Artificial Immune System (AIS) for anomaly detection and self-regulation.

---

## Why KCE is Different and Innovative?

Unlike traditional vector search solutions (such as Pinecone or Milvus) that act as passive data stores, the **KineQuetEngine (KCE)** is an **Active Cognitive Data Engine**. It doesn't just store; it organizes, protects, and executes logic over context autonomously.

### 🧬 Bio-Inspired Innovation
KCE utilizes algorithms extracted from biology to solve critical AI infrastructure problems:
- **Ant Colony Optimization (ACO-HNSW):** Instead of static searches, KCE creates "cognitive highways" using digital pheromones. Frequent queries reinforce paths, allowing sub-millisecond latencies on critical paths (P99 of 3-10ms).
- **Artificial Immune System (AIS):** Intrinsic security at the data layer. KCE has an immune system that detects and blocks semantic anomalies and prompt injection attempts before they even reach the LLM.
- **ECMA Engine:** Inspired by stem cells and apoptosis, the database manages the data lifecycle autonomously. Irrelevant information decays and is automatically removed, keeping the system memory clean and up-to-date.

### 🚀 From Context to Execution (mRNA/MCE)
While competitors return only raw text, KCE introduces **mRNA Cognitive Execution**. It compiles the retrieved context into an executable binary payload. This transforms vector search into active decision-making, drastically reducing token consumption and execution latency for AI agents by over 50%.

### 🤖 Agent-Native (MCP)
KCE speaks the native language of modern agents. With native support for the **Model Context Protocol (MCP)**, it integrates instantly with tools like Cursor and Claude Desktop, allowing AI agents to query and configure the engine autonomously and dynamically.

### 🛡️ Zero-Ops Homeostasis
Inspired by Jerne's Immune Network Theory, the system self-regulates. It monitors performance metrics and adjusts its own parameters (evaporation rates, mutation rates, and ant counts) in real-time, eliminating the need for DBAs or complex manual configurations.

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

## Features (FT-001 through FT-036)

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
| FT-035 | Gödel Encoding | Prime factorization sequence signature & loops detection |

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
| FT-034 | Differential Geometry | Manifold geodesic search and Ollivier-Ricci curvature |

### Distributed Swarm

| ID | Feature | Description |
|---|---|---|
| FT-036 | Game Theory | MCTS and sliding window resource allocation game |

---

## Specifications

Full feature specifications are in the [`.spec./`](.spec./) directory:

- [FT-001: Retrieval Engine](.spec./ft-001-retrieval-engine.md)
- [FT-002: Graph Engine](.spec./ft-002-graph-engine.md)
- [FT-003: ECMA Engine](.spec./ft-003-ecma-engine.md)
- [FT-004: MCE Engine](.spec./ft-004-mce-engine.md)
- [FT-005: KineSQL Storage](.spec./ft-005-kinesql-storage.md)
- [FT-006: API Gateway](.spec./ft-006-api-gateway.md)
- [FT-007: Observability](.spec./ft-007-observability.md)
- [FT-008: Security](.spec./ft-008-security.md)
- [FT-009: Resilience](.spec./ft-009-resilience.md)
- [FT-010: Concurrency](.spec./ft-010-concurrency.md)
- [FT-011: Robust Pipeline](.spec./ft-011-robust-pipeline.md)
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
- [FT-034: Differential Geometry](.spec./ft-034-differential-geometry.md)
- [FT-035: Gödel Encoding](.spec./ft-035-godel-encoding.md)
- [FT-036: Game Theory](.spec./ft-036-game-theory.md)

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

This project is licensed under the [Creative Commons Attribution-NonCommercial 4.0 International (CC BY-NC 4.0)](LICENSE) license. Commercial use is strictly prohibited without prior written authorization.
