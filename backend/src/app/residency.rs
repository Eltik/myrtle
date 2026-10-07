//! Lazy servers: game data that loads on the first request naming the server
//! and is dropped again after an idle TTL.
//!
//! Every configured server used to be resident for the life of the process.
//! JP and KR exist for their text and see a small share of requests, yet each
//! holds a full `GameData`. `SERVERS_LAZY=jp,kr` enters those servers unloaded
//! at boot; [`ensure_requested_server`] loads one when a request names it (the
//! `/{server}/...` path segment or a `?server=` query), and [`spawn_reaper`]
//! unloads it once nobody has asked for it for `SERVERS_LAZY_IDLE_SECS`.
//!
//! Unset (the default), every server is eager and none of this runs: the boot,
//! the request path and the jobs behave exactly as before.
//!
//! Loads go through the asset watcher's reload lock, so a lazy load never
//! stacks on a hot reload or another lazy load (each build holds a second
//! `GameData`-sized allocation until it is published), and through `cpu::run`,
//! so a load is admitted like any other CPU-bound work and never runs on an
//! async worker. A load is detached from the request that started it: a
//! request that gives up at the wait bound leaves the load running, and the
//! next request finds the server resident.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{MatchedPath, Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::app::error::ApiError;
use crate::app::state::{AppState, ServerData};
use crate::core::hypergryph::constants::Server;

/// Process clock for idle arithmetic, in whole seconds.
static START: LazyLock<Instant> = LazyLock::new(Instant::now);

fn now_secs() -> u64 {
    START.elapsed().as_secs()
}

/// Default idle time before a lazy server is unloaded: 30 minutes. A TRADE,
/// not a derivation: long enough that a reader paging through JP operators
/// does not pay a reload per visit, short enough that a burst of one-off
/// visits does not pin the copy for the day.
const DEFAULT_IDLE_SECS: u64 = 30 * 60;

/// Default bound on how long a request waits for a lazy load before it is
/// answered 503 + `Retry-After` (the load keeps running). Under the 30 s
/// handler timeout with room for the handler itself.
const DEFAULT_WAIT_MS: u64 = 20_000;

/// Ceiling on the wait: the handler timeout is 30 s and the handler still has
/// to run after the load.
const MAX_WAIT_MS: u64 = 25_000;

/// After a failed lazy load, requests are answered 503 without another
/// attempt for this long, so a broken tree is not rebuilt once per request.
const RETRY_AFTER_FAILURE: Duration = Duration::from_secs(60);

/// Lazy-load bookkeeping for one [`ServerData`]. Inert for an eager server.
pub struct Residency {
    /// Entered unloaded at boot, loaded on request, unloaded when idle.
    pub lazy: bool,
    /// [`now_secs`] of the last request that named this server.
    last_used: AtomicU64,
    /// Single flight: true while a load task is running.
    loading: AtomicBool,
    /// Bumped each time a load attempt finishes, so waiters wake.
    done: tokio::sync::watch::Sender<u64>,
    /// The last failed attempt: when, and why (the 503 message).
    failure: Mutex<Option<(Instant, String)>>,
    /// The asset version of an update that arrived while the server was not
    /// resident. The next load records the release ledger under it, so an
    /// operator that debuted in that update keeps its debut link.
    pending_res_version: Mutex<Option<String>>,
}

impl Default for Residency {
    fn default() -> Self {
        Self::new(false)
    }
}

impl Residency {
    #[must_use]
    pub fn new(lazy: bool) -> Self {
        Self {
            lazy,
            last_used: AtomicU64::new(now_secs()),
            loading: AtomicBool::new(false),
            done: tokio::sync::watch::Sender::new(0),
            failure: Mutex::new(None),
            pending_res_version: Mutex::new(None),
        }
    }

    /// Remember an update's asset version for the next load (the latest wins).
    pub(crate) fn defer_res_version(&self, res_version: &str) {
        if let Ok(mut guard) = self.pending_res_version.lock() {
            *guard = Some(res_version.to_owned());
        }
    }

    fn take_res_version(&self) -> Option<String> {
        self.pending_res_version.lock().ok()?.take()
    }

    fn touch(&self) {
        self.last_used.store(now_secs(), Ordering::Relaxed);
    }

    fn idle_secs(&self) -> u64 {
        now_secs().saturating_sub(self.last_used.load(Ordering::Relaxed))
    }

