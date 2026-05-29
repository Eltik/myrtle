use serenity::model::id::MessageId;
use sqlx::SqlitePool;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::{Mutex, RwLock};

use crate::config::Config;
use crate::watcher::AssetsState;

pub struct Data {
    pub command_counter: Mutex<HashMap<String, u64>>,
    pub config: Config,
    pub http_client: reqwest::Client,
    pub pool: SqlitePool,
    pub tracked_messages: Arc<RwLock<HashSet<MessageId>>>,
    pub assets: Arc<AssetsState>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
