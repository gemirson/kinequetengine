//! KineContext Engine — REST API Gateway.
//!
//! Built on Axum, provides REST endpoints with OpenAPI documentation,
//! API key authentication, request ID propagation, and CORS support.
//!
//! # Endpoints
//!
//! - `POST /query` — Execute a retrieval query.
//! - `POST /encode` — Encode context into mRNA.
//! - `POST /action` — Execute an mRNA instruction.
//! - `GET /health` — System health check.
//! - `GET /metrics` — Prometheus metrics.

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]

pub mod middleware;
pub mod routes;
