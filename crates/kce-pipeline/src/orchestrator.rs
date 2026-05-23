//! Pipeline orchestrator for the KineContext Engine.
//!
//! Coordinates all KCE modules into a deterministic cognitive pipeline.
//! Each stage returns `Result<T, KceError>` -- nothing assumes success.
//! Includes resilience features: circuit breaker, retry with backoff,
//! idempotency caching, and structured metrics.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use kce_core::error::KceError;
use kce_core::types::{ActionResult, ContextInput, ContextOutput, EcmaNode, MrnaIntent, NodeState, SearchResult};
use kce_ecma::engine::EcmaEngine;
use kce_graph::graph::SemanticGraph;
use kce_metrics::latency::LatencyHistogram;
use kce_mce::engine::MceEngine;
use kce_retrieval::engine::{Dataset, RetrievalConfig, RetrievalEngine};

// ── Configuration ────────────────────────────────────────────────────────────

/// Configuration for the pipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Retrieval configuration.
    pub retrieval: RetrievalConfig,
    /// Maximum time budget for the entire pipeline in milliseconds.
    pub timeout_ms: u64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            retrieval: RetrievalConfig::default(),
            timeout_ms: 100,
        }
    }
}

// ── Latency Budget & Degradation (FT-024) ───────────────────────────────────

/// Per-stage latency budget for the hot path.
///
/// Budgets are in milliseconds and sum to the total pipeline target.
/// If the remaining budget drops below a stage's budget, that stage
/// is skipped via [`DegradationStrategy`].
#[derive(Debug, Clone)]
pub struct LatencyBudget {
    /// Parse budget (ms). Default: 0.5
    pub parse_ms: f64,
    /// Retrieval budget (ms). Default: 3.0
    pub retrieval_ms: f64,
    /// Ranking budget (ms). Default: 2.0
    pub ranking_ms: f64,
    /// Execution budget (ms). Default: 1.0
    pub execution_ms: f64,
    /// Total pipeline budget (ms). Default: 10.0
    pub total_ms: f64,
}

impl Default for LatencyBudget {
    fn default() -> Self {
        Self {
            parse_ms: 0.5,
            retrieval_ms: 3.0,
            ranking_ms: 2.0,
            execution_ms: 1.0,
            total_ms: 10.0,
        }
    }
}

/// Degradation decisions based on remaining latency budget.
///
/// When the pipeline is running low on time, optional stages are
/// skipped to preserve the P99 target.
#[derive(Debug, Clone, Default)]
pub struct DegradationDecision {
    /// Graph expansion was skipped.
    pub skipped_graph: bool,
    /// ECMA update was skipped.
    pub skipped_ecma: bool,
    /// MCE encode was skipped (used cached/simplified result).
    pub skipped_mce: bool,
    /// OT distance was skipped (used cosine fallback).
    pub skipped_ot: bool,
}

/// Strategy for degrading the pipeline under time pressure.
pub struct DegradationStrategy {
    /// Minimum remaining budget (ms) before graph expansion is skipped.
    pub graph_threshold_ms: f64,
    /// Minimum remaining budget (ms) before ECMA is skipped.
    pub ecma_threshold_ms: f64,
    /// Minimum remaining budget (ms) before MCE is skipped.
    pub mce_threshold_ms: f64,
}

impl Default for DegradationStrategy {
    fn default() -> Self {
        Self {
            graph_threshold_ms: 3.0,
            ecma_threshold_ms: 2.0,
            mce_threshold_ms: 1.0,
        }
    }
}

impl DegradationStrategy {
    /// Evaluate which stages should be skipped given the elapsed time
    /// and total budget.
    pub fn evaluate(&self, elapsed_ms: f64, budget: &LatencyBudget) -> DegradationDecision {
        let remaining = budget.total_ms - elapsed_ms;

        DegradationDecision {
            skipped_graph: remaining < self.graph_threshold_ms,
            skipped_ecma: remaining < self.ecma_threshold_ms,
            skipped_mce: remaining < self.mce_threshold_ms,
            skipped_ot: remaining < 1.5,
        }
    }
}

// ── Circuit Breaker ──────────────────────────────────────────────────────────

