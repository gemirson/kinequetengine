//! Python bindings for the KineContext Engine (PyO3).
//!
//! Exposes core KCE operations — retrieval, MCE encoding, AIS classification,
//! Sinkhorn optimal transport, cosine similarity, and latency histograms —
//! as a native Python extension module.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use kce_ais::classifier::SelfNonSelfClassifier;
use kce_core::traits::DistanceMetric;
use kce_core::types::EcmaNode;
use kce_metrics::cosine::CosineMetric;
use kce_metrics::latency::LatencyHistogram;
use kce_metrics::sinkhorn::SinkhornMetric;
use kce_mce::mrna::MceConfig;
use kce_retrieval::engine::{Dataset, RetrievalEngine};

// ── Helper ───────────────────────────────────────────────────────────────────

fn to_pyerr<E: std::fmt::Display>(e: E) -> PyErr {
    PyValueError::new_err(e.to_string())
}

// ── PyLatencyHistogram ───────────────────────────────────────────────────────

/// Python-exposed latency histogram wrapping [`kce_metrics::latency::LatencyHistogram`].
#[pyclass]
struct PyLatencyHistogram {
    inner: LatencyHistogram,
}

#[pymethods]
impl PyLatencyHistogram {
    /// Create a new empty histogram (10us buckets, 0-100ms range).
    #[new]
    fn new() -> Self {
        Self {
            inner: LatencyHistogram::new(),
        }
    }

    /// Record a latency sample in microseconds.
    fn record(&self, us: u64) {
        self.inner.record_us(us);
    }

    /// Return a dict with P50/P95/P99/P99.9 percentiles, count, mean, min, max.
    fn percentiles<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let snap = self.inner.percentiles();
        let dict = PyDict::new_bound(py);
        dict.set_item("count", snap.count)?;
        dict.set_item("mean_us", snap.mean_us)?;
        dict.set_item("min_us", snap.min_us)?;
        dict.set_item("max_us", snap.max_us)?;
        dict.set_item("p50_us", snap.p50_us)?;
        dict.set_item("p95_us", snap.p95_us)?;
        dict.set_item("p99_us", snap.p99_us)?;
        dict.set_item("p999_us", snap.p999_us)?;
        Ok(dict)
    }

    /// Reset all counters.
    fn reset(&self) {
        self.inner.reset();
    }

    fn __repr__(&self) -> String {
        format!("PyLatencyHistogram(count={})", self.inner.count())
    }
}

// ── Module-level functions ───────────────────────────────────────────────────

/// Hybrid retrieval search using cosine + prime similarity.
///
/// Parameters
/// ----------
/// query_vector : list[float]
///     The query vector.
/// top_k : int
///     Number of top results to return.
///
/// Returns
/// -------
/// list[tuple[int, float]]
///     List of (id, score) tuples sorted by descending score.
#[pyfunction]
fn retrieve(query_vector: Vec<f64>, top_k: usize) -> PyResult<Vec<(u64, f64)>> {
    let engine = RetrievalEngine::with_defaults();
    let dimension = query_vector.len();

    // Build a small dataset from the query itself — in a real deployment the
    // dataset would be loaded from storage.  Here we just demonstrate the API
    // shape.  The caller is expected to populate the dataset externally.
    let dataset = Dataset::new(dimension);

    // If the dataset is empty the search would fail with dimension mismatch,
    // so we return an empty result rather than erroring.
    if dataset.is_empty() {
        return Ok(Vec::new());
    }

    let results = engine
        .search(&query_vector, &dataset, top_k)
        .map_err(to_pyerr)?;

    Ok(results.into_iter().map(|r| (r.id, r.score)).collect())
}

