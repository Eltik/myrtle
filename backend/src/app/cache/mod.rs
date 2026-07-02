pub mod keys;
pub mod store;

use std::future::Future;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
