//! Acitivty table types, used for event start/end times

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::serde_helpers::deserialize_fb_map;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityBasicInfo {
    #[serde(alias = "Id")]
    pub id: String,

    #[serde(alias = "Name", default)]
    pub name: String,

    #[serde(alias = "StartTime", default)]
    pub start_time: i64,

    #[serde(alias = "EndTime", default)]
    pub end_time: i64,

    /// When event-medal rewards stop being claimable (often a few days after
    /// `end_time`); 0 if absent. Authoritative close time for event medals.
    #[serde(alias = "RewardEndTime", default)]
    pub reward_end_time: i64,

    /// Group id tying this activity's medals together (e.g.
    /// `medalGroupActivity46side`). Empty when the activity has no medals.
    #[serde(alias = "MedalGroupId", default)]
    pub medal_group_id: String,

    #[serde(alias = "HasStage", default)]
    pub has_stage: bool,

    #[serde(alias = "IsReplicate", default)]
    pub is_replicate: bool,

    /// Activity category (e.g. `TYPE_ACT46SIDE`, `MULTIPLAY_V3`, `BOSS_RUSH`).
    /// Used by the stage universe to filter out one-time competitive events
    /// (Contingency Contract, Boss Rush, Vector Breakthrough, etc.) that can't
    /// be cleared after they end and aren't rebroadcast.
    #[serde(alias = "Type_", rename = "type", default)]
    pub activity_type: String,
}

impl ActivityBasicInfo {
    /// One-time competitive / minigame events (Contingency Contract / multiplayer,
    /// Vector Breakthrough, Boss Rush, Enemy Duel, Auto-Chess, Half-Idle, Arcade,
    /// etc.) that can't be replayed after they end and aren't rebroadcast.
    ///
    /// The single source of truth shared by the stage universe (which drops these
    /// stages entirely) and medal scoring (which marks their medals unobtainable
    /// rather than recency-decayed) - both to avoid permanently penalizing players
    /// who never had a chance to grind a one-off competitive mode.
    pub fn is_one_time_competitive(&self) -> bool {
        matches!(
            self.activity_type.as_str(),
            "MULTIPLAY"
                | "MULTIPLAY_V3"
                | "MULTIPLAY_VERIFY2"
                | "VEC_BREAK"
                | "VEC_BREAK_V2"
                | "BOSS_RUSH"
                | "ENEMY_DUEL"
                | "HALFIDLE_VERIFY1"
                | "AUTOCHESS_VERIFY1"
                | "AUTOCHESS_SEASON"
                | "ARCADE"
                | "FLOAT_PARADE"
                | "TEAM_QUEST"
                | "COLLECTION"
                | "INTERLOCK"
                | "MAINLINE_BP"
                | "FIREWORK"
        )
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ActivityTableFile {
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub basic_info: HashMap<String, ActivityBasicInfo>,
    // Don't need rest of data
}