    fn recent_failure(&self) -> Option<String> {
        let guard = self.failure.lock().ok()?;
        guard
            .as_ref()
            .filter(|(at, _)| at.elapsed() < RETRY_AFTER_FAILURE)
            .map(|(_, why)| why.clone())
    }

    fn record(&self, outcome: Result<(), String>) {
        if let Ok(mut guard) = self.failure.lock() {
            *guard = outcome.err().map(|why| (Instant::now(), why));
        }
    }
}

/// `SERVERS_LAZY` and its two knobs, read once at boot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LazyConfig {
    /// Servers entered unloaded. Empty (the default) = every server eager.
    pub servers: Vec<Server>,
    /// Idle time before an unload; `None` never unloads.
    pub idle: Option<Duration>,
    /// How long a request waits for a load before a 503.
    pub wait: Duration,
}

impl LazyConfig {
    #[must_use]
    pub fn from_env() -> Self {
        Self::parse(
            std::env::var("SERVERS_LAZY").ok().as_deref(),
            std::env::var("SERVERS_LAZY_IDLE_SECS").ok().as_deref(),
            std::env::var("SERVERS_LAZY_WAIT_MS").ok().as_deref(),
        )
    }

    /// Pure core of [`LazyConfig::from_env`]. `idle` of `0` never unloads;
    /// an unparseable value falls back to the default rather than failing the
    /// boot.
    #[must_use]
    pub fn parse(servers: Option<&str>, idle: Option<&str>, wait: Option<&str>) -> Self {
        let mut list: Vec<Server> = servers
            .unwrap_or("")
            .split(',')
            .filter_map(Server::parse)
            .collect();
        list.dedup();
        let idle_secs = idle
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(DEFAULT_IDLE_SECS);
        let wait_ms = wait
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(DEFAULT_WAIT_MS)
            .min(MAX_WAIT_MS);
        Self {
            servers: list,
            idle: (idle_secs > 0).then(|| Duration::from_secs(idle_secs)),
            wait: Duration::from_millis(wait_ms),
        }
    }
}

static CONFIG: LazyLock<LazyConfig> = LazyLock::new(LazyConfig::from_env);

/// The process's lazy configuration (from the environment, read once).
pub fn config() -> &'static LazyConfig {
    &CONFIG
}

/// Bilibili shares CN's cell; loads and logs go by the canonical server.
fn canonical(server: Server) -> Server {
    if server == Server::Bilibili {
        Server::CN
    } else {
        server
    }
}

/// Make `server` resident if it is lazy, waiting up to `wait`.
///
/// `Ok` means the request can proceed: the server is loaded, or it is not lazy
/// (an eager server's availability is the handler's business), or it is not
/// configured (the handler answers 404). `Err` is the 503 to answer with.
///
/// # Errors
/// `ServiceUnavailableMessage` when the load failed, was refused admission,
/// or did not finish inside `wait`.
pub async fn ensure_loaded(
    state: &AppState,
    server: Server,
    wait: Duration,
) -> Result<(), ApiError> {
    let server = canonical(server);
    let Some(sd) = state.entry(server) else {
        return Ok(());
    };
    if !sd.residency.lazy {
        return Ok(());
    }
    sd.residency.touch();
    if sd.loaded.load(Ordering::Acquire) {
        return Ok(());
    }
    if let Some(why) = sd.residency.recent_failure() {
        return Err(unavailable(server, &why));
    }

    // Subscribe before starting, so the finish cannot slip past the waiter.
    let mut finished = sd.residency.done.subscribe();
    start_load(state, server, &sd);

    let outcome = tokio::time::timeout(wait, async {
        loop {
            if sd.loaded.load(Ordering::Acquire) {
                return Ok(());
            }
            if !sd.residency.loading.load(Ordering::Acquire) {
                let why = sd
                    .residency
                    .recent_failure()
                    .unwrap_or_else(|| "the load did not complete".to_owned());
                return Err(unavailable(server, &why));
            }
            if finished.changed().await.is_err() {
                return Err(unavailable(server, "the load was abandoned"));
            }
        }
    })
    .await;

    if let Ok(result) = outcome {
        return result;
    }
    tracing::info!(
        server = server.as_str(),
        wait_ms = wait.as_millis() as u64,
        "lazy load still running at the wait bound; answering 503"
    );
    Err(unavailable(server, "loading, retry shortly"))
}

