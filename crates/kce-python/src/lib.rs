//! Native Python bindings for KineContext Engine (FT-026).
//!
//! Utiliza PyO3 para expor funcionalidades de alta performance do Rust para Python.

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use std::sync::Arc;

use kce_core::traits::DistanceMetric;
use kce_metrics::cosine::CosineMetric;
use kce_metrics::latency::LatencyHistogram;
use kce_metrics::sinkhorn::{SinkhornConfig, SinkhornMetric};
use kce_ais::classifier::{AisClassifier, Classification};
use kce_retrieval::engine::{Dataset, RetrievalConfig, RetrievalEngine};

/// Mapeia erros do KCE para exceções Python.
fn map_kce_err(e: impl ToString) -> PyErr {
    PyRuntimeError::new_err(e.to_string())
}

/// Exposição da classe LatencyHistogram para Python.
#[pyclass]
pub struct PyLatencyHistogram {
    inner: LatencyHistogram,
}

#[pymethods]
impl PyLatencyHistogram {
    #[new]
    fn new() -> Self {
        Self {
            inner: LatencyHistogram::new(),
        }
    }

    /// Grava uma amostra de latência em microssegundos.
    fn record(&self, us: u64) {
        self.inner.record_us(us);
    }

    /// Retorna os percentis atuais como um dicionário.
    fn percentiles(&self) -> PyResult<PyObject> {
        let snap = self.inner.percentiles();
        Python::with_gil(|py| {
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("count", snap.count)?;
            dict.set_item("mean_us", snap.mean_us)?;
            dict.set_item("min_us", snap.min_us)?;
            dict.set_item("max_us", snap.max_us)?;
            dict.set_item("p50_us", snap.p50_us)?;
            dict.set_item("p95_us", snap.p95_us)?;
            dict.set_item("p99_us", snap.p99_us)?;
            dict.set_item("p999_us", snap.p999_us)?;
            Ok(dict.to_object(py))
        })
    }

    /// Reseta o histograma.
    fn reset(&self) {
        self.inner.reset();
    }
}

/// Calcula a distância Sinkhorn entre duas distribuições.
#[pyfunction]
fn sinkhorn_distance(a: Vec<f64>, b: Vec<f64>) -> PyResult<f64> {
    let metric = SinkhornMetric::with_defaults();
    metric.compute(&a, &b).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Calcula a similaridade de cosseno entre dois vetores.
#[pyfunction]
fn cosine_similarity(a: Vec<f64>, b: Vec<f64>) -> PyResult<f64> {
    let metric = CosineMetric::new();
    metric.compute(&a, &b).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Classificação imunológica (AIS).
#[pyfunction]
fn classify(vector: Vec<f64>, self_patterns: Vec<Vec<f64>>) -> PyResult<(String, f64)> {
    let mut classifier = AisClassifier::new(0.1); // threshold padrão
    for p in self_patterns {
        classifier.add_self_pattern(p);
    }
    
    let result = classifier.classify(&vector);
    let label = match result.label {
        Classification::Self_ => "self",
        Classification::NonSelf => "non-self",
    };
    
    Ok((label.to_string(), result.confidence))
}

/// Executa busca híbrida KCE.
#[pyfunction]
fn retrieve(query_vector: Vec<f64>, dataset_vectors: Vec<(u64, Vec<f64>)>, top_k: usize) -> PyResult<Vec<(u64, f64)>> {
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);
    let mut ds = Dataset::new(query_vector.len());
    
    for (id, vec) in dataset_vectors {
        ds.push(id, vec).map_err(map_kce_err)?;
    }
    
    let results = engine.search(&query_vector, &ds, top_k).map_err(map_kce_err)?;
    Ok(results.into_iter().map(|r| (r.id, r.score)).collect())
}

/// Módulo kce_python exposto para o interpretador.
#[pymodule]
fn kce_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyLatencyHistogram>()?;
    m.add_function(wrap_pyfunction!(sinkhorn_distance, m)?)?;
    m.add_function(wrap_pyfunction!(cosine_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(classify, m)?)?;
    m.add_function(wrap_pyfunction!(retrieve, m)?)?;
    Ok(())
}
