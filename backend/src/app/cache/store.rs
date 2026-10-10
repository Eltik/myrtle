use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use serde::{Serialize, de::DeserializeOwned};

use super::keys::CacheKey;
use crate::app::metrics::{CacheOutcome, METRICS};

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
        let key_str = key.to_key_string();
        let class = key_str.split(':').next().unwrap_or("unknown");

        let result = match self {
            Self::Redis(conn) => match conn.clone().get::<_, Option<String>>(&key_str).await {
                Ok(found) => found,
                Err(e) => {
                    tracing::warn!(key = %key_str, error = %e, "cache read failed");
                    METRICS.record_cache(class, CacheOutcome::Error);
                    return None;
                }
            },
            Self::Memory { entries } => {
                let mut expired = false;
                let found = entries.get(&key_str).and_then(|entry| {
                    let (json, expires_at) = entry.value();
                    if Instant::now() >= *expires_at {
                        expired = true;
                        None
                    } else {
                        Some(json.clone())
                    }
                });
                if expired {
                    entries.remove(&key_str);
                }
                found
            }
        };

        METRICS.record_cache(
            class,
            if result.is_some() {
                CacheOutcome::Hit
            } else {
                CacheOutcome::Miss
            },
        );
        result
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
        self.set_rendered(&key.to_key_string(), key.ttl(), value)
            .await;
    }

    /// Stores under a pre-rendered key, for a detached build that outlives its
    /// borrowed `CacheKey` (see `cached_json_detached`).
    pub async fn set_rendered(&self, key_str: &str, ttl: std::time::Duration, value: String) {
        match self {
            Self::Redis(conn) => {
                let _: Result<(), _> = conn
                    .clone()
                    .set_ex(key_str, value, ttl.as_secs())
                    .await
                    .inspect_err(|e| {
                        // A failed write means every later read of this key
                        // misses and rebuilds; that should be visible.
                        tracing::warn!(key = %key_str, error = %e, "cache set failed");
                        let class = key_str.split(':').next().unwrap_or("unknown");
                        METRICS.record_cache(class, CacheOutcome::WriteError);
                    });
            }
            Self::Memory { entries } => {
                let expires_at = Instant::now() + ttl;
                entries.insert(key_str.to_owned(), (value, expires_at));
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

    /// Sweeps expired entries every 60 seconds on a background task. Memory variant only.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// `/api/release/skins` served `devP75Days` 1.2291666666666665 on the call
    /// that computed it and 1.2291666666666663 from the cache: without the
    /// `float_roundtrip` feature serde_json parses some floats one ulp off the
    /// value they were printed from.
    #[tokio::test]
    async fn a_cached_float_reads_back_bit_for_bit() {
        let values = vec![
            1.229_166_666_666_666_5_f64,
            0.1,
            1.0 / 3.0,
            1e-300,
            -2.5e300,
        ];
        let cache = CacheStore::new_memory();
        cache.set(&CacheKey::Stats, &values).await;
        let back: Vec<f64> = cache.get(&CacheKey::Stats).await.expect("cached");
        assert_eq!(back.len(), values.len());
        for (a, b) in values.iter().zip(&back) {
            assert_eq!(a.to_bits(), b.to_bits(), "{a} came back as {b}");
        }
    }

    /// The same over 200,000 arbitrary finite doubles, all exponents included.
    #[test]
    fn every_float_survives_a_json_round_trip() {
        let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
        let mut checked = 0;
        while checked < 200_000 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let value = f64::from_bits(state);
            if !value.is_finite() {
                continue;
            }
            let text = serde_json::to_string(&value).expect("finite floats serialize");
            let back: f64 = serde_json::from_str(&text).expect("and parse");
            assert_eq!(value.to_bits(), back.to_bits(), "{text}");
            checked += 1;
        }
    }
}
