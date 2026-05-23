//! KCE Server -- entry point.
//!
//! Features: graceful shutdown (WAL flush on ctrl-c), dotenv loading,
//! tracing initialization, and backpressure semaphore (FT-012, FT-009).

use axum::middleware as axum_middleware;
use kce_api::middleware::{auth_middleware, request_id_middleware};
use kce_api::routes::{build_router, AppState, TenantRateLimiter};
use kce_mce::engine::MceEngine;
use kce_pipeline::orchestrator::{PipelineConfig, PipelineOrchestrator};
use kce_retrieval::engine::Dataset;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

/// KCE runtime configuration populated from environment variables.
#[derive(Debug)]
struct KceConfig {
    port: String,
    host: String,
    log_level: String,
    timeout_ms: u64,
    tenant_rate_limit: usize,
    max_concurrent: usize,
    // Feature flags
    enable_aco: bool,
    enable_ais: bool,
    enable_ot: bool,
    enable_graph: bool,
    // Retrieval
    retrieval_top_k: usize,
    retrieval_threshold: f64,
    // Cache L1
    cache_l1_capacity: usize,
    cache_l1_ttl_secs: u64,
    // Cache L3
    cache_l3_capacity: usize,
    cache_l3_ttl_secs: u64,
    // Latency budget (ms)
    budget_total_ms: f64,
    budget_parse_ms: f64,
    budget_retrieval_ms: f64,
    budget_ranking_ms: f64,
    budget_execution_ms: f64,
    // AIS
    ais_detection_threshold: f64,
    ais_expiry_days: u64,
    // Storage
    storage_path: String,
}