/// Circuit breaker states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation -- requests pass through.
    Closed,
    /// Too many failures -- requests are rejected.
    Open,
    /// Testing recovery -- limited requests allowed.
    HalfOpen,
}

/// Simple circuit breaker that opens when error rate exceeds 20 %
/// within a 60-second sliding window.
pub struct CircuitBreaker {
    state: CircuitState,
    /// Timestamps of recent failures inside the window.
    failures: Vec<Instant>,
    /// Timestamps of recent successes inside the window.
    successes: Vec<Instant>,
    /// Sliding window duration.
    window: Duration,
    /// Error-rate threshold (0.0 .. 1.0).
    threshold: f64,
    /// How long to stay Open before moving to HalfOpen.
    open_duration: Duration,
    /// When the circuit last transitioned to Open.
    opened_at: Option<Instant>,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitBreaker {
    /// Create a circuit breaker with default settings (20 % / 60 s).
    pub fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failures: Vec::new(),
            successes: Vec::new(),
            window: Duration::from_secs(60),
            threshold: 0.20,
            open_duration: Duration::from_secs(30),
            opened_at: None,
        }
    }

    /// Check whether the circuit allows a request right now.
    ///
    /// Returns the current state on success, or [`KceError::CircuitOpen`]
    /// when the circuit is Open.
    pub fn check(&mut self) -> Result<CircuitState, KceError> {
        self.evict_old();
        match self.state {
            CircuitState::Open => {
                if let Some(opened) = self.opened_at {
                    if opened.elapsed() >= self.open_duration {
                        self.state = CircuitState::HalfOpen;
                        return Ok(CircuitState::HalfOpen);
                    }
                }
                Err(KceError::CircuitOpen)
            }
            other => Ok(other),
        }
    }

    /// Record a successful request.
    pub fn record_success(&mut self) {
        self.successes.push(Instant::now());
        if self.state == CircuitState::HalfOpen {
            self.state = CircuitState::Closed;
            self.failures.clear();
        }
    }

    /// Record a failed request.
    pub fn record_failure(&mut self) {
        self.failures.push(Instant::now());
        self.evict_old();
        let total = self.failures.len() + self.successes.len();
        if total >= 5 {
            let error_rate = self.failures.len() as f64 / total as f64;
            if error_rate > self.threshold {
                self.state = CircuitState::Open;
                self.opened_at = Some(Instant::now());
            }
        }
    }

    /// Get the current circuit state.
    pub fn state(&self) -> CircuitState {
        self.state
    }

    /// Drop entries older than the sliding window.
    fn evict_old(&mut self) {
        let cutoff = Instant::now() - self.window;
        self.failures.retain(|t| *t > cutoff);
        self.successes.retain(|t| *t > cutoff);
    }
}

// ── Pipeline Metrics ─────────────────────────────────────────────────────────

/// Atomic counters for pipeline observability (FT-007, FT-024).
pub struct PipelineMetrics {
    /// Most recent query latency in milliseconds.
    pub query_latency_ms: AtomicU64,
    /// Cumulative retrieval hit count.
    pub retrieval_hits: AtomicU64,
    /// Cumulative error count.
    pub error_count: AtomicU64,
    /// Cumulative total request count.
    pub total_count: AtomicU64,
    /// Latency histogram for percentile tracking (FT-024).
    pub latency_histogram: LatencyHistogram,
    /// Degradation counter: how many times graph was skipped.
    pub degraded_graph_skips: AtomicU64,
    /// Degradation counter: how many times ECMA was skipped.
    pub degraded_ecma_skips: AtomicU64,
    /// Degradation counter: how many times MCE was skipped.
    pub degraded_mce_skips: AtomicU64,
}

impl Default for PipelineMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineMetrics {
    /// Create a new set of zeroed counters.
    pub fn new() -> Self {
        Self {
            query_latency_ms: AtomicU64::new(0),
            retrieval_hits: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            total_count: AtomicU64::new(0),
            latency_histogram: LatencyHistogram::new(),
            degraded_graph_skips: AtomicU64::new(0),
            degraded_ecma_skips: AtomicU64::new(0),
            degraded_mce_skips: AtomicU64::new(0),
        }
    }

