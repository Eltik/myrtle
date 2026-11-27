use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use dashmap::DashMap;
use hmac::{Hmac, Mac};
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::watch;

use crate::AppError;

const SWEEP_EVERY: Duration = Duration::from_mins(1);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedJson {
    pub body: Arc<str>,
    pub etag: String,
}

impl CachedJson {
    #[must_use]
    pub fn new(body: String) -> Self {
        let etag = strong_etag(body.as_bytes());
        Self {
            body: body.into(),
            etag,
        }
    }
}

#[must_use]
pub fn strong_etag(body: &[u8]) -> String {
    format!("\"{}\"", hex::encode(Sha256::digest(body)))
}

#[must_use]
pub fn key_hash(bytes: &[u8]) -> String {
    hex::encode(&Sha256::digest(bytes)[..8])
}

#[must_use]
pub fn hmac_hex(key: &[u8], message: &[u8]) -> String {
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(key) else {
        return String::new();
    };
    mac.update(message);
    hex::encode(mac.finalize().into_bytes())
}

type Outcome = Option<Arc<Result<CachedJson, AppError>>>;
type Inflight = Arc<Mutex<HashMap<String, Building>>>;

#[derive(Clone)]
struct Building {
    outcome: watch::Receiver<Outcome>,
    outdated: Arc<AtomicBool>,
}

#[derive(Clone)]
enum Store {
    Redis(ConnectionManager),
    Memory(Arc<DashMap<String, (String, Instant)>>),
}

#[derive(Clone)]
pub struct Cache {
    store: Store,
    inflight: Inflight,
    shared: Arc<DashMap<String, (CachedJson, Instant)>>,
}

impl std::fmt::Debug for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.is_redis() {
            "Cache(redis)"
        } else {
            "Cache(memory)"
        })
    }
}

impl Cache {
    #[must_use]
    pub fn memory() -> Self {
        Self::over(Store::Memory(Arc::new(DashMap::new())))
    }

    pub async fn connect(url: Option<&str>) -> Result<Self, AppError> {
        let Some(url) = url else {
            let cache = Self::memory();
            cache.spawn_sweep();
            return Ok(cache);
        };
        let client = redis::Client::open(url).map_err(AppError::internal)?;
        let manager = ConnectionManager::new(client).await.map_err(|e| {
            tracing::error!(error = %e, "redis connect failed");
            AppError::Unavailable
        })?;
        Ok(Self::over(Store::Redis(manager)))
    }

    fn over(store: Store) -> Self {
        Self {
            store,
            inflight: Inflight::default(),
            shared: Arc::default(),
        }
    }

    #[must_use]
    pub const fn is_redis(&self) -> bool {
        matches!(self.store, Store::Redis(_))
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        match &self.store {
            Store::Redis(conn) => conn
                .clone()
                .get::<_, Option<String>>(key)
                .await
                .inspect_err(|e| tracing::warn!(key, error = %e, "cache read failed"))
                .ok()
                .flatten(),
            Store::Memory(entries) => {
                let (value, expires_at) = entries.get(key)?.value().clone();
                if Instant::now() < expires_at {
                    return Some(value);
                }
                entries.remove_if(key, |_, (_, at)| Instant::now() >= *at);
                None
            }
        }
    }

