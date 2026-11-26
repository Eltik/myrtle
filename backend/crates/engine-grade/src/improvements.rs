//! What would raise a player's grade most

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use myrtle_gamedata_raw::tables::campaign::{CampaignRotations, RotationStatus};
use myrtle_gamedata_raw::tables::medal::{MedalDefinition, Obtainability};
use myrtle_gamedata_raw::tables::operator::OperatorProfession;
use myrtle_gamedata_raw::tables::roguelike::RoguelikeThemeGameData;
use myrtle_gamedata_raw::tables::stage::Stage;
use myrtle_types::roster::RosterEntry;
use serde::Serialize;
use serde_json::Value;

use crate::medals::{UserMedalRow, is_medal_earned, rarity_weight};
use crate::operators::{
    ScoreDimension, ScoreModel, TRUST_MILESTONE_PCT, UpgradeDelta, advanced_module_levels,
    advanced_modules, is_graded, operator_score_breakdown_in, operator_upgrade_deltas,
    potential_matters, rarity_to_weight_in, total_roster_weight_in,
};
use crate::sandbox::score::{
    ACHIEVEMENT_WEIGHT, BASE_WEIGHT, CONTENT_WEIGHT, EXPLORATION_WEIGHT, QUEST_WEIGHT, TECH_WEIGHT,
};
use crate::sandbox::{SandboxGradeDetail, grade_sandbox_detail};
use crate::stages::{PlayerPools, PoolStage};
use crate::{GradeData, StageClears};

const MASTERY_MILESTONE: i16 = 3;
/// For operators that cannot master
const SKILL_LEVEL_MILESTONE: i16 = 7;
const MODULE_MILESTONE: i16 = 3;
/// Potential is 0-based
const POTENTIAL_MILESTONE_INDEX: i16 = 5;

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct StageImprovements {
    pub permanent: StagePoolImprovements,
    pub event: StagePoolImprovements,
}

