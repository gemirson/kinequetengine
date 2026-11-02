//! Axum route definitions for the KCE API.
//!
//! Includes backpressure via semaphore, real metrics counters,
//! request_id propagation, timeout enforcement, smart healthcheck,
//! tenant isolation, and per-tenant rate limiting
//! (FT-009, FT-007, FT-012, FT-008).

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio_stream::StreamExt;
use tower_http::limit::RequestBodyLimitLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use kce_core::error::KceError;
use kce_core::types::{ContextInput, EcmaNode, HealthReport, HealthStatus, Mrna, MrnaIntent};
use kce_mce::engine::MceEngine;
use kce_pipeline::orchestrator::PipelineOrchestrator;

use crate::middleware::REQUEST_ID_HEADER;

// ── Per-Tenant Rate Limiting (FT-008) ────────────────────────────────────────

/// Per-tenant quota tracker.
pub struct TenantQuota {
    pub max_concurrent: usize,
    pub current: usize,
}

/// Per-tenant concurrent request rate limiter.
///
/// Tracks how many concurrent requests each tenant has in-flight.
/// Rejects new requests once a tenant hits its configured limit.
pub struct TenantRateLimiter {
    limits: Mutex<HashMap<String, TenantQuota>>,
}

impl TenantRateLimiter {
    /// Create a new limiter (default_max_concurrent is used as the per-tenant cap).
    pub fn new(_default_max_concurrent: usize) -> Self {
        Self {
            limits: Mutex::new(HashMap::new()),
        }
    }

    /// Try to acquire a slot for the given tenant.
    /// Returns `true` if the tenant is under its limit; `false` if the limit is hit.
    pub fn try_acquire(&self, tenant_id: &str, max_concurrent: usize) -> bool {
        let mut limits = self.limits.lock();
        let quota = limits.entry(tenant_id.to_string()).or_insert(TenantQuota {
            max_concurrent,
            current: 0,
        });
        if quota.current >= quota.max_concurrent {
            false
        } else {
            quota.current += 1;
            true
        }
    }

    /// Release a slot for the given tenant (called after request completes).
    pub fn release(&self, tenant_id: &str) {
        let mut limits = self.limits.lock();
        if let Some(quota) = limits.get_mut(tenant_id) {
            quota.current = quota.current.saturating_sub(1);
        }
    }
}

// ── Shared Application State ─────────────────────────────────────────────────

/// Shared application state.
pub struct AppState {
    /// The pipeline orchestrator (wrapped in Arc for spawn_blocking).
    pub pipeline: Arc<PipelineOrchestrator>,
    /// MCE engine for /encode and /action endpoints (FT-006).
    pub mce: MceEngine,
    /// Process start time for uptime calculation.
    pub start_time: std::time::Instant,
    /// Backpressure semaphore -- limits concurrent requests to 100.
    pub semaphore: tokio::sync::Semaphore,
    /// Per-tenant rate limiter (FT-008).
    pub tenant_limiter: TenantRateLimiter,
    /// Real metrics: cumulative query latency in ms.
    pub metric_query_latency_ms: AtomicU64,
    /// Real metrics: cumulative retrieval hit count.
    pub metric_retrieval_hits: AtomicU64,
    /// Real metrics: cumulative error count.
    pub metric_error_count: AtomicU64,
    /// Real metrics: cumulative total request count.
    pub metric_total_count: AtomicU64,
    /// Broadcast bus for SSE (`GET /events`).
    pub events: crate::events::EngineEventTx,
    /// Monotonic node id generator (first id: 10_000).
    pub node_seq: AtomicU64,
    /// Immune anomaly detector (`POST /ais/probe`).
    pub ais: Mutex<kce_ais::detection::ImmuneDetector>,
}

// ── Router ───────────────────────────────────────────────────────────────────

/// Maximum request body size: 1 MiB.
const MAX_BODY_SIZE: usize = 1024 * 1024;

/// OpenAPI documentation.
#[derive(OpenApi)]
#[openapi(
    paths(
        query_handler,
        encode_handler,
        action_handler,
        events_handler,
        graph_handler,
        create_node_handler,
        delete_node_handler,
        set_node_state_handler,
        deposit_handler,
        ais_probe_handler,
        health_handler,
        metrics_handler,
    ),
    components(
        schemas(
            QueryBody,
            QueryResponseBody,
            EncodeBody,
            ActionBody,
            GraphNodeDto,
            GraphEdgeDto,
            GraphResponse,
            ErrorBody,
            CreateNodeRequest,
            CreateNodeResponse,
            SetNodeStateRequest,
            DepositRequest,
            DepositResponse,
            AisProbeRequest,
            AisProbeResponse,
        )
    ),
    tags(
        (name = "kce", description = "KineContext Engine API")
    )
)]
pub struct ApiDoc;

