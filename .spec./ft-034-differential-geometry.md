# FT-034 — Differential Geometry (Manifold Learning & Curvature)

**Module:** Mathematics | **Version:** v6.0 | **Priority:** P1 — High  
**Artifact ID:** FT-034-DIFFGEO | **Update:** 2026-05-31

---

## 1. Context and Objective

High-dimensional embedding spaces often exhibit topological distortion when using flat distance metrics (such as Cosine or Euclidean) due to the manifold hypothesis: semantic data is concentrated on a low-dimensional non-linear manifold. 

This module implements Differential Geometry algorithms on the semantic graph. It estimates local Riemannian curvature (Ollivier-Ricci curvature) to identify semantic hubs/boundaries and computes geodesic distances (shortest paths along the manifold) instead of straight-line distances. Additionally, it implements parallel transport to move context vectors along graph pathways without losing their relative orientation and semantic meaning.

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: Compute discrete Ollivier-Ricci curvature on graph edges and nodes to locate dense semantic clusters (positive curvature) and bridges (negative curvature).
- [ ] AC-011: Calculate geodesic distance along the semantic manifold using graph-path length weighted by local curvature and edge weights.
- [ ] AC-012: Implement parallel transport of context embeddings along a graph path to carry contextual vector meaning without distortion.
- [ ] AC-013: Fallback gracefully to cosine similarity if the graph connectivity is too low or disconnected.
- [ ] AC-014: Manifold-aware search achieves ≥ 5% improvement in semantic retrieval Recall@10 compared to flat cosine search.

---

## 3. Definition of Done (DoD)

- [ ] Implement Ollivier-Ricci curvature algorithm for graph edges and nodes.
- [ ] Geodesic pathfinder integrated with HNSW graph indexing.
- [ ] Parallel vector transport matrix computation implemented in Rust.
- [ ] Zero-unsafe Rust implementation.
- [ ] Unit tests covering curvature and parallel transport calculations.
- [ ] Performance benchmarks showing geodesic query latency under 10ms.

---

## 4. Usage Examples

### Curvature Calculation Request
```json
{
  "node_id": "concept_quantum_computing",
  "neighbors_depth": 1
}
```

### Curvature Calculation Response
```json
{
  "node_id": "concept_quantum_computing",
  "ricci_curvature": 0.42,
  "topology_class": "semantic_hub",
  "connected_edges": [
    { "target": "concept_qubit", "edge_curvature": 0.55 },
    { "target": "concept_cryptography", "edge_curvature": -0.12 }
  ]
}
```

### Geodesic Search
```json
{
  "query_vector": [0.12, -0.45, 0.88, 0.03],
  "top_k": 5,
  "use_manifold": true
}
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Expected Output |
|----|------|----------------|
| UT-001 | Ollivier-Ricci curvature on a ring graph | Curvature close to 0 |
| UT-002 | Ollivier-Ricci curvature on a clique (fully connected) | Positive curvature (> 0.5) |
| UT-003 | Parallel transport of orthogonal vector | Orthonormality preserved along path |
| UT-004 | Geodesic distance on empty graph | Fallback to Cosine |

### 5.2 Functional Tests

| ID | Scenario | Result |
|----|---------|-----------|
| FT-001 | Geodesic search along curved manifold | Returns non-linear semantic neighbors, Recall@10 improves |
| FT-002 | Parallel transport of intent vector | Context is shifted correctly along semantic bridge |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Graph Engine → DiffGeo | Dynamic curvature update when adding graph edges |
| IT-002 | DiffGeo → MCE Engine | Transported vector translated into correct mRNA instructions |

---

## 6. CARE Format

**Context:** Embedding spaces are non-linear; flat metrics fail to capture true manifold geometry. Curvature and geodesics provide topological awareness.

**Assumptions:** Semantic graph is dense enough to approximate a manifold; local neighbor computations can be done within low-latency constraints.

**Requirements:** R-001: Ollivier-Ricci curvature | R-002: Geodesic pathfinder | R-003: Parallel vector transport | R-004: SIMD accelerated distance fallback.

**Evidence:** `src/math/diffgeo.rs` implementation and comparative Recall/latency benchmarks.

---

## 7–11. (Summary)

**Non-functional:** Curvature calculation < 2ms | Geodesic search overhead < 5ms | Memory usage < 50MB  
**Quality:** 100% thread-safe | 0 unsafe blocks | Vector orthoconformity deviation < 1e-6  
**Deps:** `ndarray` ^0.15, `petgraph` ^0.6, `parking_lot` ^0.12  
**Traceability:** FT-034-DIFFGEO | `src/math/diffgeo.rs`, `src/graph/manifold.rs`  

**Roadmap:** MVP: Geodesic Dijkstra + local Ricci curvature | Iter1: Parallel transport + HNSW integration | Iter2: Dynamic manifold self-regulation