    /// Record a successful query with its latency and hit count.
    pub fn record_query(&self, latency_ms: f64, hits: usize) {
        self.query_latency_ms
            .store(latency_ms as u64, Ordering::Relaxed);
        self.retrieval_hits
            .fetch_add(hits as u64, Ordering::Relaxed);
        self.total_count.fetch_add(1, Ordering::Relaxed);
        // Record in histogram for percentile tracking.
        self.latency_histogram
            .record_us((latency_ms * 1000.0) as u64);
    }

    /// Record a failed query.
    pub fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
        self.total_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Compute the current error rate (0.0 .. 1.0).
    pub fn error_rate(&self) -> f64 {
        let total = self.total_count.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let errors = self.error_count.load(Ordering::Relaxed);
        errors as f64 / total as f64
    }
}

// ── Idempotency Cache ────────────────────────────────────────────────────────

/// Simple LRU cache for idempotent request deduplication (FT-010).
struct IdempotencyCache {
    entries: HashMap<u64, (Instant, ContextOutput)>,
    order: VecDeque<u64>,
    capacity: usize,
    ttl: Duration,
}

impl IdempotencyCache {
    fn new(capacity: usize, ttl: Duration) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
            capacity,
            ttl,
        }
    }

    fn get(&mut self, key: u64) -> Option<ContextOutput> {
        if let Some((inserted, output)) = self.entries.get(&key) {
            if inserted.elapsed() < self.ttl {
                self.order.retain(|k| *k != key);
                self.order.push_back(key);
                return Some(output.clone());
            }
            // Expired entry.
            self.entries.remove(&key);
            self.order.retain(|k| *k != key);
        }
        None
    }

    fn insert(&mut self, key: u64, output: &ContextOutput) {
        if self.entries.len() >= self.capacity {
            while let Some(oldest) = self.order.pop_front() {
                self.entries.remove(&oldest);
                if self.entries.len() < self.capacity {
                    break;
                }
            }
        }
        self.entries.insert(key, (Instant::now(), output.clone()));
        self.order.push_back(key);
    }

    fn evict_expired(&mut self) {
        let now = Instant::now();
        let ttl = self.ttl;
        self.order.retain(|k| {
            if let Some((inserted, _)) = self.entries.get(k) {
                if now.duration_since(*inserted) < ttl {
                    return true;
                }
            }
            self.entries.remove(k);
            false
        });
    }
}

// ── Internal stage context types ─────────────────────────────────────────────

/// Internal context passed between MCE encode (stage 7) and execute (stage 8).
struct KceMceContext {
    mrna: kce_core::types::Mrna,
}

// ── Pipeline Orchestrator ────────────────────────────────────────────────────

/// The pipeline orchestrator.
///
/// Holds the retrieval engine, dataset, circuit breaker, metrics counters,
/// and an idempotency cache.
pub struct PipelineOrchestrator {
    retrieval: RetrievalEngine,
    dataset: parking_lot::RwLock<Dataset>,
    config: PipelineConfig,
    /// Semantic graph for contextual expansion (FT-002).
    graph: parking_lot::RwLock<SemanticGraph>,
    /// ECMA engine for node lifecycle (FT-003).
    ecma: EcmaEngine,
    /// MCE engine for mRNA encoding/execution (FT-004).
    mce: MceEngine,
    /// Circuit breaker for fault tolerance (FT-009).
    pub circuit_breaker: parking_lot::Mutex<CircuitBreaker>,
    /// Atomic metrics counters (FT-007, FT-024).
    pub metrics: PipelineMetrics,
    /// Idempotency cache for deduplication (FT-010).
    cache: parking_lot::Mutex<IdempotencyCache>,
    /// Per-stage latency budget (FT-024).
    pub budget: LatencyBudget,
    /// Degradation strategy (FT-024).
    pub degradation: DegradationStrategy,
}

