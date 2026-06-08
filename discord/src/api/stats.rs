use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::api::CONFIG_TIMEOUT;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsResponse {
    pub users: StatsUsers,
    pub gacha: StatsGacha,
    pub game_data: StatsGameData,
    pub tier_lists: StatsTierLists,
    pub rosters: StatsRosters,
    pub computed_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsUsers {
    pub total: i32,
    pub by_server: StatsByServer,
    pub signups7d: i32,
    pub signups30d: i32,
    pub public_profiles: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsByServer {
    pub en: i32,
    pub jp: i32,
    pub kr: i32,
    pub cn: i32,
    pub bili: i32,
    pub tw: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsGacha {
    pub total_pulls: i32,
    pub contributing_users: i32,
    pub six_star_count: i32,
    pub five_star_count: i32,
    pub four_star_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsGameData {
    pub operators: i32,
    pub skills: i32,
    pub modules: i32,
    pub skins: i32,
    pub stages: i32,
    pub zones: i32,
    pub enemies: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsTierLists {
    pub total: i32,
    pub active: i32,
    pub total_versions: i32,
    pub total_placements: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsRosters {
    pub total: i32,
}

pub async fn stats(client: &Client, base_url: &str) -> Result<StatsResponse, reqwest::Error> {
    let url = format!("{base_url}/api/stats");
    let response = client.get(&url).timeout(CONFIG_TIMEOUT).send().await?;
    response.json::<StatsResponse>().await
}