#[derive(Debug, Clone, Serialize, Default, ts_rs::TS)]
#[ts(export)]
pub struct StagePoolImprovements {
    pub total: usize,
    pub cleared: usize,
    pub three_starred: usize,
    pub missing: Vec<StageGap>,
    pub not_three_starred: Vec<StageGap>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct StageGap {
    pub stage_id: String,
    pub code: String,
    pub name: Option<String>,
    pub zone_id: String,
    pub weight: f64,
    pub state: i16,
    /// Rotating Annihilation (`camp_r_*`) only
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<RotationInfo>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RotationInfo {
    /// `active`, `past` or `future`
    pub status: &'static str,
    #[ts(type = "number")]
    pub start_ts: i64,
    #[ts(type = "number")]
    pub end_ts: i64,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RoguelikeThemeImprovement {
    pub theme_id: String,
    pub theme_name: String,
    pub endings: ProgressPair,
    pub difficulty: RoguelikeDifficulty,
    pub collectibles: RoguelikeCollectibles,
    pub bp: ProgressPair,
    pub challenges: ProgressPair,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub special_endings: Option<Vec<RoguelikeSpecialEnding>>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RoguelikeSpecialEnding {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub mode_name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ProgressPair {
    pub current: usize,
    pub max: usize,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RoguelikeDifficulty {
    pub highest_cleared: i32,
    pub max: i32,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RoguelikeCollectibles {
    pub relics: ProgressPair,
    pub capsules: ProgressPair,
    pub bands: ProgressPair,
}

#[derive(Debug, Clone, Serialize, Default, ts_rs::TS)]
#[ts(export)]
pub struct SandboxImprovements {
    /// Equal-weight mean of each RA
    pub total: f64,
    pub topics: Vec<SandboxTopic>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct SandboxTopic {
    pub topic_id: String,
    pub name: String,
    pub weight: f64,
    pub total: f64,
    pub categories: Vec<SandboxCategory>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct SandboxCategory {
    pub key: &'static str,
    pub label: &'static str,
    pub weight: f64,
    pub score: f64,
    pub parts: Vec<SandboxPart>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct SandboxPart {
    pub label: &'static str,
    pub current: usize,
    pub max: usize,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct MedalImprovements {
    pub permanent_missing: Vec<MedalGap>,
    pub event_in_window_missing: Vec<MedalGap>,
    pub operator_locked: Vec<MedalGap>,
    pub unobtainable_missing: Vec<MedalGap>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct MedalGap {
    pub medal_id: String,
    pub name: String,
    pub rarity: String,
    pub get_method: String,
    pub description: String,
    pub is_hidden: bool,
    #[ts(type = "number | null")]
    pub end_time: Option<i64>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_lock: Option<MedalOperatorLock>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owned_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MedalOperatorLock {
    pub operator_id: String,
    pub operator_name: String,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct OperatorImprovements {
    pub score_breakdown: Vec<ScoreDimension>,
    pub below_milestone: Vec<OperatorGap>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct OperatorGap {
    pub operator_id: String,
    pub name: String,
    pub rarity: i16,
    pub current_elite: i16,
    pub current_level: i16,
    pub current_skill_level: i16,
    /// -1 without skills
    pub max_mastery: i16,
    /// -1 without a module
    pub max_module_level: i16,
    pub current_trust: f64,
    pub is_support: bool,
    pub missing: Vec<&'static str>,
    /// One per `missing` tag
    pub deltas: Vec<UpgradeDelta>,
    pub subscore_potential_gain: f64,
    pub total_potential_gain: f64,
}

#[must_use]
pub fn stage_improvements(
    data: &GradeData,
    stages: &HashMap<String, Stage>,
    rotations: &CampaignRotations,
    clears: &StageClears,
    known: &HashSet<String>,
    now: i64,
) -> StageImprovements {
    let pools = PlayerPools::new(
        &data.stage_universe,
        &clears.clears,
        Some(known),
        now,
        clears.last_synced_ts,
    );
    let rotation_for = |stage_id: &str| {
        let status = rotations.status(stage_id, now)?;
        let window = rotations.window(stage_id)?;
        Some(RotationInfo {
            status: match status {
                RotationStatus::Active => "active",
                RotationStatus::Past => "past",
                RotationStatus::Future => "future",
            },
            start_ts: window.start_ts,
            end_ts: window.end_ts,
        })
    };
    StageImprovements {
        permanent: stage_pool(pools.permanent(), stages, rotation_for),
        event: stage_pool(pools.event(), stages, rotation_for),
    }
}

pub fn stage_pool<'a>(
    pool_stages: impl Iterator<Item = PoolStage<'a>>,
    stages: &HashMap<String, Stage>,
    rotation_for: impl Fn(&str) -> Option<RotationInfo>,
) -> StagePoolImprovements {
    let mut pool = StagePoolImprovements::default();
    for stage in pool_stages {
        pool.total += 1;
        let meta = stages.get(stage.stage_id);
        let gap = StageGap {
            stage_id: stage.stage_id.to_owned(),
            code: meta.map(|s| s.code.clone()).unwrap_or_default(),
            name: meta.and_then(|s| s.name.clone()),
            zone_id: meta.map(|s| s.zone_id.clone()).unwrap_or_default(),
            weight: stage.weight,
            state: stage.state(),
            rotation: rotation_for(stage.stage_id),
        };
        if stage.is_three_starred() {
            pool.cleared += 1;
            pool.three_starred += 1;
        } else if stage.is_cleared() {
            pool.cleared += 1;
            pool.not_three_starred.push(gap);
        } else {
            pool.missing.push(gap);
        }
    }
    for gaps in [&mut pool.missing, &mut pool.not_three_starred] {
        gaps.sort_by(|a, b| {
            b.weight
                .partial_cmp(&a.weight)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.code.cmp(&b.code))
        });
    }
    pool
}

/// `progress` is `(theme_id, progress)`
#[must_use]
pub fn roguelike_improvements(
    data: &GradeData,
    progress: &[(String, Value)],
) -> Vec<RoguelikeThemeImprovement> {
    let by_theme: HashMap<&str, &Value> = progress.iter().map(|(t, p)| (t.as_str(), p)).collect();
    let mut themes: Vec<RoguelikeThemeImprovement> = data
        .roguelike
        .themes
        .iter()
        .map(|(theme_id, theme)| {
            theme_improvement(theme_id, theme, by_theme.get(theme_id.as_str()).copied())
        })
        .collect();
    themes.sort_by(|a, b| a.theme_id.cmp(&b.theme_id));
    themes
}

fn theme_improvement(
    theme_id: &str,
    theme: &RoguelikeThemeGameData,
    progress: Option<&Value>,
) -> RoguelikeThemeImprovement {
    let at = |path: &[&str]| path.iter().try_fold(progress?, |v, key| v.get(key));
    let len_at = |path: &[&str]| {
        at(path)
            .and_then(Value::as_object)
            .map_or(0, serde_json::Map::len)
    };
    let endings = at(&["record", "endingCnt"])
        .and_then(Value::as_object)
        .map_or(0, |modes| {
            modes
                .values()
                .filter_map(Value::as_object)
                .flat_map(serde_json::Map::keys)
                .collect::<HashSet<_>>()
                .len()
        });
    let highest_cleared = at(&["collect", "modeGrade", "NORMAL"])
        .and_then(Value::as_object)
        .and_then(|grades| {
            grades
                .iter()
                .filter(|(_, e)| e.get("state").and_then(Value::as_i64).unwrap_or(0) >= 2)
                .filter_map(|(grade, _)| grade.parse::<i32>().ok())
                .max()
        })
        .unwrap_or(-1);
    let collected = progress
        .map(|p| theme.count_collected(p))
        .unwrap_or_default();
    let pair = |current: usize, max: i32| ProgressPair {
        current,
        max: usize::try_from(max).unwrap_or(0),
    };
    RoguelikeThemeImprovement {
        theme_id: theme_id.to_owned(),
        theme_name: theme.theme_name.clone(),
        endings: pair(endings, theme.max_endings),
        difficulty: RoguelikeDifficulty {
            highest_cleared,
            max: theme.max_difficulty_grade,
        },
        collectibles: RoguelikeCollectibles {
            relics: pair(collected.relics, theme.max_relics),
            capsules: pair(collected.capsules, theme.max_capsules),
            bands: pair(collected.bands, theme.max_bands),
        },
        bp: pair(len_at(&["bp", "reward"]), theme.max_bp_levels),
        challenges: pair(len_at(&["challenge", "grade"]), theme.max_challenges),
        special_endings: (!theme.special_endings.is_empty()).then(|| {
            theme
                .special_endings
                .iter()
                .map(|e| RoguelikeSpecialEnding {
                    id: e.id.clone(),
                    name: e.name.clone(),
                    mode_name: e.mode_name.clone(),
                })
                .collect()
        }),
    }
}

#[must_use]
pub fn sandbox_improvements(data: &GradeData, progress: Option<&Value>) -> SandboxImprovements {
    let grade = grade_sandbox_detail(data, progress);
    let weight = 1.0 / grade.topics.len().max(1) as f64;
    SandboxImprovements {
        total: grade.total,
        topics: grade
            .topics
            .iter()
            .map(|d| SandboxTopic {
                topic_id: d.topic_id.clone().unwrap_or_default(),
                name: d.topic_name.clone(),
                weight,
                total: d.total,
                categories: sandbox_categories(d),
            })
            .collect(),
    }
}

fn sandbox_categories(d: &SandboxGradeDetail) -> Vec<SandboxCategory> {
    let part = |label, current, max| SandboxPart {
        label,
        current,
        max,
    };
    let base_parts = vec![
        part("Base level", d.base_level, d.base_level_max),
        if d.v3 {
            part("Band levels", d.band_levels, d.band_levels_max)
        } else {
            part("Blueprints", d.blueprints, d.blueprints_total)
        },
    ];
    let mut content_parts = vec![
        part("Recipes", d.recipes, d.recipes_total),
        part("Music", d.music, d.music_total),
    ];
    if !d.v3 {
        content_parts.push(part("Rifts cleared", d.rift_levels, d.rift_levels_max));
    }
    let category = |key, label, weight, score, parts| SandboxCategory {
        key,
        label,
        weight,
        score,
        parts,
    };
    vec![
        category(
            "achievements",
            "Achievements",
            ACHIEVEMENT_WEIGHT,
            d.achievements,
            vec![part(
                "Completed",
                d.achievements_completed,
                d.achievements_total,
            )],
        ),
        category(
            "exploration",
            "Exploration",
            EXPLORATION_WEIGHT,
            d.exploration,
            vec![
                part("Map nodes", d.nodes_explored, d.nodes_total),
                part("Zones", d.zones_unlocked, d.zones_total),
            ],
        ),
        category(
            "tech",
            "Tech unlocks",
            TECH_WEIGHT,
            d.tech_tree,
            vec![part("Unlocked", d.tech_unlocked, d.tech_total)],
        ),
        category(
            "quests",
            "Story acts",
            QUEST_WEIGHT,
            d.quests,
            vec![part("Completed", d.quests_completed, d.quests_total)],
        ),
        category(
            "base",
            "Base building",
            BASE_WEIGHT,
            d.base_building,
            base_parts,
        ),
        category(
            "content",
            "Content depth",
            CONTENT_WEIGHT,
            d.content_depth,
            content_parts,
        ),
    ]
}

#[must_use]
pub fn medal_improvements(
    data: &GradeData,
    medals: &[UserMedalRow],
    owned_operators: &HashSet<&str>,
    population: i64,
    owners: &HashMap<String, i64>,
    now: i64,
) -> MedalImprovements {
    let earned: HashSet<&str> = medals
        .iter()
        .filter(|(_, val, fts, rts)| {
            is_medal_earned(
                val.as_ref().unwrap_or(&Value::Null),
                fts.unwrap_or(0),
                rts.unwrap_or(0),
            )
        })
        .map(|(id, ..)| id.as_str())
        .collect();
    let gap = |medal: &MedalDefinition, end_time: Option<i64>| MedalGap {
        medal_id: medal.medal_id.clone(),
        name: medal.medal_name.clone(),
        rarity: medal.rarity.clone(),
        get_method: medal.get_method.clone(),
        description: data.medals.resolve_description(&medal.medal_id),
        is_hidden: medal.is_hidden,
        end_time,
        operator_lock: None,
        owned_pct: (population > 0).then(|| {
            let owned = owners.get(&medal.medal_id).copied().unwrap_or(0);
            (owned as f64 / population as f64 * 1000.0).round() / 10.0
        }),
    };

    let mut out = MedalImprovements {
        permanent_missing: Vec::new(),
        event_in_window_missing: Vec::new(),
        operator_locked: Vec::new(),
        unobtainable_missing: Vec::new(),
    };
    for medal in data.medals.medals.values() {
        if earned.contains(medal.medal_id.as_str()) {
            continue;
        }
        if let Some(lock) = data.medals.operator_lock(&medal.medal_id)
            && !owned_operators.contains(lock.operator_id.as_str())
        {
            out.operator_locked.push(MedalGap {
                operator_lock: Some(MedalOperatorLock {
                    operator_id: lock.operator_id.clone(),
                    operator_name: lock.operator_name.clone(),
                    reason: "collab",
                }),
                ..gap(medal, None)
            });
            continue;
        }
        match data.medals.obtainability(&medal.medal_id, now) {
            Obtainability::Permanent => out.permanent_missing.push(gap(medal, None)),
            Obtainability::Event { proxy_close_ts }
                if proxy_close_ts > 0 && proxy_close_ts < now =>
            {
                out.unobtainable_missing
                    .push(gap(medal, Some(proxy_close_ts)));
            }
            Obtainability::Event { proxy_close_ts } => out
                .event_in_window_missing
                .push(gap(medal, (proxy_close_ts > 0).then_some(proxy_close_ts))),
            Obtainability::NotYet => {}
            Obtainability::Unobtainable => out.unobtainable_missing.push(gap(medal, None)),
        }
    }

    out.permanent_missing.sort_by(by_rarity_desc);
    out.event_in_window_missing.sort_by(|a, b| {
        a.end_time
            .unwrap_or(i64::MAX)
            .cmp(&b.end_time.unwrap_or(i64::MAX))
            .then_with(|| by_rarity_desc(a, b))
    });
    out.operator_locked.sort_by(by_rarity_desc);
    out.unobtainable_missing.sort_by(|a, b| {
        b.end_time
            .unwrap_or(i64::MIN)
            .cmp(&a.end_time.unwrap_or(i64::MIN))
            .then_with(|| by_rarity_desc(a, b))
    });
    out
}

fn by_rarity_desc(a: &MedalGap, b: &MedalGap) -> Ordering {
    rarity_weight(&b.rarity)
        .partial_cmp(&rarity_weight(&a.rarity))
        .unwrap_or(Ordering::Equal)
        .then_with(|| a.medal_id.cmp(&b.medal_id))
}

#[must_use]
pub fn operator_improvements(
    roster: &[RosterEntry],
    data: &GradeData,
    support_ids: &HashSet<&str>,
    model: ScoreModel,
) -> OperatorImprovements {
    let total_weight = total_roster_weight_in(roster, data, model);
    let max_trust = data.favor.max_trust_pct();
    let mut below_milestone: Vec<OperatorGap> = Vec::new();
    for entry in roster {
        let Some(op) = data.operators.get(&entry.operator_id) else {
            continue;
        };
        if matches!(
            op.profession,
            OperatorProfession::Token | OperatorProfession::Trap
        ) || op.is_not_obtainable
            || !is_graded(entry, model)
        {
            continue;
        }

        let max_elite = i16::try_from(op.phases.len().saturating_sub(1)).unwrap_or(i16::MAX);
        let max_level_here = op
            .phases
            .get(usize::try_from(entry.elite).unwrap_or(0))
            .map_or(0, |p| i16::try_from(p.max_level).unwrap_or(i16::MAX));
        let num_skills = op.skills.len();
        let can_master = num_skills > 0 && op.phases.len() >= 3;
        let modules = advanced_modules(op);
        let max_mastery = mastery_levels(&entry.masteries).max().unwrap_or(-1);
        let max_module_level = advanced_module_levels(&entry.modules, &modules)
            .into_iter()
            .max()
            .unwrap_or(-1);
        let current_trust = data.favor.trust_pct(entry.favor_point);
        let is_support = support_ids.contains(entry.operator_id.as_str());
        let trust_target = if is_support {
            max_trust
        } else {
            TRUST_MILESTONE_PCT.min(max_trust)
        };

        let mut missing: Vec<&'static str> = Vec::new();
        if entry.elite < max_elite {
            missing.push("ELITE");
        }
        if max_level_here > 0 && entry.level < max_level_here {
            missing.push("MAX_LEVEL");
        }
        if can_master {
            if max_mastery < MASTERY_MILESTONE {
                missing.push("M3");
            }
        } else if num_skills > 0 && entry.skill_level < SKILL_LEVEL_MILESTONE {
            missing.push("SL7");
        }
        if !modules.is_empty() && max_module_level < MODULE_MILESTONE {
            missing.push("MOD3");
        }
        if potential_matters(op) && entry.potential < POTENTIAL_MILESTONE_INDEX {
            missing.push("POT6");
        }
        if trust_target > 0.0 && current_trust < trust_target {
            missing.push("TRUST");
        }
        if missing.is_empty() {
            continue;
        }

        let deltas = operator_upgrade_deltas(
            entry,
            op,
            &data.favor,
            is_support,
            &missing,
            rarity_to_weight_in(&op.rarity, model.legacy_rarity_weights),
            total_weight,
            model,
        );
        let (mut overlap, mut overlap_total) = (0.0_f64, 0.0_f64);
        let (mut independent, mut independent_total) = (0.0, 0.0);
        for d in &deltas {
            if d.tag == "ELITE" || d.tag == "MAX_LEVEL" {
                overlap = overlap.max(d.operator_grade_delta);
                overlap_total = overlap_total.max(d.total_score_delta);
            } else {
                independent += d.operator_grade_delta;
                independent_total += d.total_score_delta;
            }
        }

        below_milestone.push(OperatorGap {
            operator_id: entry.operator_id.clone(),
            name: op.name.clone(),
            rarity: op.rarity.to_star_int(),
            current_elite: entry.elite,
            current_level: entry.level,
            current_skill_level: entry.skill_level,
            max_mastery,
            max_module_level,
            current_trust,
            is_support,
            missing,
            deltas,
            subscore_potential_gain: overlap + independent,
            total_potential_gain: overlap_total + independent_total,
        });
    }

    below_milestone.sort_by(|a, b| {
        b.rarity
            .cmp(&a.rarity)
            .then_with(|| {
                b.total_potential_gain
                    .partial_cmp(&a.total_potential_gain)
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| a.operator_id.cmp(&b.operator_id))
    });
    OperatorImprovements {
        score_breakdown: operator_score_breakdown_in(roster, data, support_ids, model),
        below_milestone,
    }
}

fn mastery_levels(masteries: &Value) -> impl Iterator<Item = i16> + '_ {
    masteries
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|m| m.get("mastery").and_then(Value::as_i64))
        .map(|v| v as i16)
}