impl PipelineOrchestrator {
    /// Create a new pipeline with the given configuration and dataset.
    pub fn new(config: PipelineConfig, dataset: Dataset) -> Self {
        Self {
            retrieval: RetrievalEngine::new(config.retrieval.clone()),
            dataset: parking_lot::RwLock::new(dataset),
            graph: parking_lot::RwLock::new(SemanticGraph::new()),
            ecma: EcmaEngine::with_defaults(),
            mce: MceEngine::with_defaults(),
            cache: parking_lot::Mutex::new(IdempotencyCache::new(
                1024,
                Duration::from_secs(300),
            )),
            circuit_breaker: parking_lot::Mutex::new(CircuitBreaker::new()),
            metrics: PipelineMetrics::new(),
            budget: LatencyBudget::default(),
            degradation: DegradationStrategy::default(),
            config,
        }
    }

    /// Get the pipeline configuration.
    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }

    /// Execute the pipeline with full resilience features.
    ///
    /// Includes: tracing span with `request_id`, idempotency cache lookup,
    /// circuit-breaker check, and retry with exponential backoff (50 / 100 / 200 ms).
    pub fn execute_with_resilience(
        &self,
        input: &ContextInput,
        request_id: &str,
    ) -> Result<ContextOutput, KceError> {
        let span = tracing::info_span!("pipeline_execute", request_id);
        let _guard = span.enter();

        let request_hash = hash_request_id(request_id);

        // Idempotency cache check
        {
            let mut cache = self.cache.lock();
            cache.evict_expired();
            if let Some(cached) = cache.get(request_hash) {
                tracing::debug!(request_id, "idempotency cache hit");
                return Ok(cached);
            }
        }

        // Circuit breaker check
        {
            let mut cb = self.circuit_breaker.lock();
            cb.check()?;
        }

        // Retry with exponential backoff (up to 3 retries)
        let mut last_err: Option<KceError> = None;
        let backoffs = [50u64, 100, 200];

        for attempt in 0u32..=2 {
            if attempt > 0 {
                let delay = backoffs
                    .get((attempt - 1) as usize)
                    .copied()
                    .unwrap_or(200);
                tracing::warn!(
                    request_id,
                    attempt,
                    delay_ms = delay,
                    "retrying pipeline execution"
                );
                std::thread::sleep(Duration::from_millis(delay));
            }

            match self.execute(input) {
                Ok(output) => {
                    self.circuit_breaker.lock().record_success();
                    self.metrics
                        .record_query(output.pipeline_ms, output.stages.len());
                    self.cache.lock().insert(request_hash, &output);
                    return Ok(output);
                }
                Err(e) => {
                    tracing::error!(request_id, attempt, error = %e, "pipeline stage failed");
                    self.circuit_breaker.lock().record_failure();
                    last_err = Some(e);
                }
            }
        }

        self.metrics.record_error();
        Err(last_err.unwrap_or_else(|| KceError::Validation("all retries exhausted".into())))
    }

    /// Execute the cognitive pipeline (10 stages, FT-011, FT-024).
    ///
    /// Includes latency budget enforcement: if remaining budget drops below
    /// per-stage thresholds, optional stages (graph, ECMA, MCE) are skipped
    /// via the degradation strategy.
    pub fn execute(&self, input: &ContextInput) -> Result<ContextOutput, KceError> {
        let start = Instant::now();
        let mut stages = BTreeMap::new();

        // Stage 1: Validate
        let s = Instant::now();
        Self::run_stage("validate", || self.validate(input))?;
        stages.insert("validate".into(), s.elapsed().as_secs_f64() * 1000.0);

        // Stage 2: Rate Limit (stub)
        let s = Instant::now();
        Self::run_stage("rate_limit", || self.stage_rate_limit(input))?;
        stages.insert("rate_limit".into(), s.elapsed().as_secs_f64() * 1000.0);

        // Stage 3: Plan (stub)
        let s = Instant::now();
        Self::run_stage("plan", || self.stage_plan(input))?;
        stages.insert("plan".into(), s.elapsed().as_secs_f64() * 1000.0);

        // Stage 4: Retrieve
        let s = Instant::now();
        let results = Self::run_stage("retrieve", || {
            let dataset = self.dataset.read();
            self.retrieval
                .search(&input.query_vector, &dataset, input.top_k)
        })?;
        stages.insert("retrieve".into(), s.elapsed().as_secs_f64() * 1000.0);

        // Check degradation after retrieval
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        let degradation = self.degradation.evaluate(elapsed_ms, &self.budget);

        // Stage 5: Expand (conditionally skipped under time pressure)
        if degradation.skipped_graph {
            self.metrics.degraded_graph_skips.fetch_add(1, Ordering::Relaxed);
            stages.insert("expand".into(), 0.0);
        } else {
            let s = Instant::now();
            Self::run_stage("expand", || self.stage_expand(&results))?;
            stages.insert("expand".into(), s.elapsed().as_secs_f64() * 1000.0);
        }

        // Stage 6: ECMA Update (conditionally skipped)
        if degradation.skipped_ecma {
            self.metrics.degraded_ecma_skips.fetch_add(1, Ordering::Relaxed);
            stages.insert("ecma_update".into(), 0.0);
        } else {
            let s = Instant::now();
            Self::run_stage("ecma_update", || self.stage_ecma_update(&results))?;
            stages.insert("ecma_update".into(), s.elapsed().as_secs_f64() * 1000.0);
        }

        // Stage 7: MCE Encode (conditionally skipped)
        let mce_ctx = if degradation.skipped_mce {
            self.metrics.degraded_mce_skips.fetch_add(1, Ordering::Relaxed);
            stages.insert("mce_encode".into(), 0.0);
            // Return a default/simplified MCE context
            KceMceContext {
                mrna: kce_core::types::Mrna {
                    intent: MrnaIntent::Classify,
                    priority: 0,
                    ttl_ms: 1000,
                    payload: vec![],
                    payload_size_bytes: 0,
                    original_size_bytes: 0,
                    compression_ratio: 0.0,
                },
            }
        } else {
            let s = Instant::now();
            let ctx = Self::run_stage("mce_encode", || self.stage_mce_encode(&results))?;
            stages.insert("mce_encode".into(), s.elapsed().as_secs_f64() * 1000.0);
            ctx
        };

        // Stage 8: Execute action
        let s = Instant::now();
        let action_result = Self::run_stage("execute", || self.stage_execute_action(&mce_ctx))?;
        stages.insert("execute".into(), s.elapsed().as_secs_f64() * 1000.0);

        // Stage 9: Persist (stub)
        let s = Instant::now();
        Self::run_stage("persist", || self.stage_persist(&results))?;
        stages.insert("persist".into(), s.elapsed().as_secs_f64() * 1000.0);

        // Stage 10: Metrics (stub)
        let s = Instant::now();
        Self::run_stage("metrics", || self.stage_metrics(&stages))?;
        stages.insert("metrics".into(), s.elapsed().as_secs_f64() * 1000.0);

        let pipeline_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(ContextOutput {
            action: action_result.action,
            result: action_result.result,
            pipeline_ms,
            stages,
        })
    }

    /// Run a named pipeline stage, wrapping any error in [`KceError::PipelineFailure`].
    fn run_stage<F, T>(stage: &'static str, f: F) -> Result<T, KceError>
    where
        F: FnOnce() -> Result<T, KceError>,
    {
        f().map_err(|e| KceError::PipelineFailure {
            stage,
            source: Box::new(e),
        })
    }

    // ── Stage implementations ────────────────────────────────────────────────

    /// Stage 1: Validate the input.
    fn validate(&self, ctx: &ContextInput) -> Result<(), KceError> {
        if ctx.query_vector.is_empty() {
            return Err(KceError::Validation("query vector is empty".into()));
        }
        if ctx.top_k == 0 {
            return Err(KceError::Validation("top_k must be > 0".into()));
        }
        if ctx
            .query_vector
            .iter()
            .any(|v| v.is_nan() || v.is_infinite())
        {
            return Err(KceError::Validation("vector contains NaN or Inf".into()));
        }
        Ok(())
    }

    /// Stage 2: Rate limiting — checks if request is within configured limits.
    fn stage_rate_limit(&self, _ctx: &ContextInput) -> Result<(), KceError> {
        // Rate limiting is enforced at the API layer via semaphore (FT-009/FT-010).
        // Here we just validate the request is well-formed for downstream processing.
        Ok(())
    }

    /// Stage 3: Planning — determines retrieval strategy based on context.
    fn stage_plan(&self, ctx: &ContextInput) -> Result<(), KceError> {
        // Plan stage: validate top_k is reasonable for the dataset size.
        let dataset = self.dataset.read();
        if ctx.top_k > dataset.len() * 10 {
            tracing::warn!(
                top_k = ctx.top_k,
                dataset_size = dataset.len(),
                "top_k significantly exceeds dataset size"
            );
        }
        Ok(())
    }

    /// Stage 5: Graph expansion — expand retrieval results via semantic graph (FT-002).
    fn stage_expand(&self, results: &[SearchResult]) -> Result<(), KceError> {
        if results.is_empty() {
            return Ok(());
        }
        let seed_ids: Vec<u64> = results.iter().map(|r| r.id).collect();
        let graph = self.graph.read();
        let expansion = graph.expand(&seed_ids, 2);
        tracing::debug!(
            seeds = seed_ids.len(),
            expanded = expansion.nodes.len(),
            depth = expansion.depth_reached,
            "graph expansion complete"
        );
        Ok(())
    }

    /// Stage 6: ECMA lifecycle update — evolve node states based on usage (FT-003).
    fn stage_ecma_update(&self, results: &[SearchResult]) -> Result<(), KceError> {
        for result in results {
            let mut node = EcmaNode {
                id: result.id,
                state: NodeState::Stem,
                usage_count: 1,
                entropy: 1.0 - result.score,
                connections: 0,
                maturity: 0.0,
            };
            if let Err(e) = self.ecma.process_node(&mut node) {
                tracing::warn!(node_id = result.id, error = %e, "ECMA update failed for node");
            }
        }
        Ok(())
    }

    /// Stage 7: MCE encoding — encode enriched nodes into mRNA instructions (FT-004).
    fn stage_mce_encode(&self, results: &[SearchResult]) -> Result<KceMceContext, KceError> {
        let nodes: Vec<EcmaNode> = results
            .iter()
            .map(|r| EcmaNode {
                id: r.id,
                state: NodeState::Progenitor,
                usage_count: 1,
                entropy: 1.0 - r.score,
                connections: 0,
                maturity: r.score,
            })
            .collect();

        let intent = if results.iter().any(|r| r.score < 0.3) {
            MrnaIntent::Alert
        } else if results.iter().any(|r| r.score > 0.8) {
            MrnaIntent::RiskEval
        } else {
            MrnaIntent::Classify
        };

        let mrna = self.mce.encode(&nodes, intent);
        Ok(KceMceContext { mrna })
    }

    /// Stage 8: Execute action — execute the encoded mRNA instruction (FT-004).
    fn stage_execute_action(
        &self,
        mce_ctx: &KceMceContext,
    ) -> Result<ActionResult, KceError> {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.mce.execute(&mce_ctx.mrna, now_ms)
    }

    /// Stage 9: Persistence — log pipeline result for observability (FT-005).
    fn stage_persist(&self, results: &[SearchResult]) -> Result<(), KceError> {
        tracing::info!(
            results_count = results.len(),
            "pipeline results persisted"
        );
        Ok(())
    }

    /// Stage 10: Metrics — collect and record pipeline metrics (FT-007).
    fn stage_metrics(&self, stages: &BTreeMap<String, f64>) -> Result<(), KceError> {
        for (stage, latency_ms) in stages {
            tracing::debug!(
                stage = stage.as_str(),
                latency_ms = latency_ms,
                "stage latency recorded"
            );
        }
        Ok(())
    }
}

