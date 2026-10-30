# FT-025 — MCP Server (Model Context Protocol)

**Module:** Interface | **Version:** v6.0 | **Priority:** P1 — Important  
**Artifact ID:** FT-025-MCP-SERVER | **Update:** 2026-05-30

---

## 1. Context and Objective

**MCP Server** acts as KCE's direct interface to artificial intelligence agents (such as Cursor, Claude Desktop, and others). It implements the JSON-RPC 2.0 protocol through standard input and output bidirectional communication (`stdin`/`stdout`), exposing the essential capabilities of KCE as automated tools. This allows LLMs to perform vector searches, AI anomaly calibration, and optimal transport mathematical calculations directly on demand.

---

## 2. Acceptance Criteria (AC)

### General
- [ ] AC-001: Compiles without warnings in `--release`
- [ ] AC-002: Zero redundant allocations in the I/O event loop
- [ ] AC-003: Complete public documentation coverage (/// doc comments)

### Specifics
- [ ] AC-010: Initial handshake via `initialize` method call returning the protocol version (`2024-11-05`) and server capabilities.
- [ ] AC-011: Dynamic listing of tools available via `tools/list` method call with valid JSON schemas.
- [ ] AC-012: Support for running tools with the `tools/call` method.
- [ ] AC-013: Tool `kce_retrieve`: accepts query vector and dataset, performing KCE hybrid search.
- [ ] AC-014: Tool `kce_classify`: accepts vector and known self-recognition patterns, running the immunological classifier.
- [ ] AC-015: Tool `kce_sinkhorn`: computes the optimal Sinkhorn transport distance between two distributions.
- [ ] AC-016: Tool `kce_regulate`: performs network assessment and homeostasis with system performance metrics.
- [ ] AC-017: Responses strictly comply with the JSON-RPC 2.0 specification (fields `jsonrpc`, `id`, `result`, and `error`).
- [ ] AC-018: Safe string manipulation and JSON parser error handling (avoiding panics).

---

## 3. Definition of Done (DoD)

- [ ] Validated and robust JSON-RPC 2.0 parser
- [ ] 4 core tools registered and tested via mock I/O
- [ ] Structured error handling for invalid calls and internal errors
- [ ] Payload serialization unit tests
- [ ] Functional tests with simulation of command line interactions
- [ ] No `unwrap()` calls on critical execution paths
- [ ] Full integration with KCE logs and metrics module

---

## 4. Usage Examples

### Call to Run Search Tool (`kce_retrieve`)

**Input (stdin):**
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

**Output (stdout):**
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

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Entry | Expected Output |
|----|------|---------|----------------|
| UT-001 | Initial handshake | `initialize` method with numeric ID | JSON-RPC response with server information |
| UT-002 | Tool listing | `tools/list` method | JSON listing the 4 registered tools |
| UT-003 | Invalid method | `tools/non_existent` method | Error code `-32601` (Method not found) |
| UT-004 | parser error | Corrupt text payload or invalid JSON | Error code `-32700` (Parse error) |

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | Main loop execution | The server runs continuously reading lines from stdin until the end of the stream |
| FT-002 | Sinkhorn similarity test | Call of `kce_sinkhorn` returns successfully computed distance |
| FT-003 | Executing Homeostasis | Calling `kce_regulate` generates dynamic adjustments based on provided metrics |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | MCP → Kce-Retrieval | The input vector is mapped identically and search is performed |
| IT-002 | MCP → Kce-Ais | Self/non-self classification returns the correct label with confidence level |

---

## 6. CARE Format

**Context:** Direct communication channel with intelligent agents to make KCE actionable by modern LLM systems, extending the cognitive engine to autonomous flows.

**Assumptions:** Inputs and outputs exclusively use UTF-8 and JSON over standard file descriptors (`stdin`/`stdout`). The server does not attempt to manage concurrent sessions on the same physical channel.

**Requirements:** R-001: JSON-RPC 2.0 compliance | R-002: Model Context Protocol (MCP) compliance | R-003: display of retrieval, classify, sinkhorn and regulation as tools.

**Evidence:** Automated tests on `crates/kce-mcp` and integration of tools proven by console tests.

---

## 7. Non-Functional Criteria

| Appearance | Metric | Target |
|---------|---------|------|
| Dispatcher Latency | Message parsing and routing time | < 1ms |
| Memory footprint | Basic RAM consumption at idle | < 15MB |
| Error handling | Security under malformed data | Complete resilience (no panic) |

---

## 8. Quality and Metrics

**Success:** Correct protocol response to all messages with strict compliance (0 parsing errors in handshake).

**Failure (BLOCKING):** Abrupt connection drop (panic) under malformed JSON or stdin interruption.

---

## 9. Compatibility and Dependencies

| Crate | Version | Purpose |
|-------|--------|-----------|
| `serde_json` | ^1.0 | Serialization and deserialization |
| `kce-retrieval` | Internal | Executing the hybrid search |
| `kce-ais` | Internal | Classification and regulation |

---

## 10. Traceability

| Type | ID | Description |
|------|----|-----------|
| Spec | FT-025-MCP-SERVER | This specification |
| Code | `crates/kce-mcp/src/main.rs` | I/O Server Implementation |

---

## 11. Roadmap

### MVP (Current Phase)
Basic JSON-RPC 2.0 interface with support for the 3 standard methods and exposure of retrieval and classify.

### Iteration 1
Full integration with KCE metrics (/metrics) and support for real-time execution of homeostasis regulation.

### Iteration 2
Support additional transport channels (gRPC-Web / SSE) for Web connections.