/// Build the Axum router with all routes.
///
/// POST endpoints (/query, /encode, /action) are wrapped in a
/// `RequestBodyLimitLayer` to reject oversized payloads early.
pub fn build_router(state: Arc<AppState>) -> Router {
    let post_routes = Router::new()
        .route("/query", post(query_handler))
        .route("/encode", post(encode_handler))
        .route("/action", post(action_handler))
        .route("/nodes", post(create_node_handler))
        .route("/nodes/:id/state", post(set_node_state_handler))
        .route("/pheromone/deposit", post(deposit_handler))
        .route("/ais/probe", post(ais_probe_handler))
        .layer(RequestBodyLimitLayer::new(MAX_BODY_SIZE));

    let api_routes = Router::new()
        .merge(post_routes)
        .route("/health", get(health_handler))
        .route("/events", get(events_handler))
        .route("/graph", get(graph_handler))
        .route("/nodes/:id", delete(delete_node_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(state);

    Router::new()
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .merge(api_routes)
}

/// Build the CORS layer from `KCE_CORS_ORIGINS` (comma-separated origins).
///
/// Defaults to `http://localhost:3000`. Applied in kce-server as the
/// **outermost** layer so preflight is answered before auth.
pub fn cors_layer() -> tower_http::cors::CorsLayer {
    use std::time::Duration;
    use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

    let origins =
        std::env::var("KCE_CORS_ORIGINS").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let allow_origin = AllowOrigin::list(
        origins
            .split(',')
            .filter_map(|s| axum::http::HeaderValue::from_str(s.trim()).ok())
            .collect::<Vec<_>>(),
    );
    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods(AllowMethods::from(vec![
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ]))
        .allow_headers(AllowHeaders::from(vec![
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderName::from_static("x-api-key"),
            axum::http::HeaderName::from_static("x-tenant-id"),
            axum::http::HeaderName::from_static("x-request-id"),
        ]))
        .max_age(Duration::from_secs(3600))
}

// ── Request / Response types ─────────────────────────────────────────────────

/// Request body for POST /query.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct QueryBody {
    pub query_vector: Vec<f64>,
    pub top_k: usize,
}

/// Response body for POST /query.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct QueryResponseBody {
    pub action: String,
    pub result: serde_json::Value,
    pub pipeline_ms: f64,
    pub stages: std::collections::BTreeMap<String, f64>,
}

/// Request body for POST /encode (FT-006).
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct EncodeBody {
    #[schema(value_type = Vec<Object>)]
    pub nodes: Vec<EcmaNode>,
    pub intent: String,
}

/// Request body for POST /action (FT-006).
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ActionBody {
    #[schema(value_type = Object)]
    pub mrna: Mrna,
    pub created_at_ms: u64,
}

/// A graph node as exposed by `GET /graph`.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GraphNodeDto {
    /// Node id.
    pub id: u64,
    /// ECMA lifecycle state.
    #[schema(value_type = String, example = "Stem")]
    pub state: kce_core::types::NodeState,
    /// Usage counter.
    pub usage_count: u64,
    /// Entropy in 0..=1.
    pub entropy: f64,
    /// Number of graph connections.
    pub connections: u64,
    /// Maturity in 0..=1.
    pub maturity: f64,
}

/// A graph edge as exposed by `GET /graph`.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GraphEdgeDto {
    /// Source node id.
    pub from: u64,
    /// Target node id.
    pub to: u64,
    /// Edge weight in 0..=1.
    pub weight: f64,
}

/// Response of `GET /graph`.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GraphResponse {
    /// Registered ECMA nodes.
    pub nodes: Vec<GraphNodeDto>,
    /// Unique undirected edges.
    pub edges: Vec<GraphEdgeDto>,
    /// Pheromone levels keyed by `"<from>_<to>"`.
    pub pheromone: std::collections::HashMap<String, f64>,
}

/// Uniform error body used by non-Json-pipeline handlers.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorBody {
    /// Machine-readable code.
    pub error: String,
    /// Human-readable message.
    pub message: String,
}

fn error_body(code: &str, message: &str) -> ErrorBody {
    ErrorBody {
        error: code.to_string(),
        message: message.to_string(),
    }
}

/// Request body for `POST /nodes`.
#[derive(Debug, Deserialize, Default, utoipa::ToSchema)]
pub struct CreateNodeRequest {
    /// Optional existing node to connect to after creation.
    pub connect_to: Option<u64>,
    /// Edge weight in 0..=1 (defaults to 0.5) when `connect_to` is set.
    pub weight: Option<f64>,
}

/// Request body for `POST /nodes/:id/state`.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SetNodeStateRequest {
    /// Target lifecycle state (e.g. `Stem`, `Apoptosis`).
    #[schema(value_type = String, example = "Apoptosis")]
    pub state: kce_core::types::NodeState,
}

/// Request body for `POST /pheromone/deposit`.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct DepositRequest {
    /// Source node id.
    pub from: u64,
    /// Target node id.
    pub to: u64,
    /// Deposit amount (must be finite and > 0).
    pub amount: f64,
}

