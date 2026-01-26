use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use moka::future::Cache;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use subtle::ConstantTimeEq;

use crate::app::state::AppState;

/// Rate limit entry: (request count, window start timestamp in seconds)
#[derive(Clone, Copy)]
pub struct RateLimitEntry {
    count: u32,
    window_start: u64,
}

/// TTL-based rate limit store using moka cache
/// Entries automatically expire after ENTRY_TTL, preventing unbounded memory growth
pub type RateLimitStore = Arc<Cache<String, RateLimitEntry>>;

/// Maximum number of entries in the rate limit cache
const MAX_ENTRIES: u64 = 100_000;

/// Time-to-live for cache entries (2 minutes)
/// Entries are evicted after this time of inactivity
const ENTRY_TTL: Duration = Duration::from_secs(120);

/// Time-to-idle for cache entries (2 minutes)
/// Entries are evicted if not accessed within this time
const ENTRY_TTI: Duration = Duration::from_secs(120);

/// Rate limit window duration in seconds
const WINDOW_SECS: u64 = 60;

/// Maximum requests per window
const MAX_REQUESTS: u32 = 100;

const BYPASS_HEADER: &str = "x-internal-service-key";

/// Creates a new rate limit store with TTL-based eviction
pub fn new_rate_limit_store() -> RateLimitStore {
    Arc::new(
        Cache::builder()
            .max_capacity(MAX_ENTRIES)
            .time_to_live(ENTRY_TTL)
            .time_to_idle(ENTRY_TTI)
            .build(),
    )
}

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

/// Get current timestamp in seconds
fn current_timestamp_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
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

    let now = current_timestamp_secs();

    // Get or create entry
    let entry = store.get(&key).await;

    let (new_count, should_reject) = match entry {
        Some(entry) => {
            // Check if we're in the same window
            if now - entry.window_start < WINDOW_SECS {
                // Same window - check count
                if entry.count >= MAX_REQUESTS {
                    (entry.count, true)
                } else {
                    (entry.count + 1, false)
                }
            } else {
                // New window - reset count
                (1, false)
            }
        }
        None => {
            // New entry
            (1, false)
        }
    };

    if should_reject {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Update entry (this also refreshes TTL)
    let new_entry = RateLimitEntry {
        count: new_count,
        window_start: match entry {
            Some(e) if now - e.window_start < WINDOW_SECS => e.window_start,
            _ => now,
        },
    };
    store.insert(key, new_entry).await;

    Ok(next.run(request).await)
}
