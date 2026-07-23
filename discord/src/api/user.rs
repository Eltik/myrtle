use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::api::CONFIG_TIMEOUT;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub uid: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub level: Option<i16>,
    pub avatar_id: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
    pub resume_id: Option<String>,
    pub role: String,
    pub server: String,
    pub total_score: Option<f64>,
    pub grade: Option<String>,
    pub public_profile: Option<bool>,
    pub store_gacha: Option<bool>,
    pub share_stats: Option<bool>,
    pub exp: Option<i32>,
    pub orundum: Option<i32>,
    pub lmd: Option<i32>,
    pub sanity: Option<i16>,
    pub max_sanity: Option<i16>,
    pub gacha_tickets: Option<i32>,
    pub ten_pull_tickets: Option<i32>,
    pub monthly_sub_end: Option<i64>,
    pub register_ts: Option<i64>,
    pub last_online_ts: Option<i64>,
    pub resume: Option<String>,
    pub friend_num_limit: Option<i16>,
    pub cumulative_signin: Option<i32>,
    pub operator_count: Option<i64>,
    pub item_count: Option<i64>,
    pub skin_count: Option<i64>,
    pub non_default_skin_count: Option<i64>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub uid: String,
    pub server_id: i16,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub level: Option<i16>,
    pub role: String,
    pub avatar_id: Option<String>,
    pub resume_id: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCheckin {
    pub history: Vec<i16>,
    pub cumulative_signin: i32,
    pub checkin_group_id: Option<String>,
    pub reward_index: i16,
    pub can_check_in: bool,
    pub register_ts: Option<i64>,
    pub last_online_ts: Option<i64>,
    pub updated_at: String,
}

pub async fn user(
    client: &Client,
    base_url: &str,
    id: &str,
) -> Result<UserProfile, reqwest::Error> {
    let url = format!("{base_url}/api/get-user?uid={id}");
    let response = client.get(&url).timeout(CONFIG_TIMEOUT).send().await?;
    response.json::<UserProfile>().await
}
