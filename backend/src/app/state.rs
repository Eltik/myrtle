use std::ops::Deref;
use std::sync::Arc;

use arc_swap::ArcSwap;
use reqwest::Client;
use sqlx::PgPool;

use crate::app::cache::store::CacheStore;
use crate::core::gamedata::{assets::AssetIndex, types::GameData};

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

impl Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct AppStateInner {
    pub db: PgPool,
    pub cache: CacheStore,
    pub game_data: ArcSwap<GameData>,
    pub asset_index: ArcSwap<AssetIndex>,
    pub config: Arc<AppConfig>,
    pub http_client: Client,
}

impl AppState {
    pub fn new(
        db: PgPool,
        cache: CacheStore,
        game_data: GameData,
        asset_index: AssetIndex,
        config: AppConfig,
        client: Client,
    ) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                db,
                cache,
                game_data: ArcSwap::from_pointee(game_data),
                asset_index: ArcSwap::from_pointee(asset_index),
                config: Arc::new(config),
                http_client: client,
            }),
        }
    }

    pub fn swap_game_data(&self, new: GameData) {
        self.game_data.store(Arc::new(new));
    }

    pub fn swap_asset_index(&self, new: AssetIndex) {
        self.asset_index.store(Arc::new(new));
    }
}

pub struct AppConfig {
    pub jwt_secret: String,
    pub rate_limit_rpm: u32,
    pub service_key: String,
    pub assets_dir: String,
    pub game_data_dir: String,
    pub asset_ws_url: Option<String>,
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
            game_data_dir: std::env::var("GAME_DATA_DIR")
                .unwrap_or_else(|_| "../assets/output/gamedata/excel".into()),
            asset_ws_url: std::env::var("ASSET_WS_URL")
                .ok()
                .filter(|s| !s.is_empty() && s != "disabled"),
        }
    }
}
