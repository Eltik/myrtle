use std::sync::{Arc, OnceLock};

use reqwest::Client;
use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::{core::{authentication::config::GlobalConfig, local::types::GameData}, events::EventEmitter};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<RwLock<GlobalConfig>>,
    pub events: Arc<EventEmitter>,
    pub client: Client,
    pub game_data: Arc<GameData>
}

static GLOBAL_CONFIG: OnceLock<Arc<RwLock<GlobalConfig>>> = OnceLock::new();

pub fn init_global_config(config: Arc<RwLock<GlobalConfig>>) {
    GLOBAL_CONFIG
        .set(config)
        .expect("Global config already initialized");
}

pub fn get_global_config() -> Arc<RwLock<GlobalConfig>> {
    GLOBAL_CONFIG
        .get()
        .expect("Global config not initialized")
        .clone()
}
