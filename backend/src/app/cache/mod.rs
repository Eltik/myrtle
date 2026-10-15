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

/// Pre-serialized JSON body with its strong `ETag`, hashed once on cache entry so
/// hits and `If-None-Match` never re-hash a multi-MB body.
#[derive(Clone, Serialize, Deserialize)]
pub struct CachedJson {
    pub body: String,
    pub etag: String,
}

impl CachedJson {
    fn new(body: String) -> Self {
        let etag = strong_etag(body.as_bytes());
        Self { body, etag }
    }
}

/// A strong `ETag` for a body: its SHA-256, hex, quoted.
#[must_use]
pub fn strong_etag(body: &[u8]) -> String {
    format!("\"{}\"", hex::encode(Sha256::digest(body)))
}

/// Pre-serialized JSON held by a per-game-data cache and answered with an
/// `ETag`. The body is `Bytes`, so a request clones a refcount, never the
/// body: the music index is 1,161,730 bytes and is read on every page open.
///
/// The brotli form is encoded once, here, at the compression layer's quality
/// (6), and served as is to a client that accepts it; the layer passes a body
/// that already names its encoding. Before, the layer re-encoded the music
/// index on every request: 117 to 154 ms of server time for its 110,137 bytes.
#[derive(Clone, Debug)]
pub struct SharedJson {
    pub body: axum::body::Bytes,
    pub etag: String,
    pub br: axum::body::Bytes,
    /// The brotli form's own `ETag`: a different representation of the same body.
    pub br_etag: String,
}

/// Brotli quality, the same as `server.rs`'s `CompressionLayer` (see the
/// measurements there), so a pre-encoded body is byte-for-byte what the layer
/// would have sent.
const BROTLI_QUALITY: u32 = 6;
/// Brotli window, log2: 22 (4 MiB) covers every body this holds.
const BROTLI_LGWIN: u32 = 22;

impl SharedJson {
    /// Hashes and brotli-encodes `body`. CPU-bound: call it inside `cpu::run`.
    #[must_use]
    pub fn new(body: Vec<u8>) -> Self {
        let etag = strong_etag(&body);
        let mut br = Vec::with_capacity(body.len() / 8);
        {
            let mut writer =
                brotli::CompressorWriter::new(&mut br, 4096, BROTLI_QUALITY, BROTLI_LGWIN);
            // Writing into a Vec cannot fail.
            let _ = std::io::Write::write_all(&mut writer, &body);
        }
        let br_etag = format!("{}-br\"", etag.trim_end_matches('"'));
        Self {
            body: axum::body::Bytes::from(body),
            etag,
            br: axum::body::Bytes::from(br),
            br_etag,
        }
    }
}

/// The outcome a detached build broadcasts to everyone waiting on its key.
type Outcome = Option<Arc<Result<CachedJson, ApiError>>>;

/// Detached builds in flight, by rendered cache key: a second caller for the
/// same key joins the running build instead of starting another.
#[derive(Default)]
pub struct Inflight(Mutex<HashMap<String, watch::Receiver<Outcome>>>);

/// Like [`cached_json`], for a build that must not die with its request.
///
/// The build runs as its own task: it fills the cache, then broadcasts. A caller
/// whose handler is dropped (client gone, handler timeout) loses only its own
/// response; the next request hits the cache. Concurrent callers join the build.
/// Before this, the 30 s handler timeout discarded a 40 s debug-build base search
/// every time and each retry recomputed it.
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

/// Cached body + `ETag`, or run `build` on a miss and store both. `build` is the
/// only place the body is serialized and hashed.
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