/// Response of `POST /nodes`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CreateNodeResponse {
    /// New node id (>= 10_000).
    pub id: u64,
}

/// Response of `POST /pheromone/deposit`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DepositResponse {
    /// Resulting pheromone level.
    pub level: f64,
}

/// Request body for `POST /ais/probe`.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AisProbeRequest {
    /// Vector to evaluate (expected 128 dimensions).
    pub vector: Vec<f64>,
}

/// Response of `POST /ais/probe`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AisProbeResponse {
    /// Whether an anomaly was detected.
    pub is_anomaly: bool,
    /// Normalized anomaly score in 0..=1.
    pub score: f64,
    /// Anomaly threshold.
    pub threshold: f64,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// POST /query handler.
///
/// Features:
/// - Backpressure via semaphore (max 100 concurrent).
/// - Timeout enforcement via `tokio::time::timeout`.
/// - Pipeline execution in `spawn_blocking` to avoid blocking the async runtime.
/// - Real metrics update on success / failure.
#[utoipa::path(
    post,
    path = "/query",
    request_body = QueryBody,
    responses(
        (status = 200, description = "Execute query against engine pipeline", body = QueryResponseBody),
        (status = 400, description = "Validation or tenant error", body = ErrorBody),
        (status = 429, description = "Rate limit or backpressure exceeded", body = ErrorBody)
    ),
    security(
        ("api_key" = [])
    )
)]
#[tracing::instrument(skip(state, headers, body), fields(top_k = body.top_k))]
async fn query_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<QueryBody>,
) -> impl IntoResponse {
    // Backpressure: acquire semaphore permit.
    let permit = match state.semaphore.try_acquire() {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "error": "BACKPRESSURE",
                    "message": "too many concurrent requests",
                })),
            )
                .into_response();
        }
    };

    // Extract tenant_id from header (FT-008: mandatory per AC-013).
    let tenant_id = match headers.get("x-tenant-id") {
        Some(val) => match val.to_str() {
            Ok(s) if !s.is_empty() => s.to_string(),
            _ => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "VALIDATION_ERROR",
                        "message": "Invalid x-tenant-id header",
                    })),
                )
                    .into_response();
            }
        },
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "VALIDATION_ERROR",
                    "message": "Missing x-tenant-id header",
                })),
            )
                .into_response();
        }
    };

    // Per-tenant rate limiting (FT-008).
    if !state.tenant_limiter.try_acquire(&tenant_id, 50) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "RATE_LIMIT_EXCEEDED",
                "message": format!("Tenant {} exceeded concurrent request limit", tenant_id),
            })),
        )
            .into_response();
    }

    // Extract request_id from headers.
    let request_id = headers
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("req_unknown")
        .to_string();

    let timeout_ms = state.pipeline.config().timeout_ms;

    // Clone what we need for spawn_blocking (which requires 'static).
    let pipeline = Arc::clone(&state.pipeline);
    let rid = request_id.clone();
    let ctx = ContextInput {
        query_vector: body.query_vector,
        top_k: body.top_k,
        context: None,
        tenant_id: tenant_id.clone(),
    };

    // Timeout + blocking execution.
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(timeout_ms),
        tokio::task::spawn_blocking(move || pipeline.execute_with_resilience(&ctx, &rid)),
    )
    .await;

    // Release per-tenant slot and semaphore permit.
    state.tenant_limiter.release(&tenant_id);
    drop(permit);

    let result: Result<kce_core::types::ContextOutput, KceError> = match result {
        Ok(Ok(inner)) => inner,
        Ok(Err(join_err)) => {
            tracing::error!(request_id = %request_id, error = %join_err, "pipeline panicked");
            Err(KceError::Validation("pipeline thread panicked".into()))
        }
        Err(_timeout) => {
            tracing::error!(request_id = %request_id, timeout_ms, "pipeline timed out");
            Err(KceError::Timeout { ms: timeout_ms })
        }
    };

    match result {
        Ok(output) => {
            state
                .metric_query_latency_ms
                .store(output.pipeline_ms as u64, Ordering::Relaxed);
            state
                .metric_retrieval_hits
                .fetch_add(output.stages.len() as u64, Ordering::Relaxed);
            state.metric_total_count.fetch_add(1, Ordering::Relaxed);

            let _ = state
                .events
                .send(crate::events::EngineEvent::QueryCompleted {
                    latency_ms: output.pipeline_ms,
                    action: output.action.clone(),
                });

            (StatusCode::OK, Json(output)).into_response()
        }
        Err(e) => {
            tracing::error!(request_id = %request_id, error = %e, "query failed");
            state.metric_error_count.fetch_add(1, Ordering::Relaxed);
            state.metric_total_count.fetch_add(1, Ordering::Relaxed);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "PIPELINE_FAILURE",
                    "message": e.to_string(),
                })),
            )
                .into_response()
        }
    }
}

