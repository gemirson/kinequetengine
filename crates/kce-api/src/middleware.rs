//! API middleware: authentication, request ID propagation.

use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;

/// API key header name.
pub const API_KEY_HEADER: &str = "x-api-key";
/// Request ID header name.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Authentication middleware.
///
/// Checks for a valid API key in the `x-api-key` header.
/// Skips authentication for the `/health` endpoint.
/// Validates against the `API_KEY` environment variable when set.
pub async fn auth_middleware(
    headers: HeaderMap,
    request: axum::extract::Request,
    next: Next,
) -> impl IntoResponse {
    let path = request.uri().path();
    if path == "/health" {
        return next.run(request).await;
    }

    let expected_key = std::env::var("API_KEY").ok();

    match headers.get(API_KEY_HEADER) {
        Some(key) if !key.is_empty() => {
            if let Some(expected) = expected_key {
                match key.to_str() {
                    Ok(provided) if provided == expected => next.run(request).await,
                    _ => (
                        StatusCode::UNAUTHORIZED,
                        axum::Json(serde_json::json!({
                            "error": "UNAUTHORIZED",
                            "message": "Invalid x-api-key header",
                            "code": 401
                        })),
                    )
                        .into_response(),
                }
            } else {
                // No API_KEY env var set — accept any non-empty key (dev mode)
                next.run(request).await
            }
        }
        _ => (
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({
                "error": "UNAUTHORIZED",
                "message": "Missing or invalid x-api-key header",
                "code": 401
            })),
        )
            .into_response(),
    }
}

/// Request ID injection middleware.
///
/// If the request doesn't have an `x-request-id` header, generates one.
pub async fn request_id_middleware(
    mut request: axum::extract::Request,
    next: Next,
) -> impl IntoResponse {
    if !request.headers().contains_key(REQUEST_ID_HEADER) {
        let id = format!(
            "req_{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        request.headers_mut().insert(
            REQUEST_ID_HEADER,
            id.parse()
                .unwrap_or(axum::http::HeaderValue::from_static("req_unknown")),
        );
    }
    next.run(request).await
}
