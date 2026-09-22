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

/// How many visible players hold an item, the largest single holding, and
/// how much of it they hold between them. Drives the item picker so it
/// never offers an item nobody holds.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemHoldingSummary {
    pub item_id: String,
    #[ts(type = "number")]
    pub holders: i64,
    #[ts(type = "number")]
    pub top: i64,
    /// Every visible holding of the item summed: how much of it exists
    /// across the ranked population.
    #[ts(type = "number")]
    pub total_quantity: i64,
}

/// The population behind one item page: how many visible players hold the
/// item and how much of it they hold between them. Not exported; the page
/// carries these as its own fields.
#[derive(Debug, Clone, Copy, sqlx::FromRow)]
pub struct HoldingTotals {
    pub holders: i64,
    pub quantity: i64,
}

/// The item picker's data: every held item, and the population the holder
/// counts are cut from, so a row can say what share of players hold it.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemCatalog {
    /// Visible players under the same gate and server filter as `holders`,
    /// so `holders / population` is a share of the same people.
    #[ts(type = "number")]
    pub population: i64,
    /// Ordered by holders, most first.
    pub items: Vec<ItemHoldingSummary>,
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
