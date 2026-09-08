use arc_swap::ArcSwap;
use backend::app::server;
use backend::core::hypergryph::{config, loaders};
use backend::core::service_account::ServiceAccounts;
use backend::core::startup;
use backend::core::{
    asset_watcher, dps_watcher, gacha_detail_job, leaderboard_snapshot_job, medal_ownership_job,
    operator_ownership_job, regrade_job, trending_job,
};
use backend::{
    app::{
        cache::store::CacheStore,
        state::{AppConfig, AppState, ServerData, derive_assets_dir, derive_game_data_dir},
    },
    core::hypergryph::{config::GlobalConfig, constants::Server},
};
use dotenv::dotenv;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, atomic::AtomicBool};
use tracing::{info, warn};

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[allow(non_upper_case_globals)]
#[unsafe(export_name = "_rjem_malloc_conf")]
pub static MALLOC_CONF: &[u8] = b"background_thread:true,dirty_decay_ms:5000,muzzy_decay_ms:5000\0";

#[tokio::main]
// Startup wiring is inherently a long, linear sequence of `.await`s; splitting it into
// helpers would not make it more readable. Matches the crate-wide allow in `lib.rs`,
// which does not cover this binary's separate crate root.
#[allow(clippy::too_many_lines)]
async fn main() {
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

    let mut servers: HashMap<Server, Arc<ServerData>> = HashMap::new();
    for &srv in &config.servers {
        let game_data_dir = derive_game_data_dir(&config.assets_base_dir, srv);
        let assets_dir = derive_assets_dir(&config.assets_base_dir, srv);
        let load_result = {
            let _phase = boot.phase(&format!("gamedata:{}", srv.as_str()));
            backend::core::gamedata::init_game_data(
                Path::new(&game_data_dir),
                Path::new(&assets_dir),
            )
        };
        match load_result {
            Ok((game_data, asset_index)) => {
                info!(
                    server = srv.as_str(),
                    operators = game_data.operators.len(),
                    "game data loaded"
                );
                servers.insert(
                    srv,
                    Arc::new(ServerData {
                        game_data: ArcSwap::from_pointee(game_data),
                        asset_index: ArcSwap::from_pointee(asset_index),
                        game_data_dir,
                        assets_dir,
                        loaded: AtomicBool::new(true),
                    }),
                );
            }
            Err(e) if srv == config.default_server => {
                panic!("failed to load game data for {}: {e}", srv.as_str());
            }
            Err(e) => {
                tracing::error!(
                    server = srv.as_str(),
                    error = %e,
                    "game data failed to load; serving default-server data for this server until a hot reload succeeds"
                );
                let default_entry = servers
                    .get(&config.default_server)
                    .expect("default server data must be present");
                servers.insert(
                    srv,
                    Arc::new(ServerData {
                        game_data: ArcSwap::new(default_entry.game_data.load_full()),
                        asset_index: ArcSwap::new(default_entry.asset_index.load_full()),
                        game_data_dir,
                        assets_dir,
                        loaded: AtomicBool::new(false),
                    }),
                );
            }
        }
    }
    // Bilibili shares CN's Hypergryph data (same Arc cell, hot-reloads together).
    if let Some(cn) = servers.get(&Server::CN).cloned() {
        servers.entry(Server::Bilibili).or_insert(cn);
    }
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

    // reqwest client
    let http_client = reqwest::Client::new();

    // Initialize configs
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

    // Start server
    let state = AppState::new(
        db,
        cache,
        servers,
        default_server,
        config,
        http_client,
        service_accounts,
    );

    // Background watchers + cron jobs. These query Postgres and (in the case of
    // `regrade_job`) fan out parallel workers across every user, which is heavy
    // and pointless for local stage-viewer / API work. Set
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

    // `DISABLE_BACKGROUND_JOBS=1` to skip them during local development.
    let jobs_phase = boot.phase("jobs");
    startup::step("spawn");
    let jobs_disabled = std::env::var("DISABLE_BACKGROUND_JOBS")
        .is_ok_and(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes"));
    if jobs_disabled {
        info!("DISABLE_BACKGROUND_JOBS set - skipping asset/DPS watchers and cron jobs");
    } else {
        // Spawn asset hot-reload watcher (connects to asset pipeline WebSocket)
        asset_watcher::spawn(state.clone());

        // Spawn DPS formula auto-update watcher (polls GitHub for upstream changes)
        dps_watcher::spawn(state.clone());

        // Spawn cron jobs
        trending_job::spawn(state.clone());
        leaderboard_snapshot_job::spawn(state.clone());
        operator_ownership_job::spawn(state.clone());
        medal_ownership_job::spawn(state.clone());
        regrade_job::spawn(state.clone());
        gacha_detail_job::spawn(state.clone());
    }

    drop(jobs_phase);

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
