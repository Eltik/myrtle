use axum::{
    body::Body,
    http::{HeaderMap, StatusCode, header},
    response::Response,
};
use md5::{Digest, Md5};
use redis::AsyncCommands;
use std::future::Future;

use super::compression::compress_json;

#[allow(dead_code)]
pub struct CachedResponse {
    pub body: Vec<u8>,
    pub is_compressed: bool,
    pub etag: String,
}

pub async fn cached_handler<T, F, Fut>(
    redis: &mut redis::aio::MultiplexedConnection,
    cache_key: &str,
    ttl_secs: u64,
    headers: &HeaderMap,
    fetch_fn: F,
) -> Result<Response, StatusCode>
where
    T: serde::Serialize,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Option<T>>,
{
    // Check If-None-Match (ETag)
    let etag_key = format!("{cache_key}:etag");
    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH)
        && let Ok(stored_etag) = redis.get::<&str, String>(&etag_key).await
        && if_none_match.to_str().ok() == Some(&stored_etag)
    {
        return Ok(Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .header(header::CACHE_CONTROL, "public, max-age=3600")
            .body(Body::empty())
            .unwrap());
    }

    // Check cache for pre-compressed data
    let compressed_key = format!("{cache_key}:gz");
    if let Ok(cached) = redis.get::<&str, Vec<u8>>(&compressed_key).await
        && !cached.is_empty()
    {
        let etag: String = redis
            .get::<&str, String>(&etag_key)
            .await
            .unwrap_or_default();
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::CONTENT_ENCODING, "gzip")
            .header(header::CACHE_CONTROL, "public, max-age=3600")
            .header(header::ETAG, etag)
            .body(Body::from(cached))
            .unwrap());
    }

    // Cache miss - fetch data
    let data = fetch_fn().await.ok_or(StatusCode::NOT_FOUND)?;

    // Serialize and compress
    let json = serde_json::to_vec(&data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let compressed = compress_json(&json);
    let hash = Md5::digest(&json);
    let etag = format!(
        "\"{}\"",
        hash.iter().map(|b| format!("{b:02x}")).collect::<String>()
    );

    // Store in cache
    let _: () = redis
        .set_ex(&compressed_key, &compressed, ttl_secs)
        .await
        .unwrap_or_default();
    let _: () = redis
        .set_ex(&etag_key, &etag, ttl_secs)
        .await
        .unwrap_or_default();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CONTENT_ENCODING, "gzip")
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .header(header::ETAG, etag)
        .body(Body::from(compressed))
        .unwrap())
}
