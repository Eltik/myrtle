pub mod keys;
pub mod store;

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::{Mutex, watch};

use crate::app::error::ApiError;
use crate::app::state::AppState;
use keys::CacheKey;

/// A pre-serialized JSON body paired with its strong `ETag`. The `ETag` is computed
/// exactly once - when the body enters the cache - so cache hits and
/// `If-None-Match` revalidations never re-hash the (potentially multi-MB) body.
#[derive(Clone, Serialize, Deserialize)]
pub struct CachedJson {
    pub body: String,
    pub etag: String,
}

impl CachedJson {
    fn new(body: String) -> Self {
        let etag = format!("\"{}\"", hex::encode(Sha256::digest(body.as_bytes())));
        Self { body, etag }
    }
}

/// Fetch a pre-serialized JSON body (+ its precomputed `ETag`) from the cache, or
/// build it with `build` on a miss and store both together. `build` runs only on a
/// cache miss and is the sole place the body is serialized or the `ETag` is hashed.
/// The outcome a detached build broadcasts to everyone waiting on its key.
type Outcome = Option<Arc<Result<CachedJson, ApiError>>>;

/// Detached builds in flight, by rendered cache key: a second caller for the
/// same key joins the running build instead of starting another.
#[derive(Default)]
pub struct Inflight(Mutex<HashMap<String, watch::Receiver<Outcome>>>);

/// Like [`cached_json`], for a build that must not die with its request.
///
/// The build runs as its own task: it writes the cache and then broadcasts
/// its outcome, so a caller whose handler future is dropped - the client
/// went away, or the handler timeout fired - loses nothing but its own
/// response; the next request finds the cache filled. Callers arriving while
/// the build runs join it (single flight). Without this, moving the base
/// search to the blocking pool made the 30 s handler timeout discard a 40 s
/// debug-build search every time, and every retry recomputed it.
pub async fn cached_json_detached<F, Fut>(
    state: &AppState,
    key: &CacheKey<'_>,
    build: F,
) -> Result<CachedJson, ApiError>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<String, ApiError>> + Send + 'static,
{
    if let Some(cached) = state.cache.get::<CachedJson>(key).await {
        return Ok(cached);
    }
    let key_string = key.to_key_string();
    let ttl = key.ttl();
    // One lock covers the check and the registration, so two callers racing
    // on an empty key cannot both start a build; the guard ends with the
    // block, before the task is spawned.
    let (rx, started) = {
        let mut inflight = state.inflight.0.lock().await;
        let entry = if let Some(rx) = inflight.get(&key_string) {
            (rx.clone(), None)
        } else {
            let (tx, rx) = watch::channel(None);
            inflight.insert(key_string.clone(), rx.clone());
            (rx, Some(tx))
        };
        drop(inflight);
        entry
    };
    if let Some(tx) = started {
        let state = state.clone();
        tokio::spawn(async move {
            let result = build().await.map(CachedJson::new);
            if let Ok(cached) = &result
                && let Ok(json) = serde_json::to_string(cached)
            {
                state.cache.set_rendered(&key_string, ttl, json).await;
            }
            // Cache first, then drop the registration: a caller arriving in
            // between finds the entry, never a gap.
            state.inflight.0.lock().await.remove(&key_string);
            let _ = tx.send(Some(Arc::new(result)));
        });
    }
    join(rx).await
}

async fn join(mut rx: watch::Receiver<Outcome>) -> Result<CachedJson, ApiError> {
    loop {
        let current = rx.borrow().clone();
        if let Some(result) = current {
            return match result.as_ref() {
                Ok(cached) => Ok(cached.clone()),
                Err(e) => Err(e.shared_copy()),
            };
        }
        if rx.changed().await.is_err() {
            // The build task is gone without an outcome (a panic).
            return Err(ApiError::ServiceUnavailable);
        }
    }
}

pub async fn cached_json<F, Fut>(
    state: &AppState,
    key: &CacheKey<'_>,
    build: F,
) -> Result<CachedJson, ApiError>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<String, ApiError>>,
{
    if let Some(cached) = state.cache.get::<CachedJson>(key).await {
        return Ok(cached);
    }
    let cached = CachedJson::new(build().await?);
    state.cache.set(key, &cached).await;
    Ok(cached)
}
