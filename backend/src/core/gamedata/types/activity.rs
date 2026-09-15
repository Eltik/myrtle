//! Acitivty table types, used for event start/end times

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

use super::{
    material::Item,
    serde_helpers::{deserialize_fb_map, deserialize_fb_map_or_default},
    stage::{Stage, StageDifficulty},
};

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

    /// The event token shop's id on the game server (`shop_act54side`), the
    /// argument to `templateShop/getGoodList`. Absent when the activity has no
    /// token shop (sign-ins, logins, and the few events with a bespoke shop).
    #[serde(alias = "TemplateShopId", default)]
    pub template_shop_id: Option<String>,
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
    #[serde(default, deserialize_with = "deserialize_fb_map_or_default")]
    pub zone_to_activity: HashMap<String, String>,
    #[serde(default)]
    pub mission_data: Vec<ActivityMission>,
    #[serde(default, deserialize_with = "deserialize_fb_map_or_default")]
    pub activity_items: HashMap<String, Vec<String>>,
}

/// One event mission (`MissionData`). Only the template and its parameters
/// matter here: the stage-clear templates name a stage and the clear state
/// the mission asks for, and a player's mission record outlives the event.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ActivityMission {
    pub id: String,
    #[serde(default)]
    pub template: String,
    #[serde(default)]
    pub param: Vec<String>,
    #[serde(default)]
    pub rewards: Vec<MissionReward>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MissionReward {
    pub id: String,
    #[serde(default)]
    pub count: i32,
}

/// Event currency each activity's missions pay out, summed over
/// `MissionData` rewards whose item is one of the activity's tokens
/// (`ActivityItems`); a rerun has its own `_rep_1` tokens and missions.
pub fn mission_tokens_by_activity(
    missions: &[ActivityMission],
    activity_items: &HashMap<String, Vec<String>>,
) -> HashMap<String, i32> {
    let owner: HashMap<&str, &str> = activity_items
        .iter()
        .flat_map(|(act, items)| items.iter().map(move |i| (i.as_str(), act.as_str())))
        .collect();
    let mut out: HashMap<String, i32> = HashMap::new();
    for m in missions {
        for r in &m.rewards {
            if let Some(act) = owner.get(r.id.as_str()) {
                *out.entry((*act).to_string()).or_default() += r.count;
            }
        }
    }
    out
}

/// One stage that awards Originite Prime on first clear.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OpStage {
    pub stage_id: String,
    pub code: String,
    pub op: i32,
    pub challenge: bool,
}

/// The Originite Prime stages of each activity (`DiamondOnceDrop` over the
/// zones `ZoneToActivity` maps to it), in stage-table order. A rerun takes
/// over its original's zones, so an original with none borrows its `sre`
/// twin's list.
pub fn op_stages_by_activity(
    stages: &HashMap<String, Stage>,
    zone_to_activity: &HashMap<String, String>,
) -> HashMap<String, Vec<OpStage>> {
    let mut out: HashMap<String, Vec<OpStage>> = HashMap::new();
    for st in stages_in_order(stages) {
        if st.diamond_once_drop <= 0 {
            continue;
        }
        if let Some(act) = zone_to_activity.get(&st.zone_id) {
            out.entry(act.clone()).or_default().push(OpStage {
                stage_id: st.stage_id.clone(),
                code: st.code.clone(),
                op: st.diamond_once_drop,
                challenge: st.difficulty != StageDifficulty::Normal,
            });
        }
    }
    share_between_twins(&mut out);
    out
}

fn stages_in_order(stages: &HashMap<String, Stage>) -> Vec<&Stage> {
    let mut ordered: Vec<&Stage> = stages.values().collect();
    ordered.sort_by(|a, b| a.stage_id.cmp(&b.stage_id));
    ordered
}

