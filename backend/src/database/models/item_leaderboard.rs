use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One row of the inventory leaderboard: a player and how much of the
/// requested item they hold. Identity fields mirror `LeaderboardEntry` so the
/// same row components render both.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemLeaderboardEntry {
    /// `rank()` over quantity among visible holders: ties share a rank.
    #[ts(type = "number")]
    pub rank: i64,
    pub uid: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub level: Option<i16>,
    pub avatar_id: Option<String>,
    pub server: String,
    /// The player's score grade, so a row reads the same whichever metric
    /// ranks it.
    pub grade: Option<String>,
    #[ts(type = "number")]
    pub quantity: i64,
}

/// How many visible players hold an item and the largest single holding.
/// Drives the item picker so it never offers an item nobody holds.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemHoldingSummary {
    pub item_id: String,
    #[ts(type = "number")]
    pub holders: i64,
    #[ts(type = "number")]
    pub top: i64,
}

/// One player's place on the leaderboard for one item.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemStanding {
    pub item_id: String,
    #[ts(type = "number")]
    pub quantity: i64,
    /// Rank among every visible holder, and that population's size.
    #[ts(type = "number")]
    pub rank_global: i64,
    #[ts(type = "number")]
    pub holders_global: i64,
    /// Rank among visible holders on the player's own server, and that
    /// population's size.
    #[ts(type = "number")]
    pub rank_server: i64,
    #[ts(type = "number")]
    pub holders_server: i64,
}
