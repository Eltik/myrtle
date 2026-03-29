use std::sync::Arc;

use redis::aio::ConnectionManager;
use sqlx::PgPool;

use crate::core::gamedata::{assets::AssetIndex, types::GameData};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub game_data: Arc<GameData>,
    pub asset_index: Arc<AssetIndex>,
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub fn new(
        db: PgPool,
        redis: ConnectionManager,
        game_data: GameData,
        asset_index: AssetIndex,
        config: AppConfig,
    ) -> Self {
        Self {
            db,
            redis,
            game_data: Arc::new(game_data),
            asset_index: Arc::new(asset_index),
            config: Arc::new(config),
        }
    }
}

pub struct AppConfig {
    pub jwt_secret: String,
    pub rate_limit_rpm: u32,
    pub service_key: String,
    pub assets_dir: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            rate_limit_rpm: std::env::var("RATE_LIMIT_RPM")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            service_key: std::env::var("SERVICE_KEY").expect("SERVICE_KEY must be set"),
            assets_dir: std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output".into()),
        }
    }
}
