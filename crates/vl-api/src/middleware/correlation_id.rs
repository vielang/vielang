//! Correlation-ID middleware — records the X-Request-ID into the current
//! tracing span so every log line emitted during the request carries the ID.
//!
//! Runs after `SetRequestIdLayer` (which generates the UUID) and after
//! `TraceLayer` (which opens a span), so both the ID and the span exist.

use axum::{extract::Request, middleware::Next, response::Response};
use tower_http::request_id::RequestId;

pub async fn correlation_id_middleware(request: Request, next: Next) -> Response {
    // tower_http's SetRequestIdLayer stores the generated UUID in extensions
    // under the RequestId key.
    if let Some(request_id) = request.extensions().get::<RequestId>() {
        if let Ok(id_str) = request_id.header_value().to_str() {
            tracing::Span::current().record("request_id", id_str);
        }
    }
    next.run(request).await
}