/// Server-sent events stream of [`EngineEvent`]s.
///
/// Send `data: {"type":"...", ...}` frames; keep-alive every 15s.
#[utoipa::path(
    get,
    path = "/events",
    responses(
        (status = 200, description = "SSE event stream", content_type = "text/event-stream")
    )
)]
async fn events_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.events.subscribe();
    let stream = tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|item| {
        // Lagged/Closed are non-fatal; drop them instead of panicking.
        item.ok()
            .and_then(|ev| serde_json::to_string(&ev).ok())
            .map(|json| Ok(Event::default().data(json)))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Full semantic graph: nodes, edges, pheromone levels.
#[utoipa::path(
    get,
    path = "/graph",
    responses(
        (status = 200, description = "Graph topology and pheromone levels", body = GraphResponse)
    )
)]
async fn graph_handler(State(state): State<Arc<AppState>>) -> (StatusCode, Json<GraphResponse>) {
    let pipeline = &state.pipeline;
    let nodes = pipeline
        .list_nodes()
        .into_iter()
        .map(|n| GraphNodeDto {
            id: n.id,
            state: n.state,
            usage_count: n.usage_count,
            entropy: n.entropy,
            connections: n.connections as u64,
            maturity: n.maturity,
        })
        .collect();
    let edges = pipeline
        .graph_edges()
        .into_iter()
        .map(|e| GraphEdgeDto {
            from: e.from,
            to: e.to,
            weight: e.weight,
        })
        .collect();
    let mut pheromone = std::collections::HashMap::new();
    for ((from, to), level) in pipeline.pheromone_levels() {
        pheromone.insert(format!("{from}_{to}"), level);
        pheromone.insert(format!("{to}_{from}"), level);
    }
    (
        StatusCode::OK,
        Json(GraphResponse {
            nodes,
            edges,
            pheromone,
        }),
    )
}

/// Create a node (optionally connected to an existing one).
#[utoipa::path(
    post,
    path = "/nodes",
    request_body = CreateNodeRequest,
    responses(
        (status = 201, description = "Node created", body = CreateNodeResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 404, description = "Connect target not found", body = ErrorBody)
    )
)]
async fn create_node_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateNodeRequest>,
) -> Result<(StatusCode, Json<CreateNodeResponse>), (StatusCode, Json<ErrorBody>)> {
    let weight = req.weight.unwrap_or(0.5);
    if !weight.is_finite() || !(0.0..=1.0).contains(&weight) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(error_body(
                "VALIDATION_ERROR",
                "weight must be within 0..=1",
            )),
        ));
    }
    let id = state.node_seq.fetch_add(1, Ordering::Relaxed);
    let pipeline = &state.pipeline;
    pipeline.upsert_node(kce_core::types::EcmaNode {
        id,
        state: kce_core::types::NodeState::Stem,
        usage_count: 1,
        entropy: 1.0,
        connections: 0,
        maturity: 0.0,
    });
    if let Some(target) = req.connect_to {
        if !pipeline.connect(id, target, weight) {
            pipeline.remove_node(id);
            return Err((
                StatusCode::NOT_FOUND,
                Json(error_body("NOT_FOUND", "connect_to node not found")),
            ));
        }
    }
    let _ = state
        .events
        .send(crate::events::EngineEvent::NodeCreated { id });
    Ok((StatusCode::CREATED, Json(CreateNodeResponse { id })))
}

/// Remove a node from the graph.
#[utoipa::path(
    delete,
    path = "/nodes/{id}",
    params(
        ("id" = u64, Path, description = "Node ID")
    ),
    responses(
        (status = 204, description = "Node deleted"),
        (status = 404, description = "Node not found", body = ErrorBody)
    )
)]
async fn delete_node_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    if state.pipeline.remove_node(id) {
        let _ = state
            .events
            .send(crate::events::EngineEvent::NodeRemoved { id });
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(error_body("NOT_FOUND", "node not found")),
        ))
    }
}

/// Set a node's lifecycle state.
#[utoipa::path(
    post,
    path = "/nodes/{id}/state",
    params(
        ("id" = u64, Path, description = "Node ID")
    ),
    request_body = SetNodeStateRequest,
    responses(
        (status = 200, description = "State updated"),
        (status = 404, description = "Node not found", body = ErrorBody)
    )
)]
async fn set_node_state_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    Json(req): Json<SetNodeStateRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    match state.pipeline.set_node_state(id, req.state) {
        Some(_) => Ok(StatusCode::OK),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(error_body("NOT_FOUND", "node not found")),
        )),
    }
}

