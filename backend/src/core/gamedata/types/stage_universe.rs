//! Precomputed stage bucketing for grading.
//!
//! Built once at gamedata load from stages + zones + activities.
//! Stages are split into two pools:
//!     - permanent: Mainline, Sidestory, Branchline, MainlineRetro, Campaign, and any zone whose id starts with "permanent_sidestory"
//!     - event: Activity and MainlineActivity zones (subject to recency decay)

use serde::{Deserialize, Serialize};
use std::{cmp::Reverse, collections::HashMap};

use super::{
    activity::ActivityBasicInfo,
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

            if is_permanent(&stage.zone_id, &zone.zone_type) {
                permanent.push(UniverseEntry {
                    stage_id: stage.stage_id.clone(),
                    weight,
                });
            } else {
                // Activity / MainlineActivity - event pool
                let end_time = resolve_end_time(&stage.zone_id, &sorted_activities);
                event.push(EventEntry {
                    stage_id: stage.stage_id.clone(),
                    weight,
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
    if id.starts_with("tr_")
        || id.starts_with("camp_")
        || id.starts_with("wk_")
        || id.starts_with("tower_")
    {
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
            | ZoneType::Sidestory
            | ZoneType::Branchline
            | ZoneType::MainlineRetro
            | ZoneType::Campaign
    )
}

fn zone_weight(zone_type: &ZoneType) -> Option<f64> {
    match zone_type {
        ZoneType::Mainline => Some(1.0),
        ZoneType::Sidestory | ZoneType::Branchline | ZoneType::MainlineRetro => Some(0.85),
        ZoneType::Campaign => Some(0.7),
        ZoneType::Activity | ZoneType::MainlineActivity => Some(1.0),
        _ => None,
    }
}

fn difficulty_multiplier(difficulty: &StageDifficulty) -> f64 {
    match difficulty {
        StageDifficulty::Normal => 1.0,
        StageDifficulty::FourStar => 1.25,
        StageDifficulty::SixStar => 1.5,
        StageDifficulty::Unknown => 1.0,
    }
}

fn resolve_end_time(zone_id: &str, sorted: &[&ActivityBasicInfo]) -> Option<i64> {
    for act in sorted {
        if !act.id.is_empty() && zone_id.starts_with(&act.id) && act.end_time > 0 {
            return Some(act.end_time);
        }
    }
    None
}
