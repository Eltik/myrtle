use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Main Response
// ============================================================================

/// Full public statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsResponse {
    pub users: UserStats,
    pub gacha: GachaStats,
    pub game_data: GameDataStats,
    pub tier_lists: TierListStats,
    pub computed_at: String,
    pub cached: bool,
}

// ============================================================================
// User Statistics
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStats {
    /// Total registered users
    pub total: i64,
    /// Users by server (en, jp, cn, kr, tw)
    pub by_server: HashMap<String, i64>,
    /// New signups in the last 7 days
    pub recent_signups_7d: i64,
    /// New signups in the last 30 days
    pub recent_signups_30d: i64,
    /// Users with public profiles
    pub public_profiles: i64,
}

// ============================================================================
// Gacha Statistics (only from consenting users)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GachaStats {
    /// Total tracked pulls (from consenting users)
    pub total_pulls: i64,
    /// Number of users sharing stats
    pub contributing_users: i64,
    /// Pull rates by rarity
    pub pull_rates: PullRateStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRateStats {
    pub six_star_rate: f64,
    pub five_star_rate: f64,
    pub four_star_rate: f64,
    pub three_star_rate: f64,
}

// ============================================================================
// Game Data Statistics (from in-memory game data)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDataStats {
    /// Total operators in database
    pub operators: i64,
    /// Total skills
    pub skills: i64,
    /// Total modules
    pub modules: i64,
    /// Total skins
    pub skins: i64,
    /// Total materials/items
    pub items: i64,
    /// Total stages
    pub stages: i64,
    /// Total enemies
    pub enemies: i64,
    /// Active gacha pools
    pub gacha_pools: i64,
}

// ============================================================================
// Tier List Statistics
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TierListStats {
    /// Total tier lists
    pub total: i64,
    /// Active (published) tier lists
    pub active: i64,
    /// Total tier list versions published
    pub total_versions: i64,
    /// Total operator placements across all tier lists
    pub total_placements: i64,
    /// Community tier lists count
    pub community_count: i64,
}
