use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

pub type RateLimitStore = Arc<RwLock<HashMap<String, (Instant, u32)>>>;

pub async fn rate_limit(
    State(store): State<RateLimitStore>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = addr.ip().to_string();
    let path = request.uri().path().to_string();
    let key = format!("{}:{}", ip, path);

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
