use axum::middleware;
use axum::routing::{delete, get, post, put};
use axum::{Router, response::Json};
use reqwest::Client;
use serde::Serialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use crate::app::middleware::rate_limit::{RateLimitStore, rate_limit};
use crate::app::middleware::static_assets::serve_asset;
use crate::app::routes::admin;
use crate::app::routes::auth::update_settings::update_settings;
use crate::app::routes::auth::verify::verify_token;
use crate::app::routes::avatar::serve_avatar;
use crate::app::routes::dps_calculator::{calculate_dps, list_operators};
use crate::app::routes::get_user::{get_user_by_path, get_user_by_query};
use crate::app::routes::leaderboard::get_leaderboard;
use crate::app::routes::portrait::serve_portrait;
use crate::app::routes::search::search_users;
use crate::app::routes::static_data;
use crate::app::routes::tier_lists;
use crate::app::routes::yostar::login::{login_by_query, login_by_server, login_no_server};
use crate::app::routes::yostar::refresh::{refresh_by_query, refresh_by_server, refresh_no_server};
use crate::app::routes::yostar::send_code::{
    send_code_by_email, send_code_by_email_and_server, send_code_by_query,
};
use crate::app::state::{AppState, get_global_config, init_global_config};
use crate::core::authentication::{config::GlobalConfig, loaders};
use crate::core::local::handler::init_game_data_or_default;
use crate::core::s3::AssetSource;
use crate::database::pool::{create_pool, init_tables};
use crate::events::setup_event_listeners::setup_event_listeners;
use crate::events::{ConfigEvent, EventEmitter, GameDataStats};

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

    // Use the asset source from state for CDN routes
    let cdn_router = Router::new()
        .route("/cdn/{*asset_path}", get(serve_asset))
        .with_state(state.asset_source.clone());

    let static_router = static_data::router();

    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/cdn/avatar/{avatar_id}", get(serve_avatar))
        .route("/cdn/portrait/{char_id}", get(serve_portrait))
        .route("/get-user", get(get_user_by_query))
        .route("/get-user/{uid}", get(get_user_by_path))
        .route("/leaderboard", get(get_leaderboard))
        .route("/search", get(search_users))
        .route("/send-code", post(send_code_by_query))
        .route("/send-code/{email}", post(send_code_by_email))
        .route(
            "/send-code/{email}/{server}",
            post(send_code_by_email_and_server),
        )
        .route("/auth/verify", post(verify_token))
        .route("/auth/update-settings", post(update_settings))
        .route("/login", post(login_by_query))
        .route("/login/{email}/{code}", post(login_no_server))
        .route("/login/{email}/{code}/{server}", post(login_by_server))
        .route("/refresh", post(refresh_by_query))
        .route("/refresh/{uid}/{secret}/{seqnum}", post(refresh_no_server))
        .route(
            "/refresh/{uid}/{secret}/{seqnum}/{server}",
            post(refresh_by_server),
        )
        .nest("/static", static_router)
        // DPS Calculator
        .route("/dps-calculator", post(calculate_dps))
        .route("/dps-calculator/operators", get(list_operators))
        // Tier lists
        .route("/tier-lists", get(tier_lists::list::list_tier_lists))
        .route("/tier-lists", post(tier_lists::create::create_tier_list))
        .route("/tier-lists/{slug}", get(tier_lists::get::get_tier_list))
        .route(
            "/tier-lists/{slug}",
            put(tier_lists::update::update_tier_list),
        )
        .route(
            "/tier-lists/{slug}",
            delete(tier_lists::update::delete_tier_list),
        )
        // Tier list tiers
        .route(
            "/tier-lists/{slug}/tiers",
            get(tier_lists::tiers::list_tiers),
        )
        .route(
            "/tier-lists/{slug}/tiers",
            post(tier_lists::tiers::create_tier),
        )
        .route(
            "/tier-lists/{slug}/tiers/{tier_id}",
            put(tier_lists::tiers::update_tier),
        )
        .route(
            "/tier-lists/{slug}/tiers/{tier_id}",
            delete(tier_lists::tiers::delete_tier),
        )
        .route(
            "/tier-lists/{slug}/tiers/reorder",
            post(tier_lists::tiers::reorder_tiers),
        )
        // Tier list placements
        .route(
            "/tier-lists/{slug}/placements",
            get(tier_lists::placements::list_placements),
        )
        .route(
            "/tier-lists/{slug}/placements",
            post(tier_lists::placements::add_placement),
        )
        .route(
            "/tier-lists/{slug}/placements/{placement_id}",
            put(tier_lists::placements::update_placement),
        )
        .route(
            "/tier-lists/{slug}/placements/{placement_id}",
            delete(tier_lists::placements::remove_placement),
        )
        .route(
            "/tier-lists/{slug}/placements/{placement_id}/move",
            post(tier_lists::placements::move_placement),
        )
        // Tier list versions
        .route(
            "/tier-lists/{slug}/versions",
            get(tier_lists::versions::list_versions),
        )
        .route(
            "/tier-lists/{slug}/versions/{version}",
            get(tier_lists::versions::get_version),
        )
        .route(
            "/tier-lists/{slug}/changelog",
            get(tier_lists::versions::get_changelog),
        )
        .route(
            "/tier-lists/{slug}/operator/{operator_id}/history",
            get(tier_lists::versions::get_operator_history),
        )
        .route(
            "/tier-lists/{slug}/publish",
            post(tier_lists::versions::publish_version),
        )
        // Tier list permissions
        .route(
            "/tier-lists/{slug}/permissions",
            get(tier_lists::permissions::list_permissions),
        )
        .route(
            "/tier-lists/{slug}/permissions",
            post(tier_lists::permissions::grant_permission),
        )
        .route(
            "/tier-lists/{slug}/permissions/{user_id}/{permission}",
            delete(tier_lists::permissions::revoke_permission),
        )
        // Community tier list specific
        .route("/tier-lists/mine", get(tier_lists::mine::list_mine))
        .route(
            "/tier-lists/{slug}/report",
            post(tier_lists::report::report_tier_list),
        )
        // Admin routes
        .route("/admin/stats", get(admin::stats::get_stats))
        // Admin tier list moderation
        .route(
            "/admin/tier-lists/reports",
            get(tier_lists::report::list_reports),
        )
        .route(
            "/admin/tier-lists/reports/{report_id}/review",
            post(tier_lists::report::review_report),
        )
        .route(
            "/admin/tier-lists/{slug}/moderate",
            post(tier_lists::moderate::moderate_tier_list),
        )
        .layer(middleware::from_fn_with_state(
            (rate_store.clone(), state.clone()),
            rate_limit,
        ))
        .with_state(state)
        .merge(cdn_router)
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

    // Emit server starting event
    events.emit(ConfigEvent::ServerStarting);

    // Load all configuration
    events.emit(ConfigEvent::ConfigLoadStarted);
    loaders::init_all(&client, &config, &events).await;
    events.emit(ConfigEvent::ConfigLoadComplete);

    // Create database pool
    events.emit(ConfigEvent::DatabaseConnecting);
    let db = create_pool(&database_url).await?;
    events.emit(ConfigEvent::DatabaseConnected);

    // Initialize database tables
    init_tables(&db).await?;
    events.emit(ConfigEvent::DatabaseTablesInitialized);

    // Load game data
    let data_dir = std::env::var("DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data"));

    let assets_dir = std::env::var("ASSETS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("assets"));

    events.emit(ConfigEvent::GameDataLoadStarted {
        data_dir: data_dir.display().to_string(),
        assets_dir: assets_dir.display().to_string(),
    });

    let game_data = Arc::new(init_game_data_or_default(&data_dir, &assets_dir, &events));

    if game_data.is_loaded() {
        events.emit(ConfigEvent::GameDataLoaded(GameDataStats {
            operators: game_data.operators.len(),
            skills: game_data.skills.len(),
            modules: game_data.modules.equip_dict.len(),
            skins: game_data.skins.char_skins.len(),
            items: game_data.materials.items.len(),
            handbook_entries: game_data.handbook.handbook_dict.len(),
            chibis: game_data.chibis.characters.len(),
            ranges: game_data.ranges.len(),
            gacha_pools: game_data.gacha.gacha_pool_client.len(),
            voices: game_data.voices.char_words.len(),
            zones: game_data.zones.len(),
            stages: game_data.stages.len(),
        }));
    } else {
        events.emit(ConfigEvent::GameDataEmpty);
    }

    events.emit(ConfigEvent::RedisConnecting);
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = redis::Client::open(redis_url)?;
    let redis = redis_client.get_multiplexed_async_connection().await?;
    events.emit(ConfigEvent::RedisConnected);

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let internal_service_key = std::env::var("INTERNAL_SERVICE_KEY").ok();

    // Initialize asset source (supports local, S3, or hybrid mode)
    let asset_source = AssetSource::from_env().unwrap_or_else(|e| {
        events.emit(ConfigEvent::AssetSourceError(e.clone()));
        AssetSource::local_only(assets_dir.clone())
    });

    // Log asset source mode
    if asset_source.has_s3() {
        events.emit(ConfigEvent::AssetSourceS3Enabled {
            mode: format!("{:?}", asset_source.mode()),
        });
    }

    // Create app state
    let state = AppState {
        db,
        config,
        events: events.clone(),
        client,
        game_data,
        redis,
        jwt_secret,
        internal_service_key,
        asset_source,
    };

    // Start cron jobs
    spawn_reload_job(state.client.clone(), events.clone(), 3600);

    // Create TCP listener
    let listener = TcpListener::bind("0.0.0.0:3060").await?;
    events.emit(ConfigEvent::ServerStarted { port: 3060 });

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
            events.emit(ConfigEvent::ConfigReloaded);
        }
    });
}