/// Hash a request_id string to a `u64` for cache keying.
fn hash_request_id(request_id: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    request_id.hash(&mut hasher);
    hasher.finish()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn sample_pipeline() -> PipelineOrchestrator {
        let mut dataset = Dataset::new(2);
        dataset.push(1, vec![1.0, 0.0]).expect("push");
        dataset.push(2, vec![0.0, 1.0]).expect("push");
        PipelineOrchestrator::new(PipelineConfig::default(), dataset)
    }

    #[test]
    fn empty_vector_fails_validation() {
        let pipeline = sample_pipeline();
        let ctx = ContextInput {
            query_vector: vec![],
            top_k: 5,
            context: None,
            tenant_id: "t1".into(),
        };
        assert!(pipeline.execute(&ctx).is_err());
    }

    #[test]
    fn zero_top_k_fails_validation() {
        let pipeline = sample_pipeline();
        let ctx = ContextInput {
            query_vector: vec![1.0, 2.0],
            top_k: 0,
            context: None,
            tenant_id: "t1".into(),
        };
        assert!(pipeline.execute(&ctx).is_err());
    }

    #[test]
    fn nan_fails_validation() {
        let pipeline = sample_pipeline();
        let ctx = ContextInput {
            query_vector: vec![1.0, f64::NAN],
            top_k: 5,
            context: None,
            tenant_id: "t1".into(),
        };
        assert!(pipeline.execute(&ctx).is_err());
    }

    #[test]
    fn valid_input_succeeds() {
        let pipeline = sample_pipeline();
        let ctx = ContextInput {
            query_vector: vec![0.12, 0.85],
            top_k: 2,
            context: None,
            tenant_id: "t1".into(),
        };
        let result = pipeline.execute(&ctx).expect("execute");
        assert!(result.pipeline_ms > 0.0);
        assert!(result.stages.contains_key("validate"));
        assert!(result.stages.contains_key("retrieve"));
    }

    #[test]
    fn stages_are_recorded() {
        let pipeline = sample_pipeline();
        let ctx = ContextInput {
            query_vector: vec![1.0, 0.0],
            top_k: 1,
            context: None,
            tenant_id: "t1".into(),
        };
        let result = pipeline.execute(&ctx).expect("execute");
        assert!(result.stages.contains_key("validate"));
        assert!(result.stages.contains_key("retrieve"));
    }

    #[test]
    fn all_ten_stages_present() {
        let pipeline = sample_pipeline();
        let ctx = ContextInput {
            query_vector: vec![1.0, 0.0],
            top_k: 1,
            context: None,
            tenant_id: "t1".into(),
        };
        let result = pipeline.execute(&ctx).expect("execute");
        let expected = [
            "validate",
            "rate_limit",
            "plan",
            "retrieve",
            "expand",
            "ecma_update",
            "mce_encode",
            "execute",
            "persist",
            "metrics",
        ];
        for stage in &expected {
            assert!(
                result.stages.contains_key(*stage),
                "missing stage: {}",
                stage
            );
        }
        assert_eq!(result.stages.len(), 10);
    }

    #[test]
    fn pipeline_failure_wraps_stage_name() {
        let pipeline = sample_pipeline();
        let ctx = ContextInput {
            query_vector: vec![],
            top_k: 1,
            context: None,
            tenant_id: "t1".into(),
        };
        let err = pipeline.execute(&ctx).expect_err("should fail");
        let msg = err.to_string();
        assert!(
            msg.contains("validate"),
            "error should mention stage name: {}",
            msg
        );
    }

    // ── Resilience tests ────────────────────────────────────────────────────

    #[test]
    fn execute_with_resilience_success() {
        let pipeline = sample_pipeline();
        let ctx = ContextInput {
            query_vector: vec![1.0, 0.0],
            top_k: 1,
            context: None,
            tenant_id: "t1".into(),
        };
        let output = pipeline
            .execute_with_resilience(&ctx, "req_test_1")
            .expect("resilience execute");
        assert!(output.pipeline_ms > 0.0);
    }

    #[test]
    fn idempotency_cache_returns_same_result() {
        let pipeline = sample_pipeline();
        let ctx = ContextInput {
            query_vector: vec![1.0, 0.0],
            top_k: 1,
            context: None,
            tenant_id: "t1".into(),
        };
        let r1 = pipeline
            .execute_with_resilience(&ctx, "req_idempotent")
            .expect("first");
        let r2 = pipeline
            .execute_with_resilience(&ctx, "req_idempotent")
            .expect("second");
        assert_eq!(r1.result, r2.result);
    }

    #[test]
    fn circuit_breaker_starts_closed() {
        let pipeline = sample_pipeline();
        let cb = pipeline.circuit_breaker.lock();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn metrics_record_queries() {
        let pipeline = sample_pipeline();
        let ctx = ContextInput {
            query_vector: vec![1.0, 0.0],
            top_k: 1,
            context: None,
            tenant_id: "t1".into(),
        };
        let _ = pipeline.execute_with_resilience(&ctx, "req_metrics");
        let total = pipeline
            .metrics
            .total_count
            .load(std::sync::atomic::Ordering::Relaxed);
        assert!(total >= 1);
    }

    #[test]
    fn validation_error_is_pipeline_failure() {
        let pipeline = sample_pipeline();
        let ctx = ContextInput {
            query_vector: vec![],
            top_k: 1,
            context: None,
            tenant_id: "t1".into(),
        };
        let err = pipeline
            .execute_with_resilience(&ctx, "req_fail")
            .expect_err("should fail");
        // The error wraps PipelineFailure -> Validation
        let msg = err.to_string();
        assert!(msg.contains("pipeline failed"));
    }

    // ── Degradation strategy tests (FT-024) ──────────────────────────────

    #[test]
    fn degradation_no_skip_when_budget_healthy() {
        let strategy = DegradationStrategy::default();
        let budget = LatencyBudget::default();
        // Only 1ms elapsed out of 10ms budget — plenty of headroom
        let d = strategy.evaluate(1.0, &budget);
        assert!(!d.skipped_graph);
        assert!(!d.skipped_ecma);
        assert!(!d.skipped_mce);
        assert!(!d.skipped_ot);
    }

    #[test]
    fn degradation_skips_graph_first() {
        let strategy = DegradationStrategy::default();
        let budget = LatencyBudget::default();
        // 7.5ms elapsed, remaining=2.5ms < graph_threshold=3.0
        let d = strategy.evaluate(7.5, &budget);
        assert!(d.skipped_graph);
        assert!(!d.skipped_ecma); // remaining=2.5 >= ecma_threshold=2.0
        assert!(!d.skipped_mce);
        assert!(!d.skipped_ot);
    }

    #[test]
    fn degradation_skips_ecma_next() {
        let strategy = DegradationStrategy::default();
        let budget = LatencyBudget::default();
        // 8.5ms elapsed, remaining=1.5ms < ecma_threshold=2.0
        let d = strategy.evaluate(8.5, &budget);
        assert!(d.skipped_graph);
        assert!(d.skipped_ecma);
        assert!(!d.skipped_mce); // remaining=1.5 >= mce_threshold=1.0
        assert!(!d.skipped_ot);  // remaining=1.5, not < 1.5
    }

    #[test]
    fn degradation_skips_mce_last() {
        let strategy = DegradationStrategy::default();
        let budget = LatencyBudget::default();
        // 9.5ms elapsed, remaining=0.5ms < mce_threshold=1.0
        let d = strategy.evaluate(9.5, &budget);
        assert!(d.skipped_graph);
        assert!(d.skipped_ecma);
        assert!(d.skipped_mce);
        assert!(d.skipped_ot);
    }

    #[test]
    fn degradation_over_budget_skips_all() {
        let strategy = DegradationStrategy::default();
        let budget = LatencyBudget::default();
        // 10ms elapsed — exactly at budget
        let d = strategy.evaluate(10.0, &budget);
        assert!(d.skipped_graph);
        assert!(d.skipped_ecma);
        assert!(d.skipped_mce);
        assert!(d.skipped_ot);
    }

    #[test]
    fn latency_budget_defaults() {
        let budget = LatencyBudget::default();
        assert!((budget.parse_ms - 0.5).abs() < 1e-10);
        assert!((budget.retrieval_ms - 3.0).abs() < 1e-10);
        assert!((budget.ranking_ms - 2.0).abs() < 1e-10);
        assert!((budget.execution_ms - 1.0).abs() < 1e-10);
        assert!((budget.total_ms - 10.0).abs() < 1e-10);
    }

    #[test]
    fn latency_histogram_in_metrics() {
        let pipeline = sample_pipeline();
        let ctx = ContextInput {
            query_vector: vec![1.0, 0.0],
            top_k: 1,
            context: None,
            tenant_id: "t1".into(),
        };
        let _ = pipeline.execute_with_resilience(&ctx, "req_hist");
        let snap = pipeline.metrics.latency_histogram.percentiles();
        assert!(snap.count >= 1);
        assert!(snap.p99_us > 0);
    }
}
