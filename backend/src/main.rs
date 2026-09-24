use backend::app::server;
use backend::core::hypergryph::{config, loaders};
use backend::core::service_account::ServiceAccounts;
use backend::core::startup;
use backend::core::{
    asset_watcher, dps_watcher, event_shop_job, gacha_detail_job, leaderboard_snapshot_job,
    medal_ownership_job, operator_ownership_job, regrade_job, release, trending_job,
};
use backend::{
    app::{
        cache::store::CacheStore,
        state::{AppConfig, AppState, derive_game_data_dir, load_server_map},
    },
    core::hypergryph::config::GlobalConfig,
};
use dotenv::dotenv;
use std::path::Path;
use std::time::Duration;
use tracing::{info, warn};

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[allow(non_upper_case_globals)]
#[unsafe(export_name = "_rjem_malloc_conf")]
pub static MALLOC_CONF: &[u8] = b"background_thread:true,dirty_decay_ms:5000,muzzy_decay_ms:5000\0";

/// Runtime shutdown is bounded: dropping a tokio runtime waits for every
/// blocking-pool thread, and a base search parked there (40 s in a debug
/// build) kept Ctrl+C hanging for its whole duration. `server::run` drains
/// connections under a grace period first; this bounds the rest.
fn main() {
    spawn_shutdown_watchdog();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    runtime.block_on(async_main());
    runtime.shutdown_timeout(std::time::Duration::from_secs(2));
}

/// How long past a shutdown signal the process may still be here before it is
/// ended from outside the runtime.
const HARD_EXIT_GRACE: Duration = Duration::from_secs(15);

/// A shutdown signal the main runtime cannot ANSWER still ends the process.
///
/// `server::run` drains under a 5 s grace and `shutdown_timeout` bounds the
/// blocking pool, but both of those are futures on the main runtime: when
/// every worker is parked in synchronous compute, nothing polls them and the
/// signal is simply never seen. That is what happened on 2026-09-23, when one
/// account's grade held a worker for 39,861 ms in a debug build and SIGTERM
/// went unanswered until the process was killed by hand. This watchdog owns
/// its own thread and its own single-threaded runtime, so it is polled
/// whatever the main runtime is doing, and it leaves once the grace is up.
/// A clean shutdown is faster than the grace, so this never fires on one.
fn spawn_shutdown_watchdog() {
    let started = std::thread::Builder::new()
        .name("shutdown-watchdog".into())
        .spawn(|| {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                eprintln!("shutdown watchdog: no runtime; shutdown stays unbounded");
                return;
            };
            runtime.block_on(async {
                #[cfg(unix)]
                {
                    use tokio::signal::unix::{SignalKind, signal};
                    let (Ok(mut term), Ok(mut interrupt)) = (
                        signal(SignalKind::terminate()),
                        signal(SignalKind::interrupt()),
                    ) else {
                        eprintln!("shutdown watchdog: no signal handler; shutdown stays unbounded");
                        return;
                    };
                    tokio::select! {
                        _ = term.recv() => {},
                        _ = interrupt.recv() => {},
                    }
                }
                #[cfg(not(unix))]
                {
                    if tokio::signal::ctrl_c().await.is_err() {
                        return;
                    }
                }
                tokio::time::sleep(HARD_EXIT_GRACE).await;
                // Written straight to stderr rather than through tracing: the
                // reason this thread exists is that the rest of the process
                // may be unable to make progress.
                eprintln!(
                    "shutdown watchdog: {} s past the signal and the runtime has not left; exiting",
                    HARD_EXIT_GRACE.as_secs()
                );
                std::process::exit(0);
            });
        });
    if let Err(e) = started {
        eprintln!("shutdown watchdog not started ({e}); shutdown stays unbounded");
    }
}

