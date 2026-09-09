use serde::{Deserialize, Serialize};
use sqlx::types::{
    Uuid,
    chrono::{DateTime, Utc},
};
use ts_rs::TS;

/// `v_user_profile` view
#[derive(TS)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserProfile {
    pub id: Uuid,
    pub uid: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub level: Option<i16>,
    pub avatar_id: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
    pub resume_id: Option<String>,
    pub role: String,
    pub server: String, // FROM servers.code
    // Scores (LEFT JOIN)
    pub total_score: Option<f64>,
    pub grade: Option<String>,
    // Settings (LEFT JOIN)
    pub public_profile: Option<bool>,
    pub store_gacha: Option<bool>,
    pub share_stats: Option<bool>,
    // Status (LEFT JOIN)
    pub exp: Option<i32>,
    pub orundum: Option<i32>,
    pub lmd: Option<i32>,
    pub sanity: Option<i16>,
    pub max_sanity: Option<i16>,
    pub gacha_tickets: Option<i32>,
    pub ten_pull_tickets: Option<i32>,
    #[ts(type = "number | null")]
    pub monthly_sub_end: Option<i64>,
    #[ts(type = "number | null")]
    pub register_ts: Option<i64>,
    #[ts(type = "number | null")]
    pub last_online_ts: Option<i64>,
    pub resume: Option<String>,
    pub friend_num_limit: Option<i16>,
    pub cumulative_signin: Option<i32>,
    // Counts
    #[ts(type = "number | null")]
    pub operator_count: Option<i64>,
    #[ts(type = "number | null")]
    pub item_count: Option<i64>,
    #[ts(type = "number | null")]
    pub skin_count: Option<i64>,
    #[ts(type = "number | null")]
    pub non_default_skin_count: Option<i64>,
    pub updated_at: DateTime<Utc>,
}

/// users table
#[derive(TS)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// `user_settings` table
#[derive(TS)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UserSettings {
    pub user_id: Uuid,
    pub public_profile: bool,
    pub store_gacha: bool,
    pub share_stats: bool,
    pub updated_at: DateTime<Utc>,
}

/// `user_checkin` table — daily sign-in state from the game's `checkIn` section,
/// joined with the timestamps needed to render it as a calendar.
///
/// The game's monthly sign-in is a *sequential list of reward slots*, not a
/// dated calendar: missing a day leaves you one slot behind, it never forfeits
/// the slot. Nothing in this payload maps a claim to the date it happened on,
/// so no consumer can say "the 4th was missed" — only "N of the month's slots
/// are claimed, D days have elapsed".
#[derive(TS)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserCheckin {
    /// One flag per sign-in **claimed** this month, in claim order: `1` if the
    /// monthly-subscription Daily Supply came with that claim, `0` if not.
    ///
    /// This is the game's raw `checkInHistory`, and it is **not** a per-day
    /// calendar — a missed day produces no entry at all. Reading `flags[d - 1]`
    /// as "day `d` was claimed" is wrong, and counting the `1`s counts monthly
    /// card days, not sign-ins. (Verified against `user_status.monthly_sub_end`
    /// over the whole user table: the final flag agrees with subscription state
    /// in 2 193 of 2 219 non-empty rows.) Use [`Self::claimed_this_month`] for
    /// the sign-in count.
    #[sqlx(rename = "history")]
    pub monthly_card_flags: Vec<i16>,
    /// Sign-ins claimed in the current month's series — the length of
    /// `monthly_card_flags`, derived in SQL so no client re-derives it.
    pub claimed_this_month: i32,
    /// Lifetime cumulative sign-in days (the "total days of sign-ins" counter).
    pub cumulative_signin: i32,
    /// Active monthly sign-in series id (e.g. `signin<N>`).
    pub checkin_group_id: Option<String>,
    /// The game's raw `checkInRewardIndex`: a 0-based pointer into the month's
    /// reward slots. It equals `claimed_this_month` while a claim is pending
    /// and `claimed_this_month - 1` just after one, and a handful of rows carry
    /// a stale `0` across a series rollover — prefer `claimed_this_month`.
    pub reward_index: i16,
    /// Whether a daily sign-in is claimable right now (as of the last sync).
    pub can_check_in: bool,
    /// Account creation time (Unix seconds) — for "days since joining".
    #[ts(type = "number | null")]
    pub register_ts: Option<i64>,
    /// Player's last in-game online time (Unix seconds) — distinct from the DB
    /// sync time below.
    #[ts(type = "number | null")]
    pub last_online_ts: Option<i64>,
    /// When this row was last synced to our DB. Every field above is a snapshot
    /// as of this moment, which may be days/weeks before "now" — so the month
    /// the calendar belongs to is this timestamp's month, not today's.
    pub updated_at: DateTime<Utc>,
}
