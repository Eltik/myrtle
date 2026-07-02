use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use serde::{Serialize, de::DeserializeOwned};

use super::keys::CacheKey;

#[derive(Clone)]
pub enum CacheStore {
    Redis(ConnectionManager),
    Memory {
        entries: Arc<DashMap<String, (String, Instant)>>,
    },
}

impl CacheStore {
    pub const fn new_redis(conn: ConnectionManager) -> Self {
        Self::Redis(conn)
    }

    pub fn new_memory() -> Self {
        Self::Memory {
            entries: Arc::new(DashMap::new()),
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &CacheKey<'_>) -> Option<T> {
        let json = self.get_raw(key).await?;
        serde_json::from_str(&json)
            .inspect_err(|e| {
                tracing::debug!(key = %key.to_key_string(), error = %e, "cache deser failed");
            })
            .ok()
    }

    pub async fn get_raw(&self, key: &CacheKey<'_>) -> Option<String> {
        match self {
            Self::Redis(conn) => conn
                .clone()
                .get::<_, Option<String>>(key.to_key_string())
                .await
                .ok()
                .flatten(),
            Self::Memory { entries } => {
                let key_str = key.to_key_string();
                let entry = entries.get(&key_str)?;
                let (json, expires_at) = entry.value();
                if Instant::now() >= *expires_at {
                    drop(entry);
                    entries.remove(&key_str);
                    return None;
                }
                Some(json.clone())
            }
        }
    }

    pub async fn set<T: Serialize>(&self, key: &CacheKey<'_>, value: &T) {
        let json = match serde_json::to_string(value) {
            Ok(j) => j,
            Err(e) => {
                tracing::debug!(error = %e, "cache serialize failed");
                return;
            }
        };

        self.set_raw(key, json).await;
    }

    pub async fn set_raw(&self, key: &CacheKey<'_>, value: String) {
        match self {
            Self::Redis(conn) => {
                let _: Result<(), _> = conn
                    .clone()
                    .set_ex(key.to_key_string(), value, key.ttl().as_secs())
                    .await
                    .inspect_err(|e| {
                        tracing::debug!(key = %key.to_key_string(), error = %e, "cache set failed");
                    });
            }
            Self::Memory { entries } => {
                let expires_at = Instant::now() + key.ttl();
                entries.insert(key.to_key_string(), (value, expires_at));
            }
        }
    }

    pub async fn invalidate(&self, key: &CacheKey<'_>) {
        match self {
            Self::Redis(conn) => {
                let _: Result<(), _> = conn.clone().del(key.to_key_string()).await.inspect_err(
                    |e| tracing::debug!(key = %key.to_key_string(), error = %e, "cache del failed"),
                );
            }
            Self::Memory { entries } => {
                entries.remove(&key.to_key_string());
            }
        }
    }

    pub async fn invalidate_by_prefix(&self, prefix: &str) {
        match self {
            Self::Redis(conn) => {
                let pattern = format!("{prefix}*");
                let mut cursor: u64 = 0;
                loop {
                    let result: Result<(u64, Vec<String>), _> = redis::cmd("SCAN")
                        .arg(cursor)
                        .arg("MATCH")
                        .arg(&pattern)
                        .arg("COUNT")
                        .arg(100)
                        .query_async(&mut conn.clone())
                        .await;
                    match result {
                        Ok((next, keys)) => {
                            if !keys.is_empty() {
                                let _: Result<(), _> =
                                    conn.clone().del::<Vec<String>, ()>(keys).await;
                            }
                            if next == 0 {
                                break;
                            }
                            cursor = next;
                        }
                        Err(e) => {
                            tracing::debug!(error = %e, "cache prefix invalidation failed");
                            break;
                        }
                    }
                }
            }
            Self::Memory { entries } => {
                entries.retain(|k, _| !k.starts_with(prefix));
            }
        }
    }

    pub async fn ping(&self) -> bool {
        match self {
            Self::Redis(conn) => redis::cmd("PING")
                .query_async::<String>(&mut conn.clone())
                .await
                .is_ok(),
            Self::Memory { .. } => true,
        }
    }

    pub const fn is_redis(&self) -> bool {
        matches!(self, Self::Redis(_))
    }

    /// Spawn a background task that sweeps expired entries every 60 seconds.
    /// Only meaningful for the Memory variant.
    pub fn spawn_cleanup(&self) {
        if let Self::Memory { entries } = self {
            let entries = Arc::clone(entries);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_mins(1));
                loop {
                    interval.tick().await;
                    entries.retain(|_, (_, expires_at)| Instant::now() < *expires_at);
                }
            });
        }
    }
}