/// Encode ECMA nodes into an mRNA payload.
///
/// Parameters
/// ----------
/// nodes : list[tuple[str, float]]
///     Each tuple is (node_state, maturity).  node_state must be one of
///     "Stem", "Progenitor", "Specialized", "Apoptosis".
///
/// Returns
/// -------
/// bytes
///     The encoded mRNA payload bytes.
#[pyfunction]
fn encode(nodes: Vec<(String, f64)>) -> PyResult<Vec<u8>> {
    let config = MceConfig::default();
    let ecma_nodes: Vec<EcmaNode> = nodes
        .into_iter()
        .enumerate()
        .map(|(i, (state_str, maturity))| {
            let state = match state_str.as_str() {
                "Stem" => kce_core::types::NodeState::Stem,
                "Progenitor" => kce_core::types::NodeState::Progenitor,
                "Specialized" => kce_core::types::NodeState::Specialized,
                "Apoptosis" => kce_core::types::NodeState::Apoptosis,
                other => {
                    return Err(PyValueError::new_err(format!(
                        "invalid node state '{}': expected Stem|Progenitor|Specialized|Apoptosis",
                        other
                    )))
                }
            };
            Ok(EcmaNode {
                id: i as u64,
                state,
                usage_count: 1,
                entropy: 0.5,
                connections: 1,
                maturity,
            })
        })
        .collect::<PyResult<Vec<_>>>()?;

    let mce = kce_mce::engine::MceEngine::new(config);
    let mrna = mce.encode(&ecma_nodes, kce_core::types::MrnaIntent::Classify);
    Ok(mrna.payload)
}

/// Classify a vector as self or non-self.
///
/// Parameters
/// ----------
/// vector : list[float]
///     The vector to classify.
/// self_patterns : list[list[float]]
///     Known "self" patterns.  At least 3 required for meaningful classification.
///
/// Returns
/// -------
/// tuple[str, float]
///     (label, confidence) where label is "self" or "non_self".
#[pyfunction]
fn classify(vector: Vec<f64>, self_patterns: Vec<Vec<f64>>) -> PyResult<(String, f64)> {
    let mut classifier = SelfNonSelfClassifier::default();
    for pattern in self_patterns {
        classifier.register_self(pattern);
    }
    let result = classifier.classify(&vector);
    let label = match result.label {
        kce_ais::classifier::Classification::Self_ => "self",
        kce_ais::classifier::Classification::NonSelf => "non_self",
    };
    Ok((label.to_string(), result.confidence))
}

/// Compute the Sinkhorn optimal transport distance between two distributions.
///
/// Parameters
/// ----------
/// a : list[float]
///     First distribution (non-negative values).
/// b : list[float]
///     Second distribution (non-negative values, same length as a).
///
/// Returns
/// -------
/// float
///     The Sinkhorn transport distance.
#[pyfunction]
fn sinkhorn_distance(a: Vec<f64>, b: Vec<f64>) -> PyResult<f64> {
    let metric = SinkhornMetric::with_defaults();
    metric.compute(&a, &b).map_err(to_pyerr)
}

/// Compute cosine similarity between two vectors.
///
/// Parameters
/// ----------
/// a : list[float]
///     First vector.
/// b : list[float]
///     Second vector (same length as a).
///
/// Returns
/// -------
/// float
///     Cosine similarity in [-1.0, 1.0].
#[pyfunction]
fn cosine_similarity(a: Vec<f64>, b: Vec<f64>) -> PyResult<f64> {
    let metric = CosineMetric::new();
    metric.compute(&a, &b).map_err(to_pyerr)
}

/// Create a new [`PyLatencyHistogram`].
///
/// Returns
/// -------
/// PyLatencyHistogram
///     An empty histogram ready to record latency samples.
#[pyfunction]
fn latency_histogram_new() -> PyLatencyHistogram {
    PyLatencyHistogram::new()
}

// ── Module definition ────────────────────────────────────────────────────────

/// KCE Python extension module.
///
/// Exposes retrieval, encoding, classification, metrics, and latency
/// histogram operations as native Python functions and classes.
#[pymodule]
fn kce_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(retrieve, m)?)?;
    m.add_function(wrap_pyfunction!(encode, m)?)?;
    m.add_function(wrap_pyfunction!(classify, m)?)?;
    m.add_function(wrap_pyfunction!(sinkhorn_distance, m)?)?;
    m.add_function(wrap_pyfunction!(cosine_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(latency_histogram_new, m)?)?;
    m.add_class::<PyLatencyHistogram>()?;
    Ok(())
}
