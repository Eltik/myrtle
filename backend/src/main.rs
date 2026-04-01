use backend::{
    app::state::{AppConfig, AppState},
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

    // Redis
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    let redis_client = redis::Client::open(redis_url).expect("invalid REDIS_URL");
    let redis = redis::aio::ConnectionManager::new(redis_client)
        .await
        .expect("failed to connect to redis");

    // reqwest client
    let http_client = reqwest::Client::new();

    // Initialize configs
    backend::core::hypergryph::config::init_config(GlobalConfig::new());
    backend::core::hypergryph::loaders::init(&http_client).await;

    // Start server
    let config = AppConfig::from_env();
    let state = AppState::new(db, redis, game_data, asset_index, config, http_client);

    backend::app::server::run(state)
        .await
        .expect("server error");
}