    pub async fn set(&self, key: &str, value: &str, ttl: Duration) {
        match &self.store {
            Store::Redis(conn) => {
                let mut conn = conn.clone();
                let millis = u64::try_from(ttl.as_millis()).unwrap_or(u64::MAX);
                let stored: Result<(), _> = if millis == 0 {
                    conn.del(key).await
                } else {
                    conn.pset_ex(key, value, millis).await
                };
                if let Err(e) = stored {
                    tracing::warn!(key, error = %e, "cache set failed");
                }
            }
            Store::Memory(entries) => {
                entries.insert(key.to_owned(), (value.to_owned(), Instant::now() + ttl));
            }
        }
    }

    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let json = self.get(key).await?;
        serde_json::from_str(&json)
            .inspect_err(|e| tracing::debug!(key, error = %e, "cache deser failed"))
            .ok()
    }

    pub async fn set_json<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) {
        match serde_json::to_string(value) {
            Ok(json) => self.set(key, &json, ttl).await,
            Err(e) => tracing::debug!(key, error = %e, "cache serialize failed"),
        }
    }

    pub async fn invalidate(&self, key: &str) {
        self.outdate_builds(|building| building == key);
        self.shared.remove(key);
        self.delete(key).await;
    }

    async fn delete(&self, key: &str) {
        match &self.store {
            Store::Redis(conn) => {
                let deleted: Result<(), _> = conn.clone().del(key).await;
                if let Err(e) = deleted {
                    tracing::debug!(key, error = %e, "cache del failed");
                }
            }
            Store::Memory(entries) => {
                entries.remove(key);
            }
        }
    }

    pub async fn invalidate_prefix(&self, prefix: &str) {
        self.outdate_builds(|building| building.starts_with(prefix));
        self.shared.retain(|key, _| !key.starts_with(prefix));
        match &self.store {
            Store::Redis(conn) => {
                let mut conn = conn.clone();
                let pattern = format!("{prefix}*");
                let mut cursor = 0_u64;
                loop {
                    let page: Result<(u64, Vec<String>), _> = redis::cmd("SCAN")
                        .arg(cursor)
                        .arg("MATCH")
                        .arg(&pattern)
                        .arg("COUNT")
                        .arg(100)
                        .query_async(&mut conn)
                        .await;
                    let Ok((next, keys)) = page.inspect_err(
                        |e| tracing::debug!(prefix, error = %e, "cache prefix scan failed"),
                    ) else {
                        break;
                    };
                    if !keys.is_empty() {
                        let _: Result<(), _> = conn.del(keys).await;
                    }
                    if next == 0 {
                        break;
                    }
                    cursor = next;
                }
            }
            Store::Memory(entries) => entries.retain(|key, _| !key.starts_with(prefix)),
        }
    }

    pub async fn ping(&self) -> bool {
        match &self.store {
            Store::Redis(conn) => redis::cmd("PING")
                .query_async::<String>(&mut conn.clone())
                .await
                .is_ok(),
            Store::Memory(_) => true,
        }
    }

    pub fn spawn_sweep(&self) {
        let Store::Memory(entries) = &self.store else {
            return;
        };
        let entries = Arc::downgrade(entries);
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(SWEEP_EVERY);
            loop {
                tick.tick().await;
                let Some(entries) = entries.upgrade() else {
                    break;
                };
                let now = Instant::now();
                entries.retain(|_, (_, expires_at)| now < *expires_at);
            }
        });
    }

    pub async fn json<F, Fut>(
        &self,
        key: &str,
        ttl: Duration,
        build: F,
    ) -> Result<CachedJson, AppError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<String, AppError>>,
    {
        if let Some(cached) = self.get_json::<CachedJson>(key).await {
            return Ok(cached);
        }
        let cached = CachedJson::new(build().await?);
        self.set_json(key, &cached, ttl).await;
        Ok(cached)
    }

    pub async fn json_detached<F, Fut>(
        &self,
        key: &str,
        ttl: Duration,
        build: F,
    ) -> Result<CachedJson, AppError>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<String, AppError>> + Send + 'static,
    {
        if let Some(cached) = self.get_json::<CachedJson>(key).await {
            return Ok(cached);
        }
        self.single_flight(key, ttl, false, build).await
    }

    pub async fn shared<F, Fut>(
        &self,
        key: &str,
        ttl: Duration,
        build: F,
    ) -> Result<CachedJson, AppError>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<String, AppError>> + Send + 'static,
    {
        let hit = self
            .shared
            .get(key)
            .filter(|entry| Instant::now() < entry.1)
            .map(|entry| entry.0.clone());
        if let Some(cached) = hit {
            return Ok(cached);
        }
        self.single_flight(key, ttl, true, build).await
    }

    async fn single_flight<F, Fut>(
        &self,
        key: &str,
        ttl: Duration,
        in_process: bool,
        build: F,
    ) -> Result<CachedJson, AppError>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<String, AppError>> + Send + 'static,
    {
        let (mut rx, started) = {
            let mut inflight = self.inflight.lock().unwrap_or_else(PoisonError::into_inner);
            if let Some(building) = inflight.get(key) {
                (building.outcome.clone(), None)
            } else {
                let (tx, rx) = watch::channel(None);
                let outdated = Arc::new(AtomicBool::new(false));
                let building = Building {
                    outcome: rx.clone(),
                    outdated: Arc::clone(&outdated),
                };
                inflight.insert(key.to_owned(), building);
                drop(inflight);
                (rx, Some((tx, outdated)))
            }
        };
        if let Some((tx, outdated)) = started {
            let cache = self.clone();
            let registration = Registration {
                inflight: Arc::clone(&self.inflight),
                key: key.to_owned(),
            };
            tokio::spawn(async move {
                let result = build().await.map(CachedJson::new);
                match &result {
                    Ok(cached) if in_process => {
                        cache.keep(&registration.key, cached, ttl, &outdated);
                    }
                    Ok(cached) => cache.store(&registration.key, cached, ttl, &outdated).await,
                    Err(_) => {}
                }
                drop(registration);
                let _ = tx.send(Some(Arc::new(result)));
            });
        }
        loop {
            let current = rx.borrow().clone();
            if let Some(result) = current {
                return match result.as_ref() {
                    Ok(cached) => Ok(cached.clone()),
                    Err(e) => Err(e.shared_copy()),
                };
            }
            if rx.changed().await.is_err() {
                return Err(AppError::Unavailable);
            }
        }
    }

    fn outdate_builds(&self, matches: impl Fn(&str) -> bool) {
        let inflight = self.inflight.lock().unwrap_or_else(PoisonError::into_inner);
        for (key, building) in inflight.iter() {
            if matches(key) {
                building.outdated.store(true, Ordering::SeqCst);
            }
        }
    }

    fn keep(&self, key: &str, cached: &CachedJson, ttl: Duration, outdated: &AtomicBool) {
        self.shared
            .insert(key.to_owned(), (cached.clone(), Instant::now() + ttl));
        if outdated.load(Ordering::SeqCst) {
            self.shared
                .remove_if(key, |_, (kept, _)| Arc::ptr_eq(&kept.body, &cached.body));
        }
    }

    async fn store(&self, key: &str, cached: &CachedJson, ttl: Duration, outdated: &AtomicBool) {
        if outdated.load(Ordering::SeqCst) {
            return;
        }
        self.set_json(key, cached, ttl).await;
        if outdated.load(Ordering::SeqCst) {
            self.delete(key).await;
        }
    }

    pub fn clear_shared(&self) {
        self.outdate_builds(|_| true);
        self.shared.clear();
    }

    pub fn drop_shared(&self, prefix: &str) {
        self.outdate_builds(|building| building.starts_with(prefix));
        self.shared.retain(|key, _| !key.starts_with(prefix));
    }
}

struct Registration {
    inflight: Inflight,
    key: String,
}

impl Drop for Registration {
    fn drop(&mut self) {
        self.inflight
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&self.key);
    }
}