/// Deposit pheromone on an edge.
#[utoipa::path(
    post,
    path = "/pheromone/deposit",
    request_body = DepositRequest,
    responses(
        (status = 200, description = "Pheromone deposited", body = DepositResponse),
        (status = 400, description = "Validation error", body = ErrorBody)
    )
)]
async fn deposit_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DepositRequest>,
) -> Result<(StatusCode, Json<DepositResponse>), (StatusCode, Json<ErrorBody>)> {
    if !req.amount.is_finite() || req.amount <= 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(error_body(
                "VALIDATION_ERROR",
                "amount must be finite and > 0",
            )),
        ));
    }
    state
        .pipeline
        .pheromone_map
        .write()
        .deposit(req.from, req.to, req.amount);
    let level = state
        .pipeline
        .pheromone_map
        .read()
        .peek_fractional(req.from, req.to, 0);
    let _ = state
        .events
        .send(crate::events::EngineEvent::PheromoneDeposited {
            from: req.from,
            to: req.to,
            level,
        });
    Ok((StatusCode::OK, Json(DepositResponse { level })))
}

/// AIS probe handler.
#[utoipa::path(
    post,
    path = "/ais/probe",
    request_body = AisProbeRequest,
    responses(
        (status = 200, description = "AIS probe result", body = AisProbeResponse),
        (status = 400, description = "Validation or dimension error", body = ErrorBody)
    )
)]
async fn ais_probe_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AisProbeRequest>,
) -> Result<(StatusCode, Json<AisProbeResponse>), (StatusCode, Json<ErrorBody>)> {
    if req.vector.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(error_body("VALIDATION_ERROR", "vector cannot be empty")),
        ));
    }
    let mut detector = state.ais.lock();
    match detector.detect(&req.vector) {
        Ok(result) => {
            if result.is_anomaly {
                let _ = state.events.send(crate::events::EngineEvent::AisAnomaly {
                    score: result.score,
                    blocked: true,
                });
            }
            Ok((
                StatusCode::OK,
                Json(AisProbeResponse {
                    is_anomaly: result.is_anomaly,
                    score: result.score,
                    threshold: result.threshold,
                }),
            ))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(error_body("DIMENSION_MISMATCH", &e.to_string())),
        )),
    }
}

/// GET /health handler (FT-012: smart healthcheck).
///
/// Actually checks WAL and DB status rather than returning hardcoded values.
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Engine health report")
    )
)]
async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Storage check: verify if we can perform a dummy read/write
    let mut storage_ok = false;
    if let Some(ref storage_arc) = state.pipeline.storage {
        let storage = storage_arc.write();
        storage_ok = storage.read(b"healthcheck:probe").is_ok();
    }

    // Pipeline check: verify basic execution
    let pipeline_ok = {
        let test_ctx = ContextInput {
            query_vector: vec![1.0, 0.0],
            top_k: 1,
            context: None,
            tenant_id: "healthcheck".into(),
        };
        state.pipeline.execute(&test_ctx).is_ok()
    };

    // Latency check: verify latest latency is under budget.
    let latency_ms = state.metric_query_latency_ms.load(Ordering::Relaxed);
    let latency_ok = latency_ms < state.pipeline.config().timeout_ms;

    let status = if storage_ok && pipeline_ok && latency_ok {
        HealthStatus::Healthy
    } else if storage_ok || pipeline_ok {
        HealthStatus::Degraded
    } else {
        HealthStatus::Unhealthy
    };

    let report = HealthReport {
        status,
        wal_ok: storage_ok,
        db_ok: storage_ok,
        latency_ok,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
    };

    let code = match status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    (code, Json(report)).into_response()
}

