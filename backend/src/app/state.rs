use std::sync::{Arc, OnceLock};

use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::{core::authentication::config::GlobalConfig, events::EventEmitter};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<RwLock<GlobalConfig>>,
    pub events: Arc<EventEmitter>,
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
