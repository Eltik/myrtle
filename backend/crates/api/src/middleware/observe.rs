use std::time::{Duration, Instant};

use axum::extract::{MatchedPath, Request};
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use tracing::Instrument;

use crate::error::ErrorCode;
use crate::metrics::METRICS;

const REQUEST_ID: &str = "x-request-id";
const HANDLER_TIMEOUT: Duration = Duration::from_secs(30);

pub fn matched_route(req: &Request) -> Option<&str> {
    req.extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
}

pub async fn observe(req: Request, next: Next) -> Response {
    let started = Instant::now();
    let route = matched_route(&req).unwrap_or("<unmatched>").to_owned();
    let request_id = req
        .headers()
        .get(REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.is_empty() && v.len() <= 128)
        .map_or_else(|| uuid::Uuid::new_v4().to_string(), ToOwned::to_owned);
    let span = tracing::info_span!("http", method = %req.method(), route, request_id);

    let handled = tokio::time::timeout(HANDLER_TIMEOUT, next.run(req).instrument(span.clone()));
    let mut response = handled.await.unwrap_or_else(|_| {
        tracing::error!(parent: &span, timeout_secs = HANDLER_TIMEOUT.as_secs(), "handler timed out");
        ErrorCode::Timeout.respond("request timed out")
    });

    let elapsed = started.elapsed();
    let status = response.status();
    METRICS.record_http(&route, status.as_u16(), elapsed);
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(REQUEST_ID, value);
    }
    let latency_ms = elapsed.as_secs_f64() * 1000.0;
    if status.is_server_error() {
        tracing::warn!(parent: &span, status = status.as_u16(), latency_ms, "request failed");
    } else {
        tracing::debug!(parent: &span, status = status.as_u16(), latency_ms, "request");
    }
    response
}
