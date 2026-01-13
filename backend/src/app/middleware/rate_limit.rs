use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use subtle::ConstantTimeEq;
use tokio::sync::RwLock;

use crate::app::state::AppState;

pub type RateLimitStore = Arc<RwLock<HashMap<String, (Instant, u32)>>>;

const BYPASS_HEADER: &str = "x-internal-service-key";

/// Check if the request has a valid bypass key using constant-time comparison
fn has_valid_bypass_key(headers: &HeaderMap, expected_key: Option<&str>) -> bool {
    let Some(expected) = expected_key else {
        return false;
    };

    let Some(provided) = headers.get(BYPASS_HEADER).and_then(|v| v.to_str().ok()) else {
        return false;
    };

    // Constant-time comparison to prevent timing attacks
    provided.as_bytes().ct_eq(expected.as_bytes()).into()
}

pub async fn rate_limit(
    State((store, state)): State<(RateLimitStore, AppState)>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check for valid bypass key first - skip rate limiting if valid
    if has_valid_bypass_key(request.headers(), state.internal_service_key.as_deref()) {
        return Ok(next.run(request).await);
    }

    let ip = addr.ip().to_string();
    let path = request.uri().path().to_string();
    let key = format!("{ip}:{path}");

    let mut map = store.write().await;
    let now = Instant::now();

    if let Some((timestamp, count)) = map.get_mut(&key) {
        if now.duration_since(*timestamp) > Duration::from_secs(60) {
            *timestamp = now;
            *count = 1;
        } else if *count > 100 {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        } else {
            *count += 1;
        }
    } else {
        map.insert(key, (now, 1));
    }
    drop(map);

    Ok(next.run(request).await)
}
