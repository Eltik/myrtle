use std::collections::{HashMap, HashSet};

use std::str::from_utf8;

use serde::{Deserialize, Serialize};

use super::material::Item;
use super::mission::MissionTableFile;
use super::stage::{Stage, StageDifficulty};
use crate::serde_helpers::{deserialize_fb_map, deserialize_fb_map_or_default};

#[derive(Debug, Default)]
pub struct ActivityTables {
    /// `activity_table`
    pub activities: ActivityTableFile,
    /// `activity_table`, `MissionArchives` only
    pub mission_archives: MissionArchiveTableFile,
    /// Skin ids `activity_table` mentions anywhere, see [`scan_skin_refs`]
    pub skin_refs: HashSet<String>,
    /// `mission_table`
    pub missions: MissionTableFile,
}

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

    /// When event-medal rewards stop being claimable, 0 if absent
    #[serde(alias = "RewardEndTime", default)]
    pub reward_end_time: i64,

    /// Group id tying medals together (e.g. `medalGroupActivity46side`)
    #[serde(alias = "MedalGroupId", default)]
    pub medal_group_id: String,

    #[serde(alias = "HasStage", default)]
    pub has_stage: bool,

    #[serde(alias = "IsReplicate", default)]
    pub is_replicate: bool,

    /// Activity category (`TYPE_ACT46SIDE`, `MULTIPLAY_V3`, `BOSS_RUSH`)
    #[serde(alias = "Type_", rename = "type", default)]
    pub activity_type: String,

    /// Overall category: `SIDESTORY`, `BRANCHLINE` (Intermezzi), `MINISTORY` (Vignettes) or `NONE`
    #[serde(alias = "DisplayType", default)]
    pub display_type: String,

    /// Event token shop's id on game server (`shop_act54side`), used in `templateShop/getGoodList`
    #[serde(alias = "TemplateShopId", default)]
    pub template_shop_id: Option<String>,
}

impl ActivityBasicInfo {
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
    /// Il Siracusano's hub table, only its task rings and battle tasks
    #[serde(default)]
    pub siracusa_data: SiracusaData,
}

/// Il Siracusano hub maps that decide whether a stage is reachable
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SiracusaData {
    #[serde(default, deserialize_with = "deserialize_fb_map_or_default")]
    pub task_ring_map: HashMap<String, TaskRing>,
    #[serde(default, deserialize_with = "deserialize_fb_map_or_default")]
    pub battle_task_map: HashMap<String, BattleTask>,
}

/// One hub ring, `logic_type` is `LINEAR`, `AND` or `OR`
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TaskRing {
    #[serde(default)]
    pub logic_type: String,
    #[serde(default)]
    pub task_id_list: Vec<String>,
}

/// Hub task cleared by playing a stage
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BattleTask {
    #[serde(default)]
    pub stage_id: String,
}

impl ActivityTableFile {
    /// Stages on one arm of an either/or choice (`OR` rings), locked for good by the other arm
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

    /// Stages that only existed while the event ran (the 14 `IS-QT` stages)
    pub fn live_only_stage_ids(&self) -> HashSet<String> {
        self.siracusa_data
            .battle_task_map
            .values()
            .filter(|task| !task.stage_id.is_empty())
            .map(|task| task.stage_id.clone())
            .collect()
    }
}

/// Event mission (`MissionData`)
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

/// Event currency each activity's missions pay out, keyed by activity id
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

/// Stage that awards Originite Prime on first clear
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpStage {
    pub stage_id: String,
    pub code: String,
    pub op: i32,
    pub challenge: bool,
}

/// Originite Prime stages of each activity, in stage order
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

/// Lend each `actNNside` / `actNNsre` list to its twin when the twin has none
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FarmDrop {
    pub item_id: String,
    pub name: String,
    pub name_en: Option<String>,
    pub icon_id: String,
    pub tier: u8,
    pub occ: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FarmStage {
    pub stage_id: String,
    pub code: String,
    pub ap_cost: i32,
    pub drops: Vec<FarmDrop>,
}

pub const FARM_STAGES: usize = 3;
pub const FARM_MIN_TIER: u8 = 3;

/// Last `FARM_STAGES` normal stages of each activity dropping a tier `FARM_MIN_TIER`+ material
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

/// `activity_table.MissionArchives`, voice clips an act unlocks as its missions clear
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MissionArchiveTableFile {
    #[serde(
        alias = "MissionArchives",
        deserialize_with = "deserialize_fb_map_or_default",
        default
    )]
    pub mission_archives: HashMap<String, MissionArchive>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MissionArchive {
    #[serde(alias = "TopicId", default)]
    pub topic_id: String,
    #[serde(alias = "UnlockDesc", default)]
    pub unlock_desc: String,
    /// Zones the shelf belongs to; `main_14` is also the story group's id
    #[serde(alias = "Zones", default)]
    pub zones: Vec<String>,
    #[serde(alias = "Nodes", default)]
    pub nodes: Vec<MissionArchiveNode>,
    #[serde(alias = "HiddenClips", default)]
    pub hidden_clips: Vec<MissionArchiveClip>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MissionArchiveNode {
    #[serde(alias = "NodeId", default)]
    pub node_id: String,
    #[serde(alias = "Title", default)]
    pub title: String,
    #[serde(alias = "UnlockDesc", default)]
    pub unlock_desc: String,
    #[serde(alias = "Clips", default)]
    pub clips: Vec<MissionArchiveClip>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MissionArchiveClip {
    #[serde(alias = "CharId", default)]
    pub char_id: String,
    #[serde(alias = "VoiceId", default)]
    pub voice_id: String,
    #[serde(alias = "Index", default)]
    pub index: i32,
}

/// Skin ids (`char_002_amiya@test#1`) that appear as whole JSON strings in `raw`
pub fn scan_skin_refs(raw: &[u8]) -> HashSet<String> {
    let mut refs = HashSet::new();
    let mut i = 0;
    while let Some(off) = raw[i..].iter().position(|&b| b == b'"') {
        let start = i + off + 1;
        match skin_ref_len(&raw[start..]) {
            Some(len) if raw.get(start + len) == Some(&b'"') => {
                if let Ok(id) = from_utf8(&raw[start..start + len]) {
                    refs.insert(id.to_owned());
                }
                i = start + len + 1;
            }
            _ => i = start,
        }
    }
    refs
}

/// Length of the skin id at the start of `s`, if one is there
fn skin_ref_len(s: &[u8]) -> Option<usize> {
    /// End of the non-empty run of `pred` bytes starting at `s[at]`
    fn run(s: &[u8], at: usize, pred: impl Fn(u8) -> bool) -> Option<usize> {
        let n = s.get(at..)?.iter().take_while(|&&b| pred(b)).count();
        (n > 0).then_some(at + n)
    }
    fn expect(s: &[u8], at: usize, byte: u8) -> Option<usize> {
        (s.get(at) == Some(&byte)).then_some(at + 1)
    }

    let at = s.starts_with(b"char_").then_some(5)?;
    let at = run(s, at, |b| b.is_ascii_digit())?;
    let at = expect(s, at, b'_')?;
    let at = run(s, at, |b| b.is_ascii_alphanumeric())?;
    let at = expect(s, at, b'@')?;
    let at = run(s, at, |b| b.is_ascii_alphanumeric() || b == b'_')?;
    let at = expect(s, at, b'#')?;
    run(s, at, |b| b.is_ascii_digit())
}
