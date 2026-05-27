use std::collections::HashMap;
use tokio::sync::Mutex;

pub struct Data {
    pub command_counter: Mutex<HashMap<String, u64>>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
