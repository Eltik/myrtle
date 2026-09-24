//! One slot per server, and the SINGLE FLIGHT that guards what is behind it.
//!
//! What this module OWNS is the rule that a derived value is built at most once
//! per game-data load per server. Two things are cached that way, the library
//! index here and the community aggregate in
//! [`crate::app::services::story_community`], and they held two copies of the
//! same idiom until [`ServerCache`] took it. The archive is NOT a third: it is
//! a field of [`StoryIndexCache`] and rides the index's own slot.
//!
//! The check and the build sit inside ONE lock, because the index build is
//! 6,332 ms and every caller that arrives inside it would otherwise miss and
//! start its own. Measured on the 2026-09-24 boot, before the lock existed:
//! `spawn_warm` logged "story index warmed" for EN and the first
//! `GET /api/story/index` logged "story index built `build_ms=6332`" a second
//! later, and `/metrics` counted
//! `myrtle_cpu_task_total{kind="story_index",outcome="started"} 3` with
//! `duration_seconds_sum 14.580557` for ONE game-data load. The warm and the
//! request did not key on different `GameData` arcs: they overlapped, and
//! nothing made the second wait for the first. Measured offline with 8
//! concurrent readers on one load: 7 builds and ONE 503 before (each reader
//! takes its own `cpu::run` permit and the eighth is shed at the 2.5 s
//! admission wait), 1 build and no refusal after.

use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::Instant;

use super::dto::StoryIndex;
use super::index::{StoryIndexCache, build_index};
use crate::app::{cpu, error::ApiError, state::AppState};
use crate::core::gamedata::types::GameData;
use crate::core::hypergryph::constants::Server;

/// What one server's slot holds: a `Weak` to the `GameData` the value was built
/// from, and the value.
///
/// A hot reload stores a new `Arc`; the `Weak` then either fails to upgrade or
/// upgrades to a different allocation, so the entry is rebuilt. A raw pointer
/// would not do: after two reloads with no request between them the freed
/// allocation can be reused at the same address and a stale value would answer.
type Entry<T> = Option<(Weak<GameData>, Arc<T>)>;

/// The slot's own lock is the single flight. It is a `tokio::sync::Mutex`
/// because it is held across the build's await, and it is per-server, so a CN
/// build does not hold an EN reader.
type Slot<T> = Arc<tokio::sync::Mutex<Entry<T>>>;

/// A per-server, per-game-data cache of one derived value.
///
/// Declared as a `static`, which is why `new` is `const`. The outer lock only
/// hands out a server's slot and is never held across an await.
#[derive(Debug)]
pub struct ServerCache<T> {
    slots: OnceLock<Mutex<HashMap<Server, Slot<T>>>>,
    builds: AtomicU64,
}

impl<T> Default for ServerCache<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ServerCache<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            slots: OnceLock::new(),
            builds: AtomicU64::new(0),
        }
    }

    /// How many times the value has actually been BUILT in this process.
    ///
    /// Exported so a test can assert the single flight: with a warm and N
    /// concurrent readers on one game-data load this must move by exactly one.
    #[must_use]
    pub fn builds(&self) -> u64 {
        self.builds.load(Ordering::Relaxed)
    }

    /// The cached value for `server`, building it under the slot's lock when
    /// the slot is empty, was built from a different `GameData`, or `fresh`
    /// says the value has aged out.
    ///
    /// `gd` is dropped before `build` runs, so a build in flight never holds
    /// the game data alive on the cache's behalf; a builder that needs it keeps
    /// its own clone.
    pub async fn get_or_build<F, Fut>(
        &self,
        server: Server,
        gd: Arc<GameData>,
        fresh: impl FnOnce(&T) -> bool,
        build: F,
    ) -> Result<Arc<T>, ApiError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Arc<T>, ApiError>>,
    {
        let slot = {
            let slots = self.slots.get_or_init(|| Mutex::new(HashMap::new()));
            let mut slots = slots
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            Arc::clone(slots.entry(server).or_default())
        };
        let mut held = slot.lock().await;
        if let Some((seen, hit)) = held.as_ref()
            && seen.upgrade().is_some_and(|old| Arc::ptr_eq(&old, &gd))
            && fresh(hit)
        {
            return Ok(Arc::clone(hit));
        }
        let key = Arc::downgrade(&gd);
        drop(gd);
        let built = build().await?;
        self.builds.fetch_add(1, Ordering::Relaxed);
        *held = Some((key, Arc::clone(&built)));
        // The guard is the single flight: it must live across `build`, and it is
        // released here, before the value is handed back, not at scope end.
        drop(held);
        Ok(built)
    }
}

static INDEX: ServerCache<StoryIndexCache> = ServerCache::new();

/// How many times the story index has actually been BUILT in this process.
#[must_use]
pub fn index_builds() -> u64 {
    INDEX.builds()
}

