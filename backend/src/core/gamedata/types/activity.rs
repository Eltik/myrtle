//! Acitivty table types, used for event start/end times

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use ts_rs::TS;

use super::{
    material::Item,
    serde_helpers::{deserialize_fb_map, deserialize_fb_map_or_default},
    stage::{Stage, StageDifficulty},
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
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

    /// The Archives shelf the game files the event under: `SIDESTORY`,
    /// `BRANCHLINE` (Intermezzi), `MINISTORY` (Vignettes) or `NONE`. On EN
    /// 3 of the 50 `ACTIVITY_STORY` groups are `BRANCHLINE` (act9d0, act18d0,
    /// act18d3); the story reader shows it beside the derived category.
    #[serde(alias = "DisplayType", default)]
    pub display_type: String,

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
    /// Il Siracusano's hub table. Only its task rings and battle tasks are read;
    /// everything else in it (areas, opera, char cards) belongs to the event UI.
    #[serde(default)]
    pub siracusa_data: SiracusaData,
}

/// The two maps of the Il Siracusano hub that decide whether a stage is
/// reachable: the rings group tasks under a logic type, and a battle task
/// names the stage that clears it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SiracusaData {
    #[serde(default, deserialize_with = "deserialize_fb_map_or_default")]
    pub task_ring_map: HashMap<String, TaskRing>,
    #[serde(default, deserialize_with = "deserialize_fb_map_or_default")]
    pub battle_task_map: HashMap<String, BattleTask>,
}

/// One hub ring. `logic_type` is `LINEAR`, `AND` or `OR`; on the 2026-09-22
/// tables the 49 rings are 43 LINEAR, 3 AND and 3 OR, on both EN and CN.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TaskRing {
    #[serde(default)]
    pub logic_type: String,
    #[serde(default)]
    pub task_id_list: Vec<String>,
}

/// One hub task cleared by playing a stage. The rest of the hub's tasks are
/// story (AVG) tasks and name no stage.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BattleTask {
    #[serde(default)]
    pub stage_id: String,
}

impl ActivityTableFile {
    /// Stages a player can only reach by taking one arm of an either/or choice.
    /// Picking the other arm locks this one for good, so the stage can never be
    /// a gap: not owning it says nothing about the player.
    ///
    /// This is the single place either/or hub mechanics get derived. If another
    /// event ships a similar hub table, add its derivation here, keyed on that
    /// table's own logic type, never on stage codes or id suffixes.
    ///
    /// Measured on the 2026-09-22 EN and CN tables: exactly `{act21side_06_m}`,
    /// out of 49 rings that are 43 LINEAR, 3 AND and 3 OR on both servers. Ring
    /// `taskRing_Texas_4` is the only OR ring holding a battle task; it pairs
    /// `act21side_06_m` with a story task, and 910 of the 1,451 local users who
    /// finished Il Siracusano hold `06_m` at state 0. The `_m`/`_t` suffix is
    /// not the discriminator: the linear siblings were cleared by 1,195 to
    /// 1,588 of 2,630 local users against 549 for `06_m`.
    pub fn optional_stage_ids(&self) -> HashSet<String> {
        self.siracusa_data
            .task_ring_map
            .values()
            .filter(|ring| ring.logic_type == "OR")
            .flat_map(|ring| &ring.task_id_list)
            .filter_map(|task_id| self.siracusa_data.battle_task_map.get(task_id))
            .filter(|task| !task.stage_id.is_empty())
            .map(|task| task.stage_id.clone())
            .collect()
    }
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
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

/// Write `bytes` to `path` without ever truncating the existing file in place.
///
/// The farm archive is irreplaceable, so a torn write is worse than no write. The
/// temp file is removed if either step fails, and the caller logs. No fsync: this
/// guarantees "never a half-written archive", not durability across a crash.
fn write_atomic_archive(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut tmp_name = path.as_os_str().to_os_string();
    tmp_name.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp_name);
    if let Err(e) = std::fs::write(&tmp, bytes) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

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

    // "No archive yet" and "could not read the archive" are NOT the same thing, and
    // collapsing them with .ok()/.unwrap_or_default() is what made one failed read
    // destructive: an empty map means `before` is 0, every live activity counts as
    // new, and the write below then replaces an archive of closed events with only
    // the handful currently open. This file is the only copy of drop tables the
    // client has already stripped, so that loss cannot be re-derived from anywhere.
    //
    // On any read or parse failure, keep the file and serve degraded: the caller
    // gets the live activities alone for this load, and the next successful load
    // restores the merge.
    let mut merged: HashMap<String, Vec<FarmStage>> = match std::fs::read_to_string(&path) {
        Ok(raw) => match serde_json::from_str(&raw) {
            Ok(parsed) => parsed,
            Err(e) => {
                tracing::error!(
                    path = %path.display(),
                    error = %e,
                    "farm archive did not parse; keeping it and serving live activities only"
                );
                return live;
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => HashMap::new(),
        Err(e) => {
            tracing::error!(
                path = %path.display(),
                error = %e,
                "farm archive could not be read; keeping it and serving live activities only"
            );
            return live;
        }
    };
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
        // Temp-then-rename, and the result is checked. `std::fs::write` truncates
        // at open, so a stalled write here leaves a zero-length archive that the
        // NEXT load reads as "no events" and overwrites again, turning one bad
        // write into permanent loss. Discarding the error also meant the operator
        // had no way to know the archive had stopped being written.
        if let Err(e) =
            std::fs::create_dir_all(dir).and_then(|()| write_atomic_archive(&path, json.as_bytes()))
        {
            tracing::error!(
                path = %path.display(),
                error = %e,
                "failed to persist the farm archive; the previous copy is intact"
            );
        }
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
