use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use serde::{Serialize, de::DeserializeOwned};

use super::keys::CacheKey;

pub async fn get<T: DeserializeOwned>(
    conn: &mut ConnectionManager,
    key: &CacheKey<'_>,
) -> Option<T> {
    let raw: Option<String> = conn.get(key.to_key_string()).await.ok()?;
    let json = raw?;
    serde_json::from_str(&json)
        .inspect_err(
            |e| tracing::debug!(key = %key.to_key_string(), error = %e, "cache deser failed"),
        )
        .ok()
}

pub async fn set<T: Serialize>(conn: &mut ConnectionManager, key: &CacheKey<'_>, value: &T) {
    let json = match serde_json::to_string(value) {
        Ok(j) => j,
        Err(e) => {
            tracing::debug!(error = %e, "cache serialize failed");
            return;
        }
    };

    let _: Result<(), _> = conn
        .set_ex(key.to_key_string(), json, key.ttl().as_secs())
        .await
        .inspect_err(
            |e| tracing::debug!(key = %key.to_key_string(), error = %e, "cache set failed"),
        );
}

pub async fn invalidate(conn: &mut ConnectionManager, key: &CacheKey<'_>) {
    let _: Result<(), _> = conn.del(key.to_key_string()).await.inspect_err(
        |e| tracing::debug!(key = %key.to_key_string(), error = %e, "cache del failed"),
    );
}
