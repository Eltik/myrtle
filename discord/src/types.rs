use serenity::model::id::{GuildId, MessageId, UserId};
use sqlx::SqlitePool;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
    time::Instant,
};
use tokio::sync::{Mutex, RwLock};

use crate::config::Config;
use crate::db::AntiSpamPolicy;
use crate::watcher::AssetsState;

/// Per-`(guild, user)` rolling window of recent ping counts, keyed by send time.
///
/// Lives only in memory — antispam doesn't need to survive restarts, and a tiny deque per
/// user is cheaper than another DB roundtrip on every message.
pub type PingHistory = Arc<RwLock<HashMap<(GuildId, UserId), VecDeque<(Instant, u32)>>>>;

/// Cached antispam policies, mirroring `guild_max_ping`. Hydrated on startup and kept in
/// sync by `/antispam set` / `/antispam clear` so the hot path (every `MessageCreate` event)
/// stays out of `SQLite`.
pub type AntiSpamPolicies = Arc<RwLock<HashMap<GuildId, AntiSpamPolicy>>>;

pub struct Data {
    pub command_counter: Mutex<HashMap<String, u64>>,
    pub config: Config,
    pub http_client: reqwest::Client,
    pub pool: SqlitePool,
    pub tracked_messages: Arc<RwLock<HashSet<MessageId>>>,
    pub assets: Arc<AssetsState>,
    pub ping_history: PingHistory,
    pub antispam_policies: AntiSpamPolicies,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
