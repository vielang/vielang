use std::time::Instant;

use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};

use crate::metrics as m;

/// Axum middleware: record HTTP request count and duration for Prometheus.
///
/// Applied at the router level — runs for every request regardless of auth.
/// Uses `MatchedPath` (route pattern) as label to avoid high cardinality from
/// per-entity URLs like `/api/device/550e8400-e29b...`.
pub async fn track_metrics(matched_path: Option<MatchedPath>, request: Request, next: Next) -> Response {
    let start  = Instant::now();
    let method = request.method().to_string();

    // Prefer the route pattern over the raw URL path to keep label cardinality low
    let path = matched_path
        .as_ref()
        .map(|p| p.as_str())
        .unwrap_or(request.uri().path())
        .to_owned();

    let response = next.run(request).await;

    let elapsed = start.elapsed().as_secs_f64();
    let status  = response.status().as_u16().to_string();

    metrics::counter!(m::HTTP_REQUESTS_TOTAL,
        "method"   => method.clone(),
        "endpoint" => path.clone(),
        "status"   => status
    ).increment(1);

    metrics::histogram!(m::HTTP_REQUEST_DURATION,
        "method"   => method,
        "endpoint" => path
    ).record(elapsed);

    response
}
