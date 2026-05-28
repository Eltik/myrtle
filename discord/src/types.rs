use std::collections::HashMap;
use tokio::sync::Mutex;

use crate::config::Config;

pub struct Data {
    pub command_counter: Mutex<HashMap<String, u64>>,
    pub config: Config,
    pub http_client: reqwest::Client,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