fn unavailable(server: Server, why: &str) -> ApiError {
    ApiError::ServiceUnavailableMessage(format!(
        "game data for server '{}' is not loaded right now ({why})",
        server.as_str()
    ))
}

/// Start a load task unless one is already running (single flight).
fn start_load(state: &AppState, server: Server, sd: &Arc<ServerData>) {
    if sd
        .residency
        .loading
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    let state = state.clone();
    let sd = Arc::clone(sd);
    tokio::spawn(async move {
        let started = Instant::now();
        let outcome = load(&state, server, &sd).await;
        match &outcome {
            Ok(()) => tracing::info!(
                server = server.as_str(),
                elapsed_ms = started.elapsed().as_millis() as u64,
                "lazy server loaded"
            ),
            Err(why) => tracing::error!(
                server = server.as_str(),
                elapsed_ms = started.elapsed().as_millis() as u64,
                error = %why,
                "lazy server failed to load"
            ),
        }
        sd.residency.record(outcome);
        sd.residency.loading.store(false, Ordering::Release);
        sd.residency.done.send_modify(|n| *n += 1);
    });
}

async fn load(state: &AppState, server: Server, sd: &Arc<ServerData>) -> Result<(), String> {
    let guard = crate::core::asset_watcher::lock_reloads(server).await;
    if sd.loaded.load(Ordering::Acquire) {
        return Ok(());
    }
    let references = state.reference_servers(server);
    let (data_dir, assets_dir, art_dir) = (
        sd.game_data_dir.clone(),
        sd.assets_dir.clone(),
        sd.art_dir.clone(),
    );
    let built = crate::app::cpu::run("server_load", move || {
        crate::core::asset_watcher::build_server(&data_dir, &assets_dir, &art_dir, &references)
    })
    .await
    .map_err(|e| format!("not admitted: {e}"))?
    .map_err(|e| e.to_string())?;
    let (game_data, asset_index) = built;
    sd.game_data.store(Arc::new(game_data));
    sd.asset_index.store(Arc::new(asset_index));
    sd.loaded.store(true, Ordering::Release);
    sd.residency.touch();
    drop(guard);
    // What the boot pass records for an eager server, recorded at load, under
    // the version of any update that arrived while the server was unloaded.
    let res_version = sd.residency.take_res_version();
    crate::core::release::ledger::spawn_record(state.clone(), server, res_version);
    Ok(())
}

/// Unload every lazy server idle for at least `idle`. Returns the servers
/// unloaded. Takes the reload lock per server, so an unload never lands in the
/// middle of a reload or a load.
pub async fn unload_idle(state: &AppState, idle: Duration) -> Vec<Server> {
    let mut seen: Vec<usize> = Vec::new();
    let mut unloaded = Vec::new();
    let mut entries: Vec<(Server, Arc<ServerData>)> = state
        .servers
        .iter()
        .map(|(s, sd)| (*s, Arc::clone(sd)))
        .collect();
    entries.sort_by_key(|(s, _)| s.as_str());
    for (server, sd) in entries {
        if seen.contains(&(Arc::as_ptr(&sd) as usize)) {
            continue;
        }
        seen.push(Arc::as_ptr(&sd) as usize);
        let server = canonical(server);
        if !sd.residency.lazy || !sd.loaded.load(Ordering::Acquire) {
            continue;
        }
        if sd.residency.idle_secs() < idle.as_secs() {
            continue;
        }
        let _guard = crate::core::asset_watcher::lock_reloads(server).await;
        // Re-checked under the lock: a request may have touched it meanwhile.
        if !sd.loaded.load(Ordering::Acquire)
            || sd.residency.loading.load(Ordering::Acquire)
            || sd.residency.idle_secs() < idle.as_secs()
        {
            continue;
        }
        let idle_secs = sd.residency.idle_secs();
        sd.unload();
        tracing::info!(
            server = server.as_str(),
            idle_secs,
            "lazy server idle, game data unloaded"
        );
        unloaded.push(server);
    }
    unloaded
}

/// Spawn the idle reaper when any server is lazy and an idle TTL is set.
pub fn spawn_reaper(state: AppState) {
    let Some(idle) = config().idle else {
        return;
    };
    if !state.servers.values().any(|sd| sd.residency.lazy) {
        return;
    }
    let period = (idle / 4).clamp(Duration::from_secs(1), Duration::from_secs(60));
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(period);
        loop {
            tick.tick().await;
            unload_idle(&state, idle).await;
        }
    });
}