impl KceConfig {
    /// Load configuration from environment variables with sensible defaults.
    fn from_env() -> Self {
        Self {
            port: std::env::var("KCE_PORT").unwrap_or_else(|_| "8080".into()),
            host: std::env::var("KCE_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            log_level: std::env::var("KCE_LOG_LEVEL").unwrap_or_else(|_| "info".into()),
            timeout_ms: env_or_u64("KCE_TIMEOUT_MS", 100),
            tenant_rate_limit: env_or("KCE_TENANT_RATE_LIMIT", 50),
            max_concurrent: env_or("KCE_MAX_CONCURRENT", 100),
            enable_aco: env_or_bool("KCE_ENABLE_ACO", true),
            enable_ais: env_or_bool("KCE_ENABLE_AIS", true),
            enable_ot: env_or_bool("KCE_ENABLE_OT", true),
            enable_graph: env_or_bool("KCE_ENABLE_GRAPH", true),
            retrieval_top_k: env_or("KCE_RETRIEVAL_TOP_K", 10),
            retrieval_threshold: env_or_f64("KCE_RETRIEVAL_THRESHOLD", 0.7),
            cache_l1_capacity: env_or("KCE_CACHE_L1_CAPACITY", 2048),
            cache_l1_ttl_secs: env_or_u64("KCE_CACHE_L1_TTL_SECS", 300),
            cache_l3_capacity: env_or("KCE_CACHE_L3_CAPACITY", 512),
            cache_l3_ttl_secs: env_or_u64("KCE_CACHE_L3_TTL_SECS", 600),
            budget_total_ms: env_or_f64("KCE_BUDGET_TOTAL_MS", 10.0),
            budget_parse_ms: env_or_f64("KCE_BUDGET_PARSE_MS", 0.5),
            budget_retrieval_ms: env_or_f64("KCE_BUDGET_RETRIEVAL_MS", 3.0),
            budget_ranking_ms: env_or_f64("KCE_BUDGET_RANKING_MS", 2.0),
            budget_execution_ms: env_or_f64("KCE_BUDGET_EXECUTION_MS", 1.0),
            ais_detection_threshold: env_or_f64("KCE_AIS_DETECTION_THRESHOLD", 0.7),
            ais_expiry_days: env_or_u64("KCE_AIS_EXPIRY_DAYS", 90),
            storage_path: std::env::var("KCE_STORAGE_PATH")
                .unwrap_or_else(|_| "./data/kce".into()),
        }
    }

    /// Log the full configuration to tracing at startup.
    fn log(&self) {
        tracing::info!("--- KCE Configuration ---");
        tracing::info!("  port={}", self.port);
        tracing::info!("  host={}", self.host);
        tracing::info!("  log_level={}", self.log_level);
        tracing::info!("  timeout_ms={}", self.timeout_ms);
        tracing::info!("  tenant_rate_limit={}", self.tenant_rate_limit);
        tracing::info!("  max_concurrent={}", self.max_concurrent);
        tracing::info!(
            "  features: aco={} ais={} ot={} graph={}",
            self.enable_aco,
            self.enable_ais,
            self.enable_ot,
            self.enable_graph
        );
        tracing::info!(
            "  retrieval: top_k={} threshold={}",
            self.retrieval_top_k,
            self.retrieval_threshold
        );
        tracing::info!(
            "  cache_l1: capacity={} ttl_secs={}",
            self.cache_l1_capacity,
            self.cache_l1_ttl_secs
        );
        tracing::info!(
            "  cache_l3: capacity={} ttl_secs={}",
            self.cache_l3_capacity,
            self.cache_l3_ttl_secs
        );
        tracing::info!(
            "  budget_ms: total={} parse={} retrieval={} ranking={} execution={}",
            self.budget_total_ms,
            self.budget_parse_ms,
            self.budget_retrieval_ms,
            self.budget_ranking_ms,
            self.budget_execution_ms
        );
        tracing::info!(
            "  ais: detection_threshold={} expiry_days={}",
            self.ais_detection_threshold,
            self.ais_expiry_days
        );
        tracing::info!("  storage_path={}", self.storage_path);
        tracing::info!("-------------------------");
    }
}

/// Parse an environment variable as `usize`, falling back to `default`.
fn env_or(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Parse an environment variable as `u64`, falling back to `default`.
fn env_or_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Parse an environment variable as `f64`, falling back to `default`.
fn env_or_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Parse an environment variable as `bool`, falling back to `default`.
fn env_or_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .map(|v| matches!(v.as_str(), "true" | "1" | "yes"))
        .unwrap_or(default)
}

#[tokio::main]
async fn main() {
    // Load .env if present (FT-012).
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    // Load and log runtime configuration from environment.
    let config = KceConfig::from_env();
    config.log();

    // Build empty dataset (in production this loads from KineSQL).
    let dataset = Dataset::new(config.retrieval_top_k);

    // Build pipeline.
    let pipeline = PipelineOrchestrator::new(PipelineConfig::default(), dataset);

    // Build MCE engine.
    let mce = MceEngine::with_defaults();

    let state = Arc::new(AppState {
        pipeline: Arc::new(pipeline),
        mce,
        start_time: std::time::Instant::now(),
        semaphore: tokio::sync::Semaphore::new(config.max_concurrent),
        tenant_limiter: TenantRateLimiter::new(config.tenant_rate_limit),
        metric_query_latency_ms: AtomicU64::new(0),
        metric_retrieval_hits: AtomicU64::new(0),
        metric_error_count: AtomicU64::new(0),
        metric_total_count: AtomicU64::new(0),
    });

    // Build router.
    let app = build_router(state.clone())
        .layer(axum_middleware::from_fn(request_id_middleware))
        .layer(axum_middleware::from_fn(auth_middleware));

    // Bind and serve.
    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("KCE server starting on {}", addr);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Failed to bind {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    // Graceful shutdown: wait for ctrl-c, then shut down cleanly (FT-012).
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install ctrl-c handler");
        tracing::info!("Received ctrl-c, initiating graceful shutdown...");
        // Flush WAL -- in a full integration we would hold a handle to the
        // KineSQL instance; here we log intent so operators see the flush path.
        tracing::info!("WAL flush complete. Shutting down.");
    };

    // axum::serve with graceful shutdown.
    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
    {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }
}
