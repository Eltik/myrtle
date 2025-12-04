use axum::middleware;
use axum::routing::post;
use axum::{Router, response::Json, routing::get};
use reqwest::Client;
use serde::Serialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use crate::app::middleware::rate_limit::{RateLimitStore, rate_limit};
use crate::app::routes::get_user::{get_user_by_path, get_user_by_query};
use crate::app::routes::yostar::login::{login_by_query, login_by_server, login_no_server};
use crate::app::routes::yostar::refresh::{refresh_by_query, refresh_by_server, refresh_no_server};
use crate::app::routes::yostar::send_code::{
    send_code_by_email, send_code_by_email_and_server, send_code_by_query,
};
use crate::app::state::{AppState, get_global_config, init_global_config};
use crate::core::authentication::{config::GlobalConfig, loaders};
use crate::core::local::handler::init_game_data_or_default;
use crate::database::pool::{create_pool, init_tables};
use crate::events::EventEmitter;
use crate::events::setup_event_listeners::setup_event_listeners;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn root() -> &'static str {
    "Myrtle API"
}

fn create_router(state: AppState) -> Router {
    let rate_store: RateLimitStore = Arc::new(RwLock::new(HashMap::new()));

    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/get-user", get(get_user_by_query))
        .route("/get-user/{uid}", get(get_user_by_path))
        .route("/send-code", post(send_code_by_query))
        .route("/send-code/{email}", post(send_code_by_email))
        .route(
            "/send-code/{email}/{server}",
            post(send_code_by_email_and_server),
        )
        .route("/login", post(login_by_query))
        .route("/login/{email}/{code}", post(login_no_server))
        .route("/login/{email}/{code}/{server}", post(login_by_server))
        .route("/refresh", post(refresh_by_query))
        .route("/refresh/{uid}/{secret}/{seqnum}", post(refresh_no_server))
        .route(
            "/refresh/{uid}/{secret}/{seqnum}/{server}",
            post(refresh_by_server),
        )
        .layer(middleware::from_fn_with_state(
            rate_store.clone(),
            rate_limit,
        ))
        .with_state(state)
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create HTTP client for config loading
    let client = Client::new();

    // Create global config
    let config = Arc::new(RwLock::new(GlobalConfig::new()));

    // Create event emitter
    let events = Arc::new(EventEmitter::new());

    // Setup event listeners BEFORE loading config
    setup_event_listeners(&events);

    // Initialize global accessor for cron jobs
    init_global_config(config.clone());

    // Load all configuration
    println!("Loading configuration...");
    loaders::init_all(&client, &config, &events).await;
    println!("Configuration loaded");

    // Create database pool
    let db = create_pool(&database_url).await?;

    // Initialize database tables
    init_tables(&db).await?;

    // Load game data.
    let data_dir = std::env::var("DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data"));

    println!("Loading game data from: {:?}", data_dir);

    let game_data = Arc::new(init_game_data_or_default(&data_dir));

    if game_data.is_loaded() {
        println!(
            "Game data loaded: {} operators, {} skills, {} modules, {} skins, {} items, {} handbook entries",
            game_data.operators.len(),
            game_data.skills.len(),
            game_data.modules.equip_dict.len(),
            game_data.skins.char_skins.len(),
            game_data.materials.items.len(),
            game_data.handbook.handbook_dict.len()
        );
    } else {
        println!("Warning: Running with empty game data");
    }

    // Create app state
    let state = AppState {
        db,
        config,
        events: events.clone(),
        client,
        game_data,
    };

    // Start cron jobs
    spawn_reload_job(state.client.clone(), events, 3600);

    // Create TCP listener
    let listener = TcpListener::bind("0.0.0.0:3060").await?;
    println!("Server running on http://0.0.0.0:3060");

    // Start server
    axum::serve(
        listener,
        create_router(state).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

fn spawn_reload_job(client: Client, events: Arc<EventEmitter>, interval_secs: u64) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));

        loop {
            interval.tick().await;
            let config = get_global_config();
            loaders::reload_all(&client, &config, &events).await;
            println!("Configuration reloaded");
        }
    });
}
