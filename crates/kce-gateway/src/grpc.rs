//! gRPC implementation for the Hematoencephalic Gateway.

use std::pin::Pin;
use tokio_stream::Stream;
use tonic::{Request, Response, Status};

use crate::proto::kce_gateway_server::KceGateway;
use crate::proto::{SearchRequest, SearchResponse, TelemetryEvent, TelemetryRequest};

/// gRPC Service implementation.
pub struct KceGatewayService;

#[tonic::async_trait]
impl KceGateway for KceGatewayService {
    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let _req = request.into_inner();
        
        // Simplified search mock for FT-031 MVP.
        // In full implementation, this routes to a shard node via FT-028.
        Ok(Response::new(SearchResponse {
            results: vec![],
            latency_ms: 0.1,
        }))
    }

    type StreamTelemetryStream = Pin<Box<dyn Stream<Item = Result<TelemetryEvent, Status>> + Send>>;

    async fn stream_telemetry(
        &self,
        _request: Request<TelemetryRequest>,
    ) -> Result<Response<Self::StreamTelemetryStream>, Status> {
        // Simplified stream for FT-031 MVP.
        let output = tokio_stream::iter(vec![]);
        Ok(Response::new(Box::pin(output) as Self::StreamTelemetryStream))
    }
}
