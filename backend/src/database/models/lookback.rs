//! Account Lookback: one snapshot of how built an account was per period
//! (`user_lookback_snapshots`), and the owned-operator count rebuilt from
//! obtain times for the years before snapshots existed.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// The headline counts of one snapshot, `user_lookback_snapshots.summary`.
/// Every field defaults, so a row written before a field existed still reads.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LookbackSummary {
    /// Owned operators, one per Amiya form, as `user_operators` counts them.
    pub operators: u32,
    pub elite2: u32,
    /// At the level cap of their current promotion. An operator missing from
    /// the server's game data is not counted.
    pub max_level: u32,
    pub skill_level7: u32,
    /// Mastery ranks over every skill: an M3 counts 3.
    pub masteries: u32,
    /// Skills at Mastery 3.
    pub mastery3: u32,
    /// Unlocked modules, the default badge excluded.
    pub modules: u32,
    pub module_stage3: u32,
    /// At potential 6.
    pub max_potential: u32,
    /// Non-default skins owned.
    pub skins: u32,
    /// `user_scores` as of the snapshot: the grade that ran on that sync, or
    /// the previous one when it missed its budget. `None` when never graded.
    pub total_score: Option<f64>,
    pub grade: Option<String>,
}

/// One snapshot on the timeline, without its build.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize)]
pub struct LookbackSnapshot {
    /// `period_start` in Unix seconds: the snapshot's key on its account, and
    /// the `{id}` of `/lookback/snapshots/{id}`.
    #[ts(type = "number")]
    pub id: i64,
    pub period_start: DateTime<Utc>,
    /// When the sync that wrote it ran: the first sync of the period.
    pub taken_at: DateTime<Utc>,
    /// The event whose launch opened the period; `None` for a week period.
    pub activity_id: Option<String>,
    /// That event's name in the account server's game text, as of the snapshot.
    pub activity_name: Option<String>,
    pub summary: LookbackSummary,
}

/// One unlocked module of a [`LookbackOperator`].
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookbackModule {
    pub module_id: String,
    /// 1 to 3.
    pub stage: i16,
}

/// One operator's build in a snapshot.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookbackOperator {
    pub operator_id: String,
    pub elite: i16,
    pub level: i16,
    /// 0-based (0 = potential 1).
    pub potential: i16,
    pub skill_level: i16,
    /// Mastery per skill, by skill index. Empty when no skill is mastered.
    pub masteries: Vec<i16>,
    pub modules: Vec<LookbackModule>,
}

/// A whole snapshot: the counts and every operator's build.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize)]
pub struct LookbackBuild {
    /// When the build was read: the snapshot's sync, or now for the live build.
    pub taken_at: DateTime<Utc>,
    pub summary: LookbackSummary,
    pub operators: Vec<LookbackOperator>,
}

/// The count of operators owned at the end of one UTC day, rebuilt from their
/// obtain times. Ownership only: builds before the first snapshot are lost.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct OwnershipPoint {
    /// 00:00 UTC of the day, Unix seconds.
    #[ts(type = "number")]
    pub ts: i64,
    pub count: u32,
}

/// `GET /lookback`: the timeline.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize)]
pub struct LookbackOverview {
    /// Oldest first.
    pub snapshots: Vec<LookbackSnapshot>,
    /// One point per day an operator was obtained, oldest first. Reconstructed,
    /// not recorded: it counts today's roster by obtain time, and every Amiya
    /// form carries the base Amiya's time.
    pub ownership: Vec<OwnershipPoint>,
}

/// One operator as the snapshot builder reads it from the roster tables.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BuildRow {
    pub operator_id: String,
    pub elite: i16,
    pub level: i16,
    pub potential: i16,
    pub skill_level: i16,
    /// By skill index.
    pub masteries: Vec<i16>,
    /// Unlocked modules only, sorted by id; `module_levels` runs alongside.
    pub module_ids: Vec<String>,
    pub module_levels: Vec<i16>,
}

/// The stored form of one operator in `build`: an array, not an object, so
/// the key names are not repeated 236 times a row. Field order is the column
/// order the migration documents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredOperator(
    pub String,
    pub i16,
    pub i16,
    pub i16,
    pub i16,
    pub Vec<i16>,
    pub Vec<(String, i16)>,
);

/// `user_lookback_snapshots.build`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredBuild {
    #[serde(rename = "o")]
    pub operators: Vec<StoredOperator>,
}