/// A rerun (`actNNsre`) reuses its original's (`actNNside`) zones, so whichever
/// of the pair the table describes lends its list to the other.
fn share_between_twins<T: Clone>(map: &mut HashMap<String, Vec<T>>) {
    let borrowed: Vec<(String, Vec<T>)> = map
        .iter()
        .filter(|(_, list)| !list.is_empty())
        .filter_map(|(act, list)| {
            let twin = act
                .strip_suffix("side")
                .map(|s| format!("{s}sre"))
                .or_else(|| act.strip_suffix("sre").map(|s| format!("{s}side")))?;
            (!map.contains_key(&twin)).then(|| (twin, list.clone()))
        })
        .collect();
    map.extend(borrowed);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FarmDrop {
    pub item_id: String,
    pub name: String,
    pub name_en: Option<String>,
    pub icon_id: String,
    pub tier: u8,
    pub occ: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FarmStage {
    pub stage_id: String,
    pub code: String,
    pub ap_cost: i32,
    pub drops: Vec<FarmDrop>,
}

pub const FARM_STAGES: usize = 3;
pub const FARM_MIN_TIER: u8 = 3;

/// The last `FARM_STAGES` normal stages of each activity whose regular
/// drops include a material of tier `FARM_MIN_TIER` or better, with those
/// drops, in stage order. Mini events drop tier 2 and below and get none.
pub fn farm_stages_by_activity(
    stages: &HashMap<String, Stage>,
    zone_to_activity: &HashMap<String, String>,
    items: &HashMap<String, Item>,
) -> HashMap<String, Vec<FarmStage>> {
    let mut out: HashMap<String, Vec<FarmStage>> = HashMap::new();
    for st in stages_in_order(stages) {
        if st.difficulty != StageDifficulty::Normal || st.is_story_only {
            continue;
        }
        let Some(act) = zone_to_activity.get(&st.zone_id) else {
            continue;
        };
        let Some(info) = st.stage_drop_info.as_ref() else {
            continue;
        };
        let mut drops: Vec<FarmDrop> = info
            .display_detail_rewards
            .iter()
            .filter(|r| r.drop_type == "NORMAL" && r.item_type == "MATERIAL")
            .filter_map(|r| {
                let item = items.get(&r.id)?;
                Some(FarmDrop {
                    item_id: r.id.clone(),
                    name: item.name.clone(),
                    name_en: None,
                    icon_id: item.icon_id.clone(),
                    tier: item.rarity.tier(),
                    occ: r.occ_percent.clone(),
                })
            })
            .collect();
        if !drops.iter().any(|d| d.tier >= FARM_MIN_TIER) {
            continue;
        }
        drops.sort_by(|a, b| b.tier.cmp(&a.tier).then(a.item_id.cmp(&b.item_id)));
        drops.dedup_by(|a, b| a.item_id == b.item_id);
        out.entry(act.clone()).or_default().push(FarmStage {
            stage_id: st.stage_id.clone(),
            code: st.code.clone(),
            ap_cost: st.ap_cost,
            drops,
        });
    }
    for v in out.values_mut() {
        if v.len() > FARM_STAGES {
            v.drain(..v.len() - FARM_STAGES);
        }
    }
    share_between_twins(&mut out);
    out
}

pub const FARM_ARCHIVE: &str = "derived/farm-stages.json";

/// The client strips a stage's drop table once its event closes, so the
/// farming stages seen on any load are kept in `derived/farm-stages.json`
/// next to the extract and read back for events the current table no
/// longer describes; a live table always wins for the activities it has.
pub fn merge_farm_archive(
    data_dir: &std::path::Path,
    live: HashMap<String, Vec<FarmStage>>,
) -> HashMap<String, Vec<FarmStage>> {
    let Some(root) = data_dir.parent().and_then(std::path::Path::parent) else {
        return live;
    };
    let path = root.join(FARM_ARCHIVE);
    let mut merged: HashMap<String, Vec<FarmStage>> = std::fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    let before = merged.len();
    for (act, stages) in live {
        if !stages.is_empty() {
            merged.insert(act, stages);
        }
    }
    share_between_twins(&mut merged);
    if merged.len() != before
        && let Ok(json) = serde_json::to_string(&merged)
        && let Some(dir) = path.parent()
    {
        let _ = std::fs::create_dir_all(dir);
        let _ = std::fs::write(&path, json);
    }
    merged
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
