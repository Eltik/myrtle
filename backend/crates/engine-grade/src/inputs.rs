use std::collections::{HashMap, HashSet};

use myrtle_types::{roster::RosterEntry, stage_clear::StageClear};

use crate::{medals::UserMedalRow, operators::ScoreModel};

#[derive(Debug, Clone, Default)]
pub struct StageClears {
    pub clears: HashMap<String, StageClear>,
    pub last_synced_ts: Option<i64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BaseScore {
    pub score: f64,
    pub utilization: f64,
    pub infrastructure: f64,
}

#[derive(Debug, Clone)]
pub struct GradeInputs {
    pub roster: Vec<RosterEntry>,
    /// Operator ids of the support units.
    pub supports: Vec<String>,
    pub base: BaseScore,
    /// `(theme_id, progress)`
    pub roguelike: Vec<(String, serde_json::Value)>,
    /// `(medal_id, val, first_ts, reach_ts)` rows from `user_medals`
    pub medals: Vec<UserMedalRow>,
    pub stage_clears: StageClears,
    pub known_stages: HashSet<String>,
    /// `user_sandbox_progress` JSON, `None` when the user has none
    pub sandbox: Option<serde_json::Value>,
    pub now_unix: i64,
    pub score_model: ScoreModel,
}
