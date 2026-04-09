use backend::{
    app::{
        cache::store::CacheStore,
        state::{AppConfig, AppState},
    },
    core::{gamedata::assets::AssetIndex, hypergryph::config::GlobalConfig},
};
use dotenv::dotenv;
use std::path::Path;

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

    tracing::info!("loading game data...");
    let game_data = backend::core::gamedata::init_game_data(
        Path::new(&data_dir_str),
        Path::new(&assets_dir_str),
    )
    .expect("failed to load game data");
    tracing::info!(operators = game_data.operators.len(), "game data loaded");

    // Build asset index
    let asset_index = AssetIndex::build(Path::new(&assets_dir_str));

    // Database (pool + migrations + seeding)
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = backend::database::init(&database_url)
        .await
        .expect("failed to initialize database");

    // Cache (Redis or in-memory fallback)
    let cache = match std::env::var("REDIS_URL") {
        Ok(url) => match redis::Client::open(url) {
            Ok(client) => match redis::aio::ConnectionManager::new(client).await {
                Ok(conn) => {
                    tracing::info!("connected to Redis");
                    CacheStore::new_redis(conn)
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Redis unavailable, falling back to in-memory cache");
                    CacheStore::new_memory()
                }
            },
            Err(e) => {
                tracing::warn!(error = %e, "invalid REDIS_URL, falling back to in-memory cache");
                CacheStore::new_memory()
            }
        },
        Err(_) => {
            tracing::info!("REDIS_URL not set, using in-memory cache");
            CacheStore::new_memory()
        }
    };
    cache.spawn_cleanup();

    // reqwest client
    let http_client = reqwest::Client::new();

    // Initialize configs
    backend::core::hypergryph::config::init_config(GlobalConfig::new());
    backend::core::hypergryph::loaders::init(&http_client).await;

    // Start server
    let config = AppConfig::from_env();
    let state = AppState::new(db, cache, game_data, asset_index, config, http_client);

    // Spawn asset hot-reload watcher (connects to asset pipeline WebSocket)
    backend::core::asset_watcher::spawn(state.clone());

    backend::app::server::run(state)
        .await
        .expect("server error");
}