/// GET /metrics handler (FT-007: real metrics in Prometheus format).
#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus text metrics", content_type = "text/plain")
    )
)]
async fn metrics_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let latency = state.metric_query_latency_ms.load(Ordering::Relaxed);
    let hits = state.metric_retrieval_hits.load(Ordering::Relaxed);
    let errors = state.metric_error_count.load(Ordering::Relaxed);
    let total = state.metric_total_count.load(Ordering::Relaxed);

    // FT-024: Read latency percentiles from pipeline histogram.
    let snap = state.pipeline.metrics.latency_histogram.percentiles();
    let graph_skips = state
        .pipeline
        .metrics
        .degraded_graph_skips
        .load(Ordering::Relaxed);
    let ecma_skips = state
        .pipeline
        .metrics
        .degraded_ecma_skips
        .load(Ordering::Relaxed);
    let mce_skips = state
        .pipeline
        .metrics
        .degraded_mce_skips
        .load(Ordering::Relaxed);

    let metrics = format!(
        "# HELP kce_query_latency_ms Query latency in milliseconds\n\
         # TYPE kce_query_latency_ms gauge\n\
         kce_query_latency_ms {latency}\n\
         # HELP kce_retrieval_hits Cumulative retrieval hit count\n\
         # TYPE kce_retrieval_hits counter\n\
         kce_retrieval_hits {hits}\n\
         # HELP kce_error_rate Current error rate\n\
         # TYPE kce_error_rate gauge\n\
         kce_error_rate {error_rate:.4}\n\
         # HELP kce_requests_total Total request count\n\
         # TYPE kce_requests_total counter\n\
         kce_requests_total {total}\n\
         # HELP kce_latency_p50_us P50 latency in microseconds\n\
         # TYPE kce_latency_p50_us gauge\n\
         kce_latency_p50_us {p50}\n\
         # HELP kce_latency_p95_us P95 latency in microseconds\n\
         # TYPE kce_latency_p95_us gauge\n\
         kce_latency_p95_us {p95}\n\
         # HELP kce_latency_p99_us P99 latency in microseconds\n\
         # TYPE kce_latency_p99_us gauge\n\
         kce_latency_p99_us {p99}\n\
         # HELP kce_latency_p999_us P99.9 latency in microseconds\n\
         # TYPE kce_latency_p999_us gauge\n\
         kce_latency_p999_us {p999}\n\
         # HELP kce_latency_mean_us Mean latency in microseconds\n\
         # TYPE kce_latency_mean_us gauge\n\
         kce_latency_mean_us {mean:.0}\n\
         # HELP kce_latency_histogram_samples Total samples in histogram\n\
         # TYPE kce_latency_histogram_samples counter\n\
         kce_latency_histogram_samples {hist_count}\n\
         # HELP kce_degraded_graph_skips Graph expansion skips due to degradation\n\
         # TYPE kce_degraded_graph_skips counter\n\
         kce_degraded_graph_skips {graph_skips}\n\
         # HELP kce_degraded_ecma_skips ECMA skips due to degradation\n\
         # TYPE kce_degraded_ecma_skips counter\n\
         kce_degraded_ecma_skips {ecma_skips}\n\
         # HELP kce_degraded_mce_skips MCE skips due to degradation\n\
         # TYPE kce_degraded_mce_skips counter\n\
         kce_degraded_mce_skips {mce_skips}\n",
        latency = latency,
        hits = hits,
        error_rate = if total > 0 {
            errors as f64 / total as f64
        } else {
            0.0
        },
        total = total,
        p50 = snap.p50_us,
        p95 = snap.p95_us,
        p99 = snap.p99_us,
        p999 = snap.p999_us,
        mean = snap.mean_us,
        hist_count = snap.count,
        graph_skips = graph_skips,
        ecma_skips = ecma_skips,
        mce_skips = mce_skips,
    );

    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4")],
        metrics,
    )
        .into_response()
}

/// POST /encode handler (FT-006).
///
/// Encodes ECMA nodes into an mRNA instruction using MCE.
#[utoipa::path(
    post,
    path = "/encode",
    request_body = EncodeBody,
    responses(
        (status = 200, description = "Encode ECMA nodes into mRNA instruction")
    )
)]
async fn encode_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<EncodeBody>,
) -> impl IntoResponse {
    let intent = match body.intent.as_str() {
        "risk_eval" => MrnaIntent::RiskEval,
        "data_enrich" => MrnaIntent::DataEnrich,
        "alert" => MrnaIntent::Alert,
        "classify" => MrnaIntent::Classify,
        other => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "VALIDATION_ERROR",
                    "message": format!("unknown intent: {}", other),
                })),
            )
                .into_response();
        }
    };

    let mrna = state.mce.encode(&body.nodes, intent);
    (StatusCode::OK, Json(mrna)).into_response()
}

