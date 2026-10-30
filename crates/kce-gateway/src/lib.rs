//! FT-031 — Hematoencephalic Gateway.
//!
//! Provides gRPC-Web and WebSockets interfaces for external access.

pub mod grpc;
pub mod websocket;

pub use grpc::KceGatewayService;
pub use websocket::start_websocket_server;

/// Generated gRPC code.
pub mod proto {
    tonic::include_proto!("kce.gateway");
}
