//! Stage viewer index.
//!
//! A flat, display-ready catalogue of every browsable stage across all game
//! modes, precomputed at gamedata init. Unlike `stage_table` (which only covers
//! story / event / SSS / Annihilation content), this also folds in the
//! procedural modes whose stages live in separate tables and level files -
//! Integrated Strategies, Reclamation Algorithm, Contingency Contract and
//! Paradox Simulation - so the frontend's Stage List can populate every
//! category from a single resource. All taxonomy/labelling comes from
//! [`StageClassifier`], the single source of truth.
//!
//! [`StageClassifier`]: crate::core::gamedata::enrich::stage_class::StageClassifier

use serde::Serialize;

/// One browsable stage in the Stage List.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StageIndexEntry {
    /// `stage_table` id when this stage has one; otherwise the level file's
    /// relative id (procedural-mode nodes), so the value is always a stable key.
    pub stage_id: String,
    /// Relative level id (`obt/roguelike/...`) when known, for map viewing.
    pub level_id: Option<String>,
    /// Short in-game stage code (`1-7`, `LT-8`, IS/RA node label).
    pub code: String,
    /// Longer stage name, when distinct from the code.
    pub name: Option<String>,
    /// Grouping key the UI buckets under (event/chapter/season/mode id).
    pub zone_id: String,
    /// Resolved zone/event/season display name; `None` falls back to `zone_id`.
    pub zone_name: Option<String>,
    /// Sort key within a group: episode number for story, else the zone index.
    pub zone_order: i64,
    /// Fine group (`story` / `events` / `annihilation` / `is` / `ra` / `sss` /
    /// `paradox` / `cc` / `supplies` / `other`). Authoritative.
    pub group: String,
    /// Coarse bucket: `stages` / `events` / `modes`.
    pub category: String,
    /// Sanity cost, `0` for free / procedural nodes.
    pub ap_cost: i32,
    /// Marked as a boss stage in the stage table.
    pub boss: bool,
    /// Hard / Adverse / Challenge-Mode variant.
    pub is_hard: bool,
    /// `NORMAL` / `FOUR_STAR` / `SIX_STAR` (used by the UI to dedupe CM variants
    /// of the same code).
    pub difficulty: String,
    /// Whether the stage detail / map viewer can currently render it (true when
    /// the backing level file exists on disk).
    pub can_view: bool,
    /// Asset-relative path to this stage's map preview image (served under
    /// `/api/assets/`), resolved against disk at load. `None` if no art exists.
    pub preview: Option<String>,
    /// Asset-relative path to the zone/event/season banner key-art. `None` if
    /// none resolved (the UI falls back to a stage preview).
    pub banner: Option<String>,
}

/// The full stage catalogue. Built by
/// [`build_stage_index`](crate::core::gamedata::enrich::stage_index::build_stage_index).
pub type StageIndex = Vec<StageIndexEntry>;
