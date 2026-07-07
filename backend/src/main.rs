use arc_swap::ArcSwap;
use backend::app::server;
use backend::core::hypergryph::{config, loaders};
use backend::core::{
    asset_watcher, dps_watcher, leaderboard_snapshot_job, medal_ownership_job,
    operator_ownership_job, regrade_job, trending_job,
};
use backend::{
    app::{
        cache::store::CacheStore,
        state::{AppConfig, AppState, ServerData, derive_assets_dir, derive_game_data_dir},
    },
    core::{
        gamedata::assets::AssetIndex,
        hypergryph::{config::GlobalConfig, constants::Server},
    },
};
use dotenv::dotenv;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[allow(non_upper_case_globals)]
#[unsafe(export_name = "_rjem_malloc_conf")]
pub static MALLOC_CONF: &[u8] = b"background_thread:true,dirty_decay_ms:5000,muzzy_decay_ms:5000\0";

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=info,tower_http=info".into()),
        )
        .init();

    // Game data (per-server). ASSETS_DIR is a base dir; each server loads from
    // `{base}/{server}` (+ `/gamedata/excel`). SERVERS selects which to load.
    let config = AppConfig::from_env();
    let mut servers: HashMap<Server, Arc<ServerData>> = HashMap::new();
    for &srv in &config.servers {
        let game_data_dir = derive_game_data_dir(&config.assets_base_dir, srv);
        let assets_dir = derive_assets_dir(&config.assets_base_dir, srv);
        info!(server = srv.as_str(), "loading game data...");
        let game_data = backend::core::gamedata::init_game_data(
            Path::new(&game_data_dir),
            Path::new(&assets_dir),
        )
        .unwrap_or_else(|e| panic!("failed to load game data for {}: {e}", srv.as_str()));
        let asset_index = AssetIndex::build(Path::new(&assets_dir));
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
            }),
        );
    }
    // Bilibili shares CN's Hypergryph data (same Arc cell, hot-reloads together).
    if let Some(cn) = servers.get(&Server::CN).cloned() {
        servers.entry(Server::Bilibili).or_insert(cn);
    }
    let default_server = config.default_server;

    // Database (pool + migrations + seeding)
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = backend::database::init(&database_url)
        .await
        .expect("failed to initialize database");

    // Cache (Redis or in-memory fallback)
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

    // reqwest client
    let http_client = reqwest::Client::new();

    // Initialize configs
    config::init_config(GlobalConfig::new());
    loaders::init(&http_client).await;

    // Start server
    let state = AppState::new(db, cache, servers, default_server, config, http_client);

    // Background watchers + cron jobs. These query Postgres and (in the case of
    // `regrade_job`) fan out parallel workers across every user, which is heavy
    // and pointless for local stage-viewer / API work. Set
    // `DISABLE_BACKGROUND_JOBS=1` to skip them during local development.
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
    }

    server::run(state).await.expect("server error");
}