// Startup wiring is inherently a long, linear sequence of `.await`s; splitting it into
// helpers would not make it more readable. Matches the crate-wide allow in `lib.rs`,
// which does not cover this binary's separate crate root.
#[allow(clippy::too_many_lines)]
async fn async_main() {
    dotenv().ok();

    // The writer clears the boot progress bars before a line lands, so startup
    // logging and the bars can share a terminal.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=info,tower_http=info".into()),
        )
        .with_writer(startup::log_writer())
        .init();

    // Game data (per-server). ASSETS_DIR is a base dir; each server loads from
    // `{base}/{server}` (+ `/gamedata/excel`). SERVERS selects which to load.
    let config = AppConfig::from_env();

    // Costed from the previous boot's timings (see `core::startup`).
    let boot = startup::Boot::start(boot_plan(&config));

    // Same loader the tool binaries use, so a forced refresh sees exactly the
    // game data the server does, placeholder fallbacks and the Bilibili/CN
    // aliasing included.
    let servers = load_server_map(&config, |key| boot.phase(key));
    let default_server = config.default_server;

    // Database (pool + migrations + seeding)
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = {
        let _phase = boot.phase("database");
        backend::database::init(&database_url)
            .await
            .expect("failed to initialize database")
    };

    // Cache (Redis or in-memory fallback)
    let cache_phase = boot.phase("cache");
    startup::step("connect");
    let cache = if let Ok(url) = std::env::var("REDIS_URL") {
        match redis::Client::open(url) {
            Ok(client) => match redis::aio::ConnectionManager::new(client).await {
                Ok(conn) => {
                    info!("connected to Redis");
                    CacheStore::new_redis(conn)
                }
                Err(e) => {
                    warn!(error = %e, "Redis unavailable, falling back to in-memory cache");
                    CacheStore::new_memory()
                }
            },
            Err(e) => {
                warn!(error = %e, "invalid REDIS_URL, falling back to in-memory cache");
                CacheStore::new_memory()
            }
        }
    } else {
        info!("REDIS_URL not set, using in-memory cache");
        CacheStore::new_memory()
    };
    cache.spawn_cleanup();
    drop(cache_phase);

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(5))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(16)
        .build()
        .expect("failed to build HTTP client");

    {
        let _phase = boot.phase("hypergryph");
        config::init_config(GlobalConfig::new());
        loaders::init(&http_client).await;
    }

    // The backend's own game accounts, one per configured server. Optional:
    // without them the pool-detail refresh is skipped and banners fall back to
    // their static rate-up blobs.
    let service_accounts = {
        let _phase = boot.phase("accounts");
        startup::step("load");
        ServiceAccounts::load(&config.servers)
    };

    let state = AppState::new(
        db,
        cache,
        servers,
        default_server,
        config,
        http_client,
        service_accounts,
    );

    // Report any table that fell back to an empty default at boot. Same alert the
    // hot-reload path fires; startup needs its own call because `perform_reload`
    // never runs at startup.
    for &srv in &state.config.servers {
        if let Some(sd) = state.try_server_data(srv) {
            let warnings = sd.game_data.load_full().table_warnings.clone();
            backend::core::alerts::report_degraded_tables(
                &state.http_client,
                srv.as_str(),
                &warnings,
            )
            .await;
        }
    }

    // Background watchers + cron jobs. These query Postgres and (in the case of
    // `regrade_job`) fan out parallel workers across every user, which is heavy
    // and pointless for local stage-viewer / API work. Set
    // `DISABLE_BACKGROUND_JOBS=1` to skip them during local development.
    let jobs_phase = boot.phase("jobs");
    startup::step("spawn");
    let jobs_disabled = std::env::var("DISABLE_BACKGROUND_JOBS")
        .is_ok_and(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes"));
    if jobs_disabled {
        info!("DISABLE_BACKGROUND_JOBS set - skipping asset/DPS watchers and cron jobs");
    } else {
        asset_watcher::spawn(state.clone());

        dps_watcher::spawn(state.clone());

        trending_job::spawn(state.clone());
        leaderboard_snapshot_job::spawn(state.clone());
        operator_ownership_job::spawn(state.clone());
        medal_ownership_job::spawn(state.clone());
        regrade_job::spawn(state.clone());
        gacha_detail_job::spawn(state.clone());
        event_shop_job::spawn(state.clone());
    }

    drop(jobs_phase);

    // Boot pass of the release ledger: record what every loaded server carries
    // right now. No version is known at boot, so this pass never creates debut
    // links; the watcher's reloads do. Outside the jobs gate because it is a
    // one-shot write; RELEASE_LEDGER=0 is its own switch.
    {
        let mut seen: std::collections::HashSet<*const ()> = std::collections::HashSet::new();
        for (&server, sd) in &state.servers {
            // Bilibili aliases CN's cell; record each cell once.
            if !seen.insert(std::sync::Arc::as_ptr(sd).cast::<()>()) {
                continue;
            }
            if !sd.loaded.load(std::sync::atomic::Ordering::Acquire) {
                continue;
            }
            release::ledger::spawn_record(state.clone(), server, None);
        }
    }

    // The story library index, off the request path. It loads and parses every
    // script (EN: 1,887 of them), which is seconds of CPU, so the first reader
    // must not be the one who pays it. Spawned, so it does not delay listening,
    // and the lazy build in the service stays the fallback.
    {
        let mut seen: std::collections::HashSet<*const ()> = std::collections::HashSet::new();
        for (&server, sd) in &state.servers {
            if !seen.insert(std::sync::Arc::as_ptr(sd).cast::<()>()) {
                continue;
            }
            if !sd.loaded.load(std::sync::atomic::Ordering::Acquire) {
                continue;
            }
            backend::app::services::story::spawn_warm(state.clone(), server);
            // And the community reading aggregate on top of it: a walk over
            // every account's stage records, which is far too long to run on a
            // request. It recomputes on its own six-hour clock from here.
            backend::app::services::story_community::spawn_warm(state.clone(), server);
        }
    }

    // Finish before `server::run`, so the listener's log line lands after the
    // bars are gone.
    boot.finish();

    server::run(state).await.expect("server error");
}

/// The boot, phase by phase, costed from the previous boot's timings.
///
/// The per-server game-data steps come from `gamedata::boot_steps`, declared
/// next to the code that reports them.
fn boot_plan(config: &AppConfig) -> Vec<startup::PhaseSpec> {
    let mut plan = Vec::new();
    for &srv in &config.servers {
        let game_data_dir = derive_game_data_dir(&config.assets_base_dir, srv);
        plan.push(
            startup::PhaseSpec::new(
                format!("gamedata:{}", srv.as_str()),
                format!("game data · {}", srv.as_str().to_uppercase()),
            )
            .with_steps(backend::core::gamedata::boot_steps(Path::new(
                &game_data_dir,
            ))),
        );
    }
    plan.push(startup::PhaseSpec::new("database", "database").with_steps([
        "connect",
        "migrations",
        "seed",
    ]));
    plan.push(startup::PhaseSpec::new("cache", "cache").with_steps(["connect"]));
    plan.push(
        startup::PhaseSpec::new("hypergryph", "hypergryph").with_steps([
            "device ids",
            "network config",
            "version config",
        ]),
    );
    plan.push(startup::PhaseSpec::new("accounts", "service accounts").with_steps(["load"]));
    plan.push(startup::PhaseSpec::new("jobs", "background jobs").with_steps(["spawn"]));
    plan
}
