use backend::app::server;
use backend::core::hypergryph::{config, loaders};
use backend::core::{
    asset_watcher, dps_watcher, leaderboard_snapshot_job, regrade_job, trending_job,
};
use backend::{
    app::{
        cache::store::CacheStore,
        state::{AppConfig, AppState},
    },
    core::{gamedata::assets::AssetIndex, hypergryph::config::GlobalConfig},
};
use dotenv::dotenv;
use std::path::Path;
use tracing::{info, warn};

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

    // Game data
    let data_dir_str =
        std::env::var("GAME_DATA_DIR").unwrap_or_else(|_| "../assets/output/gamedata/excel".into());
    let assets_dir_str = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output".into());

    info!("loading game data...");
    let game_data = backend::core::gamedata::init_game_data(
        Path::new(&data_dir_str),
        Path::new(&assets_dir_str),
    )
    .expect("failed to load game data");
    info!(operators = game_data.operators.len(), "game data loaded");

    // Build asset index
    let asset_index = AssetIndex::build(Path::new(&assets_dir_str));

    // Database (pool + migrations + seeding)
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = backend::database::init(&database_url)
        .await
        .expect("failed to initialize database");

    // Cache (Redis or in-memory fallback)
    let cache = if let Ok(url) = std::env::var("REDIS_URL") { match redis::Client::open(url) {
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
    } } else {
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
    let config = AppConfig::from_env();
    let state = AppState::new(db, cache, game_data, asset_index, config, http_client);

    // Spawn asset hot-reload watcher (connects to asset pipeline WebSocket)
    asset_watcher::spawn(state.clone());

    // Spawn DPS formula auto-update watcher (polls GitHub for upstream changes)
    dps_watcher::spawn(state.clone());

    // Spawn cron jobs
    trending_job::spawn(state.clone());
    leaderboard_snapshot_job::spawn(state.clone());
    regrade_job::spawn(state.clone());

    server::run(state).await.expect("server error");
}