/// The cached index for a server, built on the first call after each game data
/// load. Keyed by the `GameData` allocation, so a hot reload gets a fresh build
/// and the stale one is dropped.
///
/// Public because the account refresh needs `by_txt` to turn the game's own
/// played-script paths into story ids, and cloning the whole `StoryIndex`
/// through [`get_story_index`] to reach one map would copy 1,887 entries per
/// refresh for nothing.
pub async fn cached_index(
    state: &AppState,
    server: Server,
) -> Result<Arc<StoryIndexCache>, ApiError> {
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let gd = server_data.game_data.load_full();
    let assets = server_data.asset_index.load_full();
    let assets_dir = std::path::PathBuf::from(&server_data.assets_dir);
    let build_gd = Arc::clone(&gd);
    // An index never ages out on its own: only a new `GameData` retires one.
    INDEX
        .get_or_build(
            server,
            gd,
            |_| true,
            || async move {
                let built = cpu::run("story_index", move || {
                    Arc::new(build_index(&build_gd, &assets, &assets_dir))
                })
                .await?;
                tracing::info!(
                    server = ?server,
                    groups = built.index.groups.len(),
                    records = built.index.records.len(),
                    stories = built.lookup.len(),
                    words = built.index.totals.words,
                    build_ms = built.build_ms,
                    probe_ms = built.probe_ms,
                    story_assets_ms = built.story_assets_ms,
                    "story index built"
                );
                Ok(built)
            },
        )
        .await
}

/// Build the library index for `server` OFF the request path: at startup and
/// after every hot reload, so the first reader does not pay the parse. The
/// lazy path in [`cached_index`] stays the fallback, so a request that
/// arrives before the warm lands simply builds it itself.
pub fn spawn_warm(state: AppState, server: Server) {
    tokio::spawn(async move {
        let started = Instant::now();
        match cached_index(&state, server).await {
            Ok(built) => tracing::info!(
                server = ?server,
                warm_ms = started.elapsed().as_millis(),
                build_ms = built.build_ms,
                story_assets_ms = built.story_assets_ms,
                stories = built.lookup.len(),
                "story index warmed"
            ),
            Err(e) => tracing::warn!(
                server = ?server,
                error = %e,
                "story index warm failed; the first reader will build it"
            ),
        }
    });
}

pub async fn get_story_index(state: &AppState, server: Server) -> Result<StoryIndex, ApiError> {
    Ok(cached_index(state, server).await?.index.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A build that records that it ran and answers with the value it was told
    /// to, so a test can tell a hit from a build without a real index.
    async fn built(n: u32) -> Result<Arc<u32>, ApiError> {
        Ok(Arc::new(n))
    }

    #[tokio::test]
    async fn a_slot_answers_until_the_game_data_or_the_freshness_rule_retires_it() {
        static CACHE: ServerCache<u32> = ServerCache::new();
        let gd = Arc::new(GameData::default());

        let first = CACHE
            .get_or_build(Server::EN, Arc::clone(&gd), |_| true, || built(1))
            .await
            .unwrap();
        assert_eq!(*first, 1);
        assert_eq!(CACHE.builds(), 1);

        // Same game data and still fresh: the slot answers and nothing builds.
        let hit = CACHE
            .get_or_build(Server::EN, Arc::clone(&gd), |_| true, || built(2))
            .await
            .unwrap();
        assert_eq!(*hit, 1);
        assert_eq!(CACHE.builds(), 1);

        // Same game data, aged out: rebuilt in place.
        let aged = CACHE
            .get_or_build(Server::EN, Arc::clone(&gd), |_| false, || built(3))
            .await
            .unwrap();
        assert_eq!(*aged, 3);
        assert_eq!(CACHE.builds(), 2);

        // A hot reload is a different allocation, so the `Weak` check misses.
        let reloaded = CACHE
            .get_or_build(
                Server::EN,
                Arc::new(GameData::default()),
                |_| true,
                || built(4),
            )
            .await
            .unwrap();
        assert_eq!(*reloaded, 4);
        assert_eq!(CACHE.builds(), 3);

        // Another server has its own slot and does not see EN's value.
        let cn = CACHE
            .get_or_build(Server::CN, Arc::clone(&gd), |_| true, || built(5))
            .await
            .unwrap();
        assert_eq!(*cn, 5);
        assert_eq!(CACHE.builds(), 4);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn eight_concurrent_callers_build_once() {
        static CACHE: ServerCache<u32> = ServerCache::new();
        static RUNS: AtomicU64 = AtomicU64::new(0);
        let gd = Arc::new(GameData::default());

        let mut tasks = Vec::new();
        for _ in 0..8 {
            let gd = Arc::clone(&gd);
            tasks.push(tokio::spawn(async move {
                CACHE
                    .get_or_build(
                        Server::EN,
                        gd,
                        |_| true,
                        || async {
                            // Long enough that every other caller is inside
                            // `get_or_build` while this one builds.
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                            Ok(Arc::new(RUNS.fetch_add(1, Ordering::Relaxed) as u32))
                        },
                    )
                    .await
                    .unwrap()
            }));
        }
        for task in tasks {
            assert_eq!(*task.await.unwrap(), 0);
        }
        assert_eq!(RUNS.load(Ordering::Relaxed), 1);
        assert_eq!(CACHE.builds(), 1);
    }
}
