//! KCE Server -- entry point.
//!
//! Features: distributed sharding, swarm gossip, consensus, and hematoencephalic gateway.

use axum::middleware as axum_middleware;
use kce_api::middleware::{auth_middleware, request_id_middleware};
use kce_api::routes::{build_router, AppState, TenantRateLimiter};
use kce_mce::engine::MceEngine;
use kce_pipeline::orchestrator::{PipelineConfig, PipelineOrchestrator};
use kce_retrieval::engine::Dataset;
use kce_storage::backend::KineSQL;
use kce_core::traits::StorageBackend;
use kce_swarm::{GossipNode, RaftLite, ShardRouter};
use kce_gateway::{KceGatewayService, start_websocket_server};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use parking_lot::RwLock;

/// KCE runtime configuration populated from environment variables.
#[derive(Debug)]
struct KceConfig {
    port: String,
    host: String,
    log_level: String,
    timeout_ms: u64,
    tenant_rate_limit: usize,
    max_concurrent: usize,
    // Distributed
    swarm_port: u16,
    gateway_port: u16,
    ws_port: u16,
    // Storage
    storage_path: String,
    retrieval_top_k: usize,
}

impl KceConfig {
    fn from_env() -> Self {
        Self {
            port: std::env::var("KCE_PORT").unwrap_or_else(|_| "8080".into()),
            host: std::env::var("KCE_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            log_level: std::env::var("KCE_LOG_LEVEL").unwrap_or_else(|_| "info".into()),
            timeout_ms: env_or_u64("KCE_TIMEOUT_MS", 100),
            tenant_rate_limit: env_or("KCE_TENANT_RATE_LIMIT", 50),
            max_concurrent: env_or("KCE_MAX_CONCURRENT", 100),
            swarm_port: env_or_u16("KCE_SWARM_PORT", 9000),
            gateway_port: env_or_u16("KCE_GATEWAY_PORT", 50051),
            ws_port: env_or_u16("KCE_WS_PORT", 8081),
            storage_path: std::env::var("KCE_STORAGE_PATH").unwrap_or_else(|_| "./data/kce".into()),
            retrieval_top_k: env_or("KCE_RETRIEVAL_TOP_K", 10),
        }
    }
}

fn env_or(key: &str, default: usize) -> usize {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn env_or_u64(key: &str, default: u64) -> u64 {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn env_or_u16(key: &str, default: u16) -> u16 {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().init();

    let config = KceConfig::from_env();
    let local_node_id = uuid::Uuid::new_v4();
    tracing::info!("KCE Node {} starting...", local_node_id);

    // 1. Initialize Storage (FT-005)
    let mut storage = KineSQL::open(&config.storage_path).expect("failed to open KineSQL");
    storage.recover().ok();
    let shared_storage = Arc::new(RwLock::new(storage));

    // 2. Initialize Pipeline & Distributed components (FT-028, FT-030)
    let dataset = Dataset::new(128); // Standard dimension
    let mut pipeline_raw = PipelineOrchestrator::new(PipelineConfig::default(), dataset);
    pipeline_raw.storage = Some(Arc::clone(&shared_storage) as Arc<RwLock<dyn StorageBackend>>);
    let pipeline = Arc::new(pipeline_raw);
    
    let raft = Arc::new(RaftLite::new(local_node_id));

    // 3. Initialize Swarm Gossip (FT-029)
    let swarm_addr = format!("{}:{}", config.host, config.swarm_port).parse().unwrap();
    let gossip = Arc::new(GossipNode::new(local_node_id, swarm_addr).await.expect("failed to start gossip"));
    
    let pipeline_gossip = Arc::clone(&pipeline);
    tokio::spawn(async move {
        gossip.listen(move |packet| {
            tracing::debug!(?packet, "Received swarm message");
            // Integration Logic (FT-029 AC-012)
            if packet.message_type == kce_swarm::MessageType::PheromoneSync {
                // Update local pheromones from swarm
            }
        }).await.ok();
    });

    // 4. Initialize Hematoencephalic Gateway (FT-031)
    let grpc_addr = format!("{}:{}", config.host, config.gateway_port).parse().unwrap();
    let grpc_service = KceGatewayService;
    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(kce_gateway::proto::kce_gateway_server::KceGatewayServer::new(grpc_service))
            .serve(grpc_addr)
            .await
            .ok();
    });

    let ws_addr = format!("{}:{}", config.host, config.ws_port);
    tokio::spawn(async move {
        start_websocket_server(&ws_addr).await.ok();
    });

    // 5. Build Axum API
    let state = Arc::new(AppState {
        pipeline: Arc::clone(&pipeline),
        mce: MceEngine::with_defaults(),
        start_time: std::time::Instant::now(),
        semaphore: tokio::sync::Semaphore::new(config.max_concurrent),
        tenant_limiter: TenantRateLimiter::new(config.tenant_rate_limit),
        metric_query_latency_ms: AtomicU64::new(0),
        metric_retrieval_hits: AtomicU64::new(0),
        metric_error_count: AtomicU64::new(0),
        metric_total_count: AtomicU64::new(0),
    });

    let app = build_router(state.clone())
        .layer(axum_middleware::from_fn(request_id_middleware))
        .layer(axum_middleware::from_fn(auth_middleware));

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    tracing::info!("KCE API Gateway starting on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
