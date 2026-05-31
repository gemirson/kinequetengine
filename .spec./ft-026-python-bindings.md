# FT-026 — Native Python Bindings (PyO3)

**Module:** Interface | **Version:** v6.0 | **Priority:** P1 — Important  
**Artifact ID:** FT-026-PYTHON-BINDINGS | **Update:** 2026-05-30

---

## 1. Context and Objective

**Native Python Bindings** exposes the high-performance features of KCE (developed in Rust) directly to the Python ecosystem. Using the PyO3 library, this module generates a compiled dynamic package (`.so`/`.pyd`) that can be imported into traditional Python scripts (`import kce_python`). This simplifies the adoption of the cognitive engine in data science and machine learning engineering projects without compromising the computational performance of the Rust backend.

---

## 2. Acceptance Criteria (AC)

### General
- [ ] AC-001: Clean dynamic compilation on Linux systems (generating `.so` file)
- [ ] AC-002: Efficient management of Python's Global Interpreter Lock (GIL) in heavy operations
- [ ] AC-003: Clear mapping of Rust errors to proper Python exceptions (`ValueError`, `RuntimeError`)

### Specifics
- [ ] AC-010: Exposure of the `PyLatencyHistogram` class with methods `record(us)`, `percentiles()` and `reset()`.
- [ ] AC-011: Exposure of the `retrieve(query_vector, top_k)` function returning a list of `(id, score)` tuples.
- [ ] AC-012: Exposure of the `encode(nodes)` function receiving a list of `(state, maturity)` tuples and returning the mRNA payload compiled into bytes.
- [ ] AC-013: Exposure of the `classify(vector, self_patterns)` function returning the classification label and confidence score.
- [ ] AC-014: Exposure of the `sinkhorn_distance(a, b)` function returning the value of the Sinkhorn optimal transport distance.
- [ ] AC-015: Exposure of the `cosine_similarity(a, b)` function returning cosine similarity quickly.
- [ ] AC-016: Preservation of memory integrity in complex data conversions between CPython and the Rust runtime.

---

## 3. Definition of Done (DoD)

- [ ] Extension compiled via maturin/cargo-c successfully
- [ ] 6 native functions exposed and tested via Python scripts
- [ ] Functional and documented `PyLatencyHistogram` telemetry class
- [ ] Unit tests with the `pytest` framework passing in CI
- [ ] Correct mapping and propagation of exceptions without catastrophic breaks (Core Dumps)
- [ ] API documentation in the Docstring standard integrated

---

## 4. Usage Examples

### Call in Python Script

```python
import kce_python

# 1. Distância Sinkhorn entre duas distribuições
dist = kce_python.sinkhorn_distance([0.2, 0.8], [0.5, 0.5])
print(f"Distância Sinkhorn: {dist}")

# 2. Medir similaridade cosseno
score = kce_python.cosine_similarity([1.0, 0.0], [1.0, 0.0])
assert score == 1.0

# 3. Classificação Imunológica (AIS)
label, confidence = kce_python.classify(
    vector=[0.1, 0.2, 0.3],
    self_patterns=[
        [0.1, 0.2, 0.3],
        [0.11, 0.19, 0.29],
        [0.09, 0.21, 0.31]
    ]
)
print(f"Classificação: {label} (Confiança: {confidence})")

# 4. Histograma de Latência
hist = kce_python.latency_histogram_new()
hist.record(1200) # 1.2ms
hist.record(850)  # 0.85ms
metrics = hist.percentiles()
print(f"P95 latency: {metrics['p95_us']} us")
```

---

## 5. Test Plans

### 5.1 Unit Tests (pytest)

| ID | Case | Entry | Expected Output |
|----|------|---------|----------------|
| UT-001 | Cosine Similarity | Two identical vectors | Returns `1.0` |
| UT-002 | Dimension error in cosine | Vectors with different sizes | Release `ValueError` in Python |
| UT-003 | AIS Classifier | Vector identical to a self pattern | Label `"self"` with high confidence |
| UT-004 | Percentile histogram | Sample recording | Dictionary containing count, mean, p50, p95 and p99 |

### 5.2 Functional Tests

| ID | Scenario | Expected Result |
|----|---------|---------------------|
| FT-001 | Integration with numpy/scipy | Vectors extracted from numpy arrays are converted and processed correctly |
| FT-002 | Garbage Collection Validation | Repeated allocation and deallocation of histogram instances do not generate memory leaks |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Python Binding → MceEngine | The `encode` function translates state strings ("Stem", "Apoptosis") to Rust enums, returning valid bytes |

---

## 6. CARE Format

**Context:** Provision of the KCE library for data scientists and machine learning engineers who use Python, combining the ease of the language with computational performance in Rust.

**Assumptions:** Target Python is v3.8 or higher. The passed vectors are convertible to primitive types (`f64`). The PyO3 compiler manages references securely under the GIL.

**Requirements:** R-001: Native compiled packaging | R-002: Robust translation of enums and structured types | R-003: Mathematical coherence in outputs compared to the Rust core.

**Evidence:** Test suite `pytest` located at `crates/kce-python/tests/` run in local pipeline.

---

## 7. Non-Functional Criteria

| Appearance | Metric | Target |
|---------|---------|------|
| Conversion Overhead | Extra cost of passing data Python -> Rust | < 2% of the cost of native operation |
| Type Compatibility | Python/Numpy numeric type coercion | Support for 64-bit integers and floats |

---

## 8. Quality and Metrics

**Success:** Successfully passed all unit tests via `pytest`, with no occurrences of catastrophic segmentation faults (`segmentation faults`).

**Fault (BLOCKING):** Systematic memory leaks verified in the interpreter's GC loop or stack overflow due to cyclic conversion.

---

## 9. Compatibility and Dependencies

| Crate/Tool | Version | Purpose |
|--------------|--------|-----------|
| `pyo3` | ^0.21 | Rust-Python Interface |
| `maturin` | ^1.0 | Extension Builder and Packer |
| Python | >= 3.8 | Target runtime interpreter |

---

## 10. Traceability

| Type | ID | Description |
|------|----|-----------|
| Spec | FT-026-PYTHON-BINDINGS | This specification |
| Code | `crates/kce-python/src/lib.rs` | PyO3 dynamic bindings |

---

## 11. Roadmap

### MVP (Current Phase)
Direct export of mathematical metrics, classification of anomalies by similarity vector and export of the latency histogram.

### Iteration 1
Support for loading entire KineSQL datasets directly from file paths in Python.

### Iteration 2
Support for multi-thread parallelism with explicit release of the GIL (`py.allow_threads`).
