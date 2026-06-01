//! Precomputed stage bucketing for grading.
//!
//! Built once at gamedata load from stages + zones + activities.
//! Stages are split into two pools:
//!     - permanent: Mainline, `MainlineActivity`, `MainlineRetro`, Sidestory, Branchline, Campaign,
//!       and any zone whose id starts with "`permanent_sidestory`".
//!       `MainlineActivity` is treated as permanent because in the data it covers chapters that
//!       are permanent campaign content (e.g. ch.15/16); the matching `MainlineRetro` zones are
//!       currently empty, so the gameplay stages still live under the activity ID.
//!     - event: Activity zones (subject to recency decay).
//!       One-time competitive activity types (Contingency Contract, Boss Rush, Vector
//!       Breakthrough, etc. - see `is_excluded_activity_type`) are dropped entirely because
//!       they can't be cleared after they end and aren't rebroadcast.
//!       Rotating Annihilation maps (`camp_r_*`) are also routed here using their
//!       rotation window as the event window, so a map that rotated out long ago
//!       decays instead of dragging the grade as a permanent gap. The three
//!       permanent Annihilation maps (`camp_01/02/03`) stay in the permanent pool.

use serde::{Deserialize, Serialize};
use std::{cmp::Reverse, collections::HashMap};

use super::{
    activity::ActivityBasicInfo,
    campaign::CampaignRotations,
    stage::{Stage, StageDifficulty, StageType},
    zone::{Zone, ZoneType},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseEntry {
    pub stage_id: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEntry {
    pub stage_id: String,
    pub weight: f64,
    /// Activity `start_time` (unix seconds), if resolvable. Used to skip events that
    /// launched after a user's last sync so stale data isn't penalized.
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StageUniverse {
    pub permanent: Vec<UniverseEntry>,
    pub event: Vec<EventEntry>,
    pub permanent_max: f64,
}

impl StageUniverse {
    pub fn build(
        stages: &HashMap<String, Stage>,
        zones: &HashMap<String, Zone>,
        activities: &HashMap<String, ActivityBasicInfo>,
        campaign: &CampaignRotations,
    ) -> Self {
        let mut sorted_activities: Vec<&ActivityBasicInfo> = activities.values().collect();
        sorted_activities.sort_by_key(|a| Reverse(a.id.len()));

        let mut permanent: Vec<UniverseEntry> = Vec::new();
        let mut event: Vec<EventEntry> = Vec::new();

        for stage in stages.values() {
            if !include_stage(stage) {
                continue;
            }

            let Some(zone) = zones.get(&stage.zone_id) else {
                continue;
            };

            if matches!(zone.zone_type, ZoneType::Roguelike) {
                continue;
            }

            let Some(zone_weight) = zone_weight(&zone.zone_type) else {
                continue;
            };

            let weight = zone_weight * difficulty_multiplier(&stage.difficulty);

            if let Some(window) = campaign.window(&stage.stage_id) {
                event.push(EventEntry {
                    stage_id: stage.stage_id.clone(),
                    weight,
                    start_time: Some(window.start_ts),
                    end_time: Some(window.end_ts),
                });
                continue;
            }

            if is_permanent(&stage.zone_id, &zone.zone_type) {
                permanent.push(UniverseEntry {
                    stage_id: stage.stage_id.clone(),
                    weight,
                });
            } else {
                // Activity zones - event pool
                let activity = resolve_activity(&stage.zone_id, &sorted_activities);

                if let Some(act) = activity
                    && is_excluded_activity_type(&act.activity_type)
                {
                    continue;
                }

                let (start_time, end_time) = activity.map_or((None, None), |a| {
                    (
                        (a.start_time > 0).then_some(a.start_time),
                        (a.end_time > 0).then_some(a.end_time),
                    )
                });

                event.push(EventEntry {
                    stage_id: stage.stage_id.clone(),
                    weight,
                    start_time,
                    end_time,
                });
            }
        }

        let permanent_max: f64 = permanent.iter().map(|e| e.weight).sum();

        Self {
            permanent,
            event,
            permanent_max,
        }
    }
}

fn include_stage(stage: &Stage) -> bool {
    if stage.is_story_only {
        return false;
    }
    if matches!(
        stage.stage_type,
        StageType::Sub
            | StageType::Guide
            | StageType::SpecialStory
            | StageType::Daily
            | StageType::ClimbTower
    ) {
        return false;
    }
    let id = &stage.stage_id;
    if id.starts_with("tr_") || id.starts_with("wk_") || id.starts_with("tower_") {
        return false;
    }
    // #f# stages are first-time cutscene replays unlocked after main_02-08; they share
    // a level with their non-#f# counterpart and shouldn't double-count the chapter.
    // Stage_table also flags them FOUR_STAR for UI purposes, so they were inflating
    // their own weight by 1.25× on top of the duplication.
    if id.contains("#f#") {
        return false;
    }
    true
}

fn is_permanent(zone_id: &str, zone_type: &ZoneType) -> bool {
    if zone_id.starts_with("permanent_sidestory") {
        return true;
    }
    matches!(
        zone_type,
        ZoneType::Mainline
            | ZoneType::MainlineActivity
            | ZoneType::MainlineRetro
            | ZoneType::Sidestory
            | ZoneType::Branchline
            | ZoneType::Campaign
    )
}

const fn zone_weight(zone_type: &ZoneType) -> Option<f64> {
    match zone_type {
        ZoneType::Mainline => Some(1.0),
        ZoneType::Sidestory | ZoneType::Branchline | ZoneType::MainlineRetro => Some(0.85),
        ZoneType::Campaign => Some(0.7),
        ZoneType::Activity | ZoneType::MainlineActivity => Some(1.0),
        _ => None,
    }
}

const fn difficulty_multiplier(difficulty: &StageDifficulty) -> f64 {
    match difficulty {
        StageDifficulty::Normal => 1.0,
        StageDifficulty::FourStar => 1.25,
        StageDifficulty::SixStar => 1.5,
        StageDifficulty::Unknown => 1.0,
    }
}

fn resolve_activity<'a>(
    zone_id: &str,
    sorted: &[&'a ActivityBasicInfo],
) -> Option<&'a ActivityBasicInfo> {
    sorted
        .iter()
        .find(|act| !act.id.is_empty() && zone_id.starts_with(&act.id))
        .copied()
}

/// Activity `Type_` values for one-time / competitive content that can't be
/// cleared after the event ends and isn't rebroadcast. Including them in the
/// universe permanently penalizes anyone who didn't grind them at launch.
///
/// Categories represented: Contingency Contract / Multiplayer, Vector
/// Breakthrough, Boss Rush, Enemy Duel, Half-Idle, Auto-Chess, Arcade,
/// Float Parade, Team Quest, Collection events, Interlock, and the
/// Mainline Battle Pass standalone activity.
fn is_excluded_activity_type(activity_type: &str) -> bool {
    matches!(
        activity_type,
        "MULTIPLAY"
            | "MULTIPLAY_V3"
            | "MULTIPLAY_VERIFY2"
            | "VEC_BREAK"
            | "VEC_BREAK_V2"
            | "BOSS_RUSH"
            | "ENEMY_DUEL"
            | "HALFIDLE_VERIFY1"
            | "AUTOCHESS_VERIFY1"
            | "ARCADE"
            | "FLOAT_PARADE"
            | "TEAM_QUEST"
            | "COLLECTION"
            | "INTERLOCK"
            | "MAINLINE_BP"
            | "FIREWORK"
    )
}