/// The servers a request names: the path segment matched by `{server}`, and
/// any `server=` query parameter. Unparseable values are left to the handler.
fn requested_servers(req: &Request) -> Vec<Server> {
    let mut out = Vec::new();
    if let Some(matched) = req.extensions().get::<MatchedPath>() {
        let pattern: Vec<&str> = matched.as_str().split('/').collect();
        if let Some(i) = pattern.iter().position(|seg| *seg == "{server}")
            && let Some(seg) = req.uri().path().split('/').nth(i)
            && let Some(server) = Server::parse(seg)
        {
            out.push(server);
        }
    }
    if let Some(query) = req.uri().query() {
        for pair in query.split('&') {
            if let Some(("server", value)) = pair.split_once('=')
                && let Some(server) = Server::parse(value)
                && !out.contains(&server)
            {
                out.push(server);
            }
        }
    }
    out
}

/// Route layer: make every lazy server the request names resident before the
/// handler runs. A no-op unless some server is lazy.
pub async fn ensure_requested_server(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    if state.servers.values().any(|sd| sd.residency.lazy) {
        for server in requested_servers(&req) {
            if let Err(e) = ensure_loaded(&state, server, config().wait).await {
                return e.into_response();
            }
        }
    }
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::cache::keys::CacheKey;
    use crate::app::cache::store::CacheStore;
    use crate::app::state::{AppConfig, load_server_map_with};
    use crate::core::auth::credentials::CredentialKey;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    /// A scratch `ASSETS_DIR`, removed on drop.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "myrtle-residency-{tag}-{}-{}",
                std::process::id(),
                now_secs()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("scratch dir");
            Self(dir)
        }

        /// The smallest tree the loader accepts: one empty `character_table`.
        /// Every other table falls back to its default with a warning.
        fn write_server(&self, server: Server) {
            let excel = self.0.join(server.as_str()).join("gamedata/excel");
            std::fs::create_dir_all(&excel).expect("excel dir");
            std::fs::write(excel.join("character_table.json"), br#"{"Characters":[]}"#)
                .expect("character_table");
        }

        fn base(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn state_for(base: &Path, servers: &[Server], lazy: &[Server]) -> AppState {
        let config = AppConfig {
            jwt_secret: "test-jwt-secret-not-used-by-this-test".into(),
            game_credential_key: CredentialKey::parse(&"11".repeat(32)).expect("key"),
            rate_limit_rpm: 100,
            service_key: "test-service-key".into(),
            assets_base_dir: base.to_string_lossy().into_owned(),
            servers: servers.to_vec(),
            default_server: servers[0],
            asset_ws_urls: HashMap::new(),
        };
        let map = load_server_map_with(&config, lazy, |_| ());
        let db = sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(Duration::from_millis(200))
            .connect_lazy("postgres://nobody:nothing@127.0.0.1:1/none")
            .expect("lazy pool");
        AppState::new(
            db,
            CacheStore::new_memory(),
            map,
            servers[0],
            config,
            reqwest::Client::new(),
            crate::core::service_account::ServiceAccounts::default(),
        )
    }

    /// Game-data loads call `startup::step`, which advances a process-global
    /// boot timeline; the startup tests time that timeline, so a load running
    /// beside them corrupts their measurement. Every test here that loads holds
    /// their lock.
    fn serial() -> std::sync::MutexGuard<'static, ()> {
        crate::core::startup::tests::SERIAL
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn is_unavailable(r: Result<Arc<ServerData>, ApiError>) -> bool {
        matches!(r, Err(ApiError::ServiceUnavailableMessage(_)))
    }

    /// Task 1: a default server with no data no longer panics the boot. Both
    /// configured servers come up unloaded and answer 503; an unconfigured one
    /// stays 404; the next reload for a server whose files are now complete
    /// brings it in, and internal lookups fall back to it.
    #[allow(clippy::await_holding_lock)] // test-only serialisation
    #[tokio::test]
    async fn failed_servers_boot_unavailable_and_a_reload_retries_them() {
        let _serial = serial();
        let scratch = Scratch::new("fail");
        let state = state_for(scratch.base(), &[Server::EN, Server::CN], &[]);

        assert!(state.try_server_data(Server::EN).is_none());
        assert!(state.try_server_data(Server::CN).is_none());
        assert!(is_unavailable(state.require_server_data(Server::EN)));
        assert!(is_unavailable(state.require_server_data(Server::CN)));
        assert!(is_unavailable(state.require_server_data(Server::Bilibili)));
        assert!(matches!(
            state.require_server_data(Server::JP),
            Err(ApiError::NotFound)
        ));
        // Placeholders hold empty data, not a borrowed server's.
        assert_eq!(state.game_data(Server::CN).operators.len(), 0);

        // The files arrive; the watcher's reload for CN retries the load.
        scratch.write_server(Server::CN);
        crate::core::asset_watcher::perform_reload(&state, Server::CN, None).await;
        let cn = state
            .require_server_data(Server::CN)
            .expect("CN loaded on retry");
        assert!(
            state.require_server_data(Server::Bilibili).is_ok(),
            "alias follows CN"
        );
        assert!(
            is_unavailable(state.require_server_data(Server::EN)),
            "EN still missing"
        );
        // The unloaded default resolves to the first loaded server internally.
        assert!(Arc::ptr_eq(&state.server_data(Server::EN), &cn));
    }

    /// Task 2: a lazy server is unloaded at boot but pickable, loads once for
    /// a burst of concurrent requests, unloads when idle, and a hot reload
    /// does not make it resident again.
    #[allow(clippy::await_holding_lock)] // test-only serialisation
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn lazy_server_loads_once_on_request_and_unloads_when_idle() {
        let _serial = serial();
        let scratch = Scratch::new("lazy");
        scratch.write_server(Server::EN);
        scratch.write_server(Server::JP);
        let state = state_for(scratch.base(), &[Server::EN, Server::JP], &[Server::JP]);

        let jp = state.entry(Server::JP).expect("JP configured");
        assert!(jp.residency.lazy);
        assert!(
            !jp.loaded.load(Ordering::Acquire),
            "lazy = unloaded at boot"
        );
        assert!(state.pickable_servers().contains(&Server::JP));
        assert!(
            state.try_server_data(Server::EN).is_some(),
            "the default is never lazy"
        );

        let mut burst = tokio::task::JoinSet::new();
        for _ in 0..8 {
            let state = state.clone();
            burst.spawn(
                async move { ensure_loaded(&state, Server::JP, Duration::from_secs(20)).await },
            );
        }
        while let Some(r) = burst.join_next().await {
            assert!(r.expect("join").is_ok());
        }
        assert!(jp.loaded.load(Ordering::Acquire));
        assert_eq!(*jp.residency.done.borrow(), 1, "eight requests, ONE load");

        // Not idle yet under a long TTL; idle under a zero one.
        assert!(
            unload_idle(&state, Duration::from_secs(3600))
                .await
                .is_empty()
        );
        assert_eq!(unload_idle(&state, Duration::ZERO).await, vec![Server::JP]);
        assert!(!jp.loaded.load(Ordering::Acquire));
        assert_eq!(jp.game_data.load().operators.len(), 0);
        assert!(
            state.try_server_data(Server::EN).is_some(),
            "the default is never unloaded"
        );

        // A hot reload of an unloaded lazy server skips the rebuild, but still
        // drops the server's cached responses (24 h, else an update's new
        // operators stay missing from them) and keeps the update's version
        // for the next load's release-ledger record.
        let jp_list = CacheKey::StaticData {
            resource: "operators",
            server: Server::JP.as_str(),
            fields_hash: 0,
            page: 0,
        };
        let en_list = CacheKey::StaticData {
            resource: "operators",
            server: Server::EN.as_str(),
            fields_hash: 0,
            page: 0,
        };
        state.cache.set_raw(&jp_list, "old".into()).await;
        state.cache.set_raw(&en_list, "en".into()).await;
        crate::core::asset_watcher::perform_reload(&state, Server::JP, Some("26-10-07-v1")).await;
        assert!(!jp.loaded.load(Ordering::Acquire));
        assert_eq!(
            state.cache.get_raw(&jp_list).await,
            None,
            "JP cache cleared"
        );
        assert_eq!(
            state.cache.get_raw(&en_list).await.as_deref(),
            Some("en"),
            "other servers' cache kept"
        );
        assert_eq!(
            jp.residency.pending_res_version.lock().unwrap().as_deref(),
            Some("26-10-07-v1")
        );

        // And a request brings it back, taking the deferred version.
        ensure_loaded(&state, Server::JP, Duration::from_secs(20))
            .await
            .expect("reload on request");
        assert!(jp.loaded.load(Ordering::Acquire));
        assert_eq!(*jp.residency.pending_res_version.lock().unwrap(), None);
    }

    /// Task 3: a sidecar patch publishes a new `GameData` that shares the live
    /// tables instead of a rebuilt copy.
    #[allow(clippy::await_holding_lock)] // test-only serialisation
    #[tokio::test]
    async fn sidecar_patch_shares_the_live_tables() {
        let _serial = serial();
        let scratch = Scratch::new("patch");
        scratch.write_server(Server::EN);
        scratch.write_server(Server::CN);
        let state = state_for(scratch.base(), &[Server::EN, Server::CN], &[]);
        let cn = state.entry(Server::CN).expect("CN");
        let before = cn.game_data.load_full();

        crate::core::asset_watcher::perform_sidecar_patch(&state, Server::CN).await;
        let after = cn.game_data.load_full();
        assert!(
            !Arc::ptr_eq(&before, &after),
            "a new GameData was published"
        );
        assert!(after.shares_tables(&before), "its tables are the live ones");

        // A full reload, by contrast, builds new tables.
        crate::core::asset_watcher::perform_reload(&state, Server::CN, None).await;
        assert!(!cn.game_data.load_full().shares_tables(&after));
    }

    #[test]
    fn unset_is_all_eager() {
        let c = LazyConfig::parse(None, None, None);
        assert!(c.servers.is_empty());
        assert_eq!(c.idle, Some(Duration::from_secs(DEFAULT_IDLE_SECS)));
        assert_eq!(c.wait, Duration::from_millis(DEFAULT_WAIT_MS));
    }

    #[test]
    fn parses_servers_and_knobs() {
        let c = LazyConfig::parse(Some("jp, kr,bogus"), Some("0"), Some("99999"));
        assert_eq!(c.servers, vec![Server::JP, Server::KR]);
        assert_eq!(c.idle, None, "0 never unloads");
        assert_eq!(c.wait, Duration::from_millis(MAX_WAIT_MS), "wait is capped");
    }

    /// The route layer loads a lazy server named by the `{server}` path
    /// segment or by `?server=`, and answers 503 + `Retry-After` when the
    /// load fails; unnamed and eager servers pass straight through.
    #[allow(clippy::await_holding_lock)] // test-only serialisation
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn route_layer_loads_the_named_server() {
        let _serial = serial();
        use axum::body::Body;
        use axum::http::{Request as HttpRequest, StatusCode};
        use tower::ServiceExt;

        let scratch = Scratch::new("layer");
        scratch.write_server(Server::EN);
        scratch.write_server(Server::JP);
        // KR is lazy with no files: its load fails.
        let state = state_for(
            scratch.base(),
            &[Server::EN, Server::JP, Server::KR],
            &[Server::JP, Server::KR],
        );
        let app = axum::Router::new()
            .route("/api/{server}/thing", axum::routing::get(|| async { "ok" }))
            .route("/api/thing", axum::routing::get(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                ensure_requested_server,
            ))
            .with_state(state.clone());
        let get = |uri: &str| {
            HttpRequest::get(uri.to_owned())
                .body(Body::empty())
                .expect("request")
        };

        let r = app.clone().oneshot(get("/api/thing")).await.expect("resp");
        assert_eq!(r.status(), StatusCode::OK);
        assert!(
            state.try_server_data(Server::JP).is_none(),
            "nothing named, nothing loaded"
        );

        let r = app
            .clone()
            .oneshot(get("/api/jp/thing"))
            .await
            .expect("resp");
        assert_eq!(r.status(), StatusCode::OK);
        assert!(
            state.try_server_data(Server::JP).is_some(),
            "path segment loads JP"
        );

        let r = app
            .clone()
            .oneshot(get("/api/thing?x=1&server=kr"))
            .await
            .expect("resp");
        assert_eq!(r.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            r.headers()
                .get("retry-after")
                .map(axum::http::HeaderValue::as_bytes),
            Some(&b"5"[..])
        );
        // Inside the failure backoff, no second attempt is made.
        let attempts = *state.entry(Server::KR).expect("KR").residency.done.borrow();
        let r = app
            .clone()
            .oneshot(get("/api/kr/thing"))
            .await
            .expect("resp");
        assert_eq!(r.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            *state.entry(Server::KR).expect("KR").residency.done.borrow(),
            attempts
        );
    }
}
