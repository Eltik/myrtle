//! Acitivty table types, used for event start/end times

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

use super::serde_helpers::deserialize_fb_map;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct ActivityBasicInfo {
    #[serde(alias = "Id")]
    pub id: String,

    #[serde(alias = "Name", default)]
    pub name: String,

    #[serde(alias = "StartTime", default)]
    #[ts(type = "number")]
    pub start_time: i64,

    #[serde(alias = "EndTime", default)]
    #[ts(type = "number")]
    pub end_time: i64,

    /// When event-medal rewards stop being claimable (often a few days after
    /// `end_time`); 0 if absent. Authoritative close time for event medals.
    #[serde(alias = "RewardEndTime", default)]
    #[ts(type = "number")]
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

/// Skin ids mentioned anywhere in `activity_table` (event rewards, drop
/// tables, hub fixtures), by a text scan of the file rather than a typed walk:
/// the table's shape changes every event and a `serde_json::Value` of the
/// whole 14 MB was a boot-memory peak once. Measured 2026-09-14: 100 ids on CN.
pub fn scan_skin_refs(raw: &str) -> std::collections::HashSet<String> {
    let re =
        regex::Regex::new(r#""(char_\d+_[A-Za-z0-9]+@[A-Za-z0-9_]+#\d+)""#).expect("static regex");
    re.captures_iter(raw).map(|c| c[1].to_string()).collect()
}

/// The loading illustration each activity's stages point at, keyed by
/// activity id (stage ids start with it: `act46side_01`). The generic screens
/// (`loading1`..`loading4`, `loadingE2`, `loadingS`) are not event art and are
/// skipped; among the rest the most-used picture wins. Measured 2026-09-14:
/// 94 of 151 CN stage activities resolve.
pub fn loading_pics_by_activity(
    stages: &HashMap<String, super::stage::Stage>,
    activities: &HashMap<String, ActivityBasicInfo>,
) -> HashMap<String, String> {
    let generic = |p: &str| {
        let tail = p.strip_prefix("loading").unwrap_or(p);
        tail.is_empty() || tail.chars().all(|c| c.is_ascii_digit()) || tail == "E2" || tail == "S"
    };
    let mut counts: HashMap<&str, HashMap<&str, usize>> = HashMap::new();
    for (stage_id, st) in stages {
        let Some(pic) = st
            .loading_pic_id
            .as_deref()
            .filter(|p| !p.is_empty() && !generic(p))
        else {
            continue;
        };
        let Some(act) = stage_id
            .split('_')
            .next()
            .filter(|a| activities.contains_key(*a))
        else {
            continue;
        };
        *counts.entry(act).or_default().entry(pic).or_default() += 1;
    }
    counts
        .into_iter()
        .filter_map(|(act, pics)| {
            pics.into_iter()
                .max_by_key(|(p, n)| (*n, std::cmp::Reverse((*p).to_string())))
                .map(|(p, _)| (act.to_string(), p.to_string()))
        })
        .collect()
}