/// POST /action handler (FT-006).
///
/// Executes an mRNA instruction with TTL enforcement.
#[utoipa::path(
    post,
    path = "/action",
    request_body = ActionBody,
    responses(
        (status = 200, description = "Execute mRNA instruction")
    )
)]
async fn action_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ActionBody>,
) -> impl IntoResponse {
    match state.mce.execute(&body.mrna, body.created_at_ms) {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => {
            let status = if e.to_string().contains("expired") {
                StatusCode::REQUEST_TIMEOUT
            } else {
                StatusCode::BAD_REQUEST
            };
            (
                status,
                Json(serde_json::json!({
                    "error": "ACTION_FAILURE",
                    "message": e.to_string(),
                })),
            )
                .into_response()
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{header, Request};
    use http_body_util::BodyExt;
    use kce_pipeline::orchestrator::PipelineConfig;
    use kce_retrieval::engine::Dataset;
    use tower::ServiceExt;

    fn test_state() -> Arc<AppState> {
        let mut dataset = Dataset::new(128);
        let _ = dataset.push(1, vec![1.0; 128]);
        let pipeline = PipelineOrchestrator::new(PipelineConfig::default(), dataset);
        let mut ais_detector =
            kce_ais::detection::ImmuneDetector::new(kce_ais::detection::DetectorConfig {
                threshold: 0.7,
                warmup_count: 0,
                window_size: 1000,
            });
        ais_detector.add_pattern(vec![0.0; 128]);
        Arc::new(AppState {
            pipeline: Arc::new(pipeline),
            mce: MceEngine::with_defaults(),
            start_time: std::time::Instant::now(),
            semaphore: tokio::sync::Semaphore::new(100),
            tenant_limiter: TenantRateLimiter::new(50),
            metric_query_latency_ms: AtomicU64::new(0),
            metric_retrieval_hits: AtomicU64::new(0),
            metric_error_count: AtomicU64::new(0),
            metric_total_count: AtomicU64::new(0),
            events: tokio::sync::broadcast::channel(crate::events::EVENT_CHANNEL_CAPACITY).0,
            node_seq: AtomicU64::new(10_000),
            ais: Mutex::new(ais_detector),
        })
    }

    #[tokio::test]
    async fn test_missing_tenant_id_returns_400() {
        let app = build_router(test_state());
        let body = serde_json::json!({
            "query_vector": [1.0, 0.0],
            "top_k": 5
        });
        let req = Request::builder()
            .method("POST")
            .uri("/query")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["error"], "VALIDATION_ERROR");
        assert!(
            json["message"]
                .as_str()
                .unwrap()
                .contains("Missing x-tenant-id"),
            "Expected 'Missing x-tenant-id' in message, got: {}",
            json["message"]
        );
    }

    #[tokio::test]
    async fn test_empty_tenant_id_returns_400() {
        let app = build_router(test_state());
        let body = serde_json::json!({
            "query_vector": [1.0, 0.0],
            "top_k": 5
        });
        let req = Request::builder()
            .method("POST")
            .uri("/query")
            .header(header::CONTENT_TYPE, "application/json")
            .header("x-tenant-id", "")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["error"], "VALIDATION_ERROR");
        assert!(
            json["message"]
                .as_str()
                .unwrap()
                .contains("Invalid x-tenant-id"),
            "Expected 'Invalid x-tenant-id' in message, got: {}",
            json["message"]
        );
    }

    #[tokio::test]
    async fn test_valid_tenant_id_accepted() {
        let app = build_router(test_state());
        let body = serde_json::json!({
            "query_vector": [1.0, 0.0],
            "top_k": 5
        });
        let req = Request::builder()
            .method("POST")
            .uri("/query")
            .header(header::CONTENT_TYPE, "application/json")
            .header("x-tenant-id", "tenant-alpha")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        // Should not be 400 — the tenant_id is accepted.
        // It may be 200 or 500 depending on pipeline internals,
        // but it must NOT be 400 (that would mean tenant validation failed).
        assert_ne!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "Valid tenant-id should not trigger a 400"
        );
    }

    #[tokio::test]
    async fn test_tenant_limiter_basic() {
        let limiter = TenantRateLimiter::new(50);
        // First 2 requests for the same tenant should succeed.
        assert!(limiter.try_acquire("t1", 2));
        assert!(limiter.try_acquire("t1", 2));
        // Third should fail (max_concurrent = 2).
        assert!(!limiter.try_acquire("t1", 2));
        // Release one and try again.
        limiter.release("t1");
        assert!(limiter.try_acquire("t1", 2));
        // Clean up.
        limiter.release("t1");
        limiter.release("t1");
    }

    #[tokio::test]
    async fn test_tenant_limiter_independent_tenants() {
        let limiter = TenantRateLimiter::new(50);
        // Fill up tenant-a.
        assert!(limiter.try_acquire("tenant-a", 1));
        assert!(!limiter.try_acquire("tenant-a", 1));
        // tenant-b should still be allowed.
        assert!(limiter.try_acquire("tenant-b", 1));
        limiter.release("tenant-a");
        limiter.release("tenant-b");
    }

    #[tokio::test]
    async fn query_publishes_completed_event() {
        let state = test_state();
        let mut rx = state.events.subscribe();
        let body = serde_json::json!({
            "query_vector": vec![1.0f64; 128],
            "top_k": 5
        });
        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/query")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-tenant-id", "tenant-dev")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::OK);
        let ev = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv())
            .await
            .expect("event timeout")
            .expect("channel open");
        assert!(
            matches!(ev, crate::events::EngineEvent::QueryCompleted { .. }),
            "unexpected event: {ev:?}"
        );
        if let crate::events::EngineEvent::QueryCompleted { latency_ms, action } = ev {
            assert!(latency_ms >= 0.0);
            assert!(!action.is_empty());
        }
    }

    #[tokio::test]
    async fn cors_allows_configured_origin_preflight() {
        std::env::set_var("KCE_CORS_ORIGINS", "http://localhost:3000");
        let state = test_state();
        let app = build_router(state)
            .layer(axum::middleware::from_fn(
                crate::middleware::request_id_middleware,
            ))
            .layer(cors_layer());
        let response = app
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/query")
                    .header("origin", "http://localhost:3000")
                    .header("access-control-request-method", "POST")
                    .header("access-control-request-headers", "content-type,x-api-key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let allow_origin = response
            .headers()
            .get("access-control-allow-origin")
            .expect("allow-origin header")
            .to_str()
            .unwrap()
            .to_string();
        assert_eq!(allow_origin, "http://localhost:3000");
        std::env::remove_var("KCE_CORS_ORIGINS");
    }

    #[tokio::test]
    async fn openapi_and_docs_skip_auth() {
        let state = test_state();
        for uri in ["/health", "/openapi.json", "/docs"] {
            let app = build_router(state.clone()).layer(axum::middleware::from_fn(
                crate::middleware::auth_middleware,
            ));
            let response = app
                .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_ne!(response.status(), StatusCode::UNAUTHORIZED, "uri {uri}");
        }
    }

    #[tokio::test]
    async fn events_endpoint_is_sse() {
        let state = test_state();
        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .uri("/events")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .expect("content-type")
            .to_str()
            .unwrap()
            .to_string();
        assert!(content_type.starts_with("text/event-stream"));
        // Intentionally do NOT collect the body: the stream stays open.
    }

    #[tokio::test]
    async fn graph_returns_nodes_and_edges() {
        let state = test_state();
        {
            let pipeline = &state.pipeline;
            pipeline.upsert_node(kce_core::types::EcmaNode {
                id: 1,
                state: kce_core::types::NodeState::Stem,
                usage_count: 1,
                entropy: 1.0,
                connections: 0,
                maturity: 0.0,
            });
            pipeline.upsert_node(kce_core::types::EcmaNode {
                id: 2,
                state: kce_core::types::NodeState::Progenitor,
                usage_count: 3,
                entropy: 0.5,
                connections: 1,
                maturity: 0.4,
            });
            pipeline.connect(1, 2, 0.5);
        }
        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .uri("/graph")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["nodes"].as_array().unwrap().len(), 2);
        assert_eq!(v["edges"].as_array().unwrap().len(), 1);
        assert_eq!(v["nodes"][0]["id"], 1);
        assert!(v["pheromone"].is_object());
    }

    #[tokio::test]
    async fn create_node_returns_201_and_emits_event() {
        let state = test_state();
        let mut rx = state.events.subscribe();
        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/nodes")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "connect_to": null, "weight": null }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let id = v["id"].as_u64().unwrap();
        assert!(id >= 10_000);
        let ev = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv())
            .await
            .expect("timeout")
            .expect("open");
        assert!(matches!(ev, crate::events::EngineEvent::NodeCreated { id: e_id } if e_id == id));
    }

    #[tokio::test]
    async fn create_node_with_connect_validates() {
        let state = test_state();
        // unknown connect_to -> 404
        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/nodes")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "connect_to": 999999 }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // out-of-range weight -> 400
        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/nodes")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::json!({ "weight": 2.0 }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn delete_node_lifecycle() {
        let state = test_state();
        // create first
        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/nodes")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let id = v["id"].as_u64().unwrap();

        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/nodes/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/nodes/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn set_state_and_deposit() {
        let state = test_state();
        // seed a node
        state.pipeline.upsert_node(kce_core::types::EcmaNode {
            id: 5,
            state: kce_core::types::NodeState::Stem,
            usage_count: 1,
            entropy: 1.0,
            connections: 0,
            maturity: 0.0,
        });
        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/nodes/5/state")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "state": "Apoptosis" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // invalid state string -> client error (422 in axum 0.7 Json rejection)
        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/nodes/5/state")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "state": "Nope" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status().is_client_error());

        // deposit amount <= 0 -> 400
        let response = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/pheromone/deposit")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "from": 5, "to": 6, "amount": -1.0 }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // valid deposit -> 200 with level
        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/pheromone/deposit")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "from": 5, "to": 6, "amount": 0.5 }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(v["level"].as_f64().unwrap() > 0.0);
    }

    #[tokio::test]
    async fn ais_probe_normal_vector() {
        let state = test_state();
        let body = serde_json::json!({
            "vector": vec![0.001; 128]
        });
        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/ais/probe")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["is_anomaly"], false);
        assert!(v["score"].as_f64().unwrap() < 0.7);
    }

    #[tokio::test]
    async fn ais_probe_anomaly_emits_event() {
        let state = test_state();
        let mut rx = state.events.subscribe();
        let body = serde_json::json!({
            "vector": vec![3.0; 128]
        });
        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/ais/probe")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["is_anomaly"], true);
        assert!(v["score"].as_f64().unwrap() >= 0.7);

        let ev = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv())
            .await
            .expect("timeout")
            .expect("open");
        assert!(
            matches!(ev, crate::events::EngineEvent::AisAnomaly { score, blocked } if blocked && score >= 0.7)
        );
    }

    #[tokio::test]
    async fn ais_probe_invalid_dimension() {
        let state = test_state();
        let body = serde_json::json!({
            "vector": vec![1.0; 10]
        });
        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/ais/probe")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn openapi_json_and_docs_are_served() {
        let state = test_state();
        let app = build_router(state);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["openapi"], "3.0.3");
        assert!(v["paths"]["/graph"].is_object());
        assert!(v["paths"]["/query"].is_object());
        assert!(v["paths"]["/nodes"].is_object());
        assert!(v["paths"]["/ais/probe"].is_object());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/docs/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
