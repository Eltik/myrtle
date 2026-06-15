//! Enemy -> stages inverted index.
//!
//! Maps each enemy id to the list of stages it appears in, precomputed at
//! gamedata init by parsing the per-stage level files under
//! `gamedata/levels/`. Serves the `/static/enemy-stages` resource so the
//! frontend can answer "where does this enemy show up?" without fetching
//! hundreds of level files itself.

use serde::Serialize;
use std::collections::HashMap;

/// A single appearance of an enemy in a stage.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnemyStageRef {
    pub stage_id: String,
    /// Human-readable code, e.g. "0-1", "WD-8".
    pub code: String,
    pub zone_id: String,
    pub stage_name: Option<String>,
    /// True for hard-mode / Adverse variants (tough levels, 4★/6★ difficulty).
    pub is_hard: bool,
    /// Total spawned across every wave (sum of SPAWN action counts). 0 means
    /// the enemy is declared for the stage but not directly spawned
    /// (summoned, conditional, or a relative-spawn parent).
    pub count: u32,
}

/// `enemy_id -> appearances`. Built by [`build_enemy_stage_index`].
///
/// [`build_enemy_stage_index`]: crate::core::gamedata::enrich::enemy_stages::build_enemy_stage_index
pub type EnemyStageIndex = HashMap<String, Vec<EnemyStageRef>>;
