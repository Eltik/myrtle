use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
};

use myrtle_gamedata_raw::tables::{
    activity::ActivityBasicInfo,
    campaign::CampaignRotations,
    stage::{Stage, StageDifficulty, StageType},
    zone::{Zone, ZoneType},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseEntry {
    pub stage_id: String,
    pub weight: f64,
    #[serde(default)]
    pub optional: bool,
    /// Whether currently playable
    #[serde(default)]
    pub live_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEntry {
    pub stage_id: String,
    pub weight: f64,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub is_permanent: bool,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub live_only: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StageUniverse {
    pub permanent: Vec<UniverseEntry>,
    pub event: Vec<EventEntry>,
    pub permanent_max: f64,
}

impl StageUniverse {
    #[must_use]
    pub fn build(
        stages: &HashMap<String, Stage>,
        zones: &HashMap<String, Zone>,
        activities: &HashMap<String, ActivityBasicInfo>,
        campaign: &CampaignRotations,
        retro_linked_acts: &HashSet<String>,
        optional: &HashSet<String>,
        live_only: &HashSet<String>,
    ) -> Self {
        let mut sorted_activities: Vec<&ActivityBasicInfo> = activities.values().collect();
        sorted_activities.sort_by_key(|a| Reverse(a.id.len()));

        let mut permanent: Vec<UniverseEntry> = Vec::new();
        let mut event: Vec<EventEntry> = Vec::new();

        let mut sorted_stages: Vec<&Stage> = stages.values().collect();
        sorted_stages.sort_unstable_by(|a, b| a.stage_id.cmp(&b.stage_id));

        for stage in sorted_stages {
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
            let is_optional = optional.contains(&stage.stage_id);
            let is_live_only = live_only.contains(&stage.stage_id);

            if let Some(window) = campaign.window(&stage.stage_id) {
                event.push(EventEntry {
                    stage_id: stage.stage_id.clone(),
                    weight,
                    start_time: Some(window.start_ts),
                    end_time: Some(window.end_ts),
                    is_permanent: false,
                    optional: is_optional,
                    live_only: is_live_only,
                });
                continue;
            }

            if is_permanent(&stage.zone_id, &zone.zone_type, &stage.stage_type) {
                permanent.push(UniverseEntry {
                    stage_id: stage.stage_id.clone(),
                    weight,
                    optional: is_optional,
                    live_only: is_live_only,
                });
            } else {
                let activity = resolve_activity(&stage.zone_id, &sorted_activities)
                    .or_else(|| resolve_activity(&stage.stage_id, &sorted_activities));

                if let Some(act) = activity
                    && act.is_one_time_competitive()
                {
                    continue;
                }

                let (start_time, end_time) = activity.map_or((None, None), |a| {
                    (
                        (a.start_time > 0).then_some(a.start_time),
                        (a.end_time > 0).then_some(a.end_time),
                    )
                });

                let is_permanent = activity.is_some_and(|a| retro_linked_acts.contains(&a.id));

                event.push(EventEntry {
                    stage_id: stage.stage_id.clone(),
                    weight,
                    start_time,
                    end_time,
                    is_permanent,
                    optional: is_optional,
                    live_only: is_live_only,
                });
            }
        }

        let permanent_max: f64 = permanent
            .iter()
            .filter(|e| !e.optional && !e.live_only)
            .map(|e| e.weight)
            .sum();

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
    // #f# stages are first-time cutscene replays unlocked after main_02-08.
    // they share a level with their non-#f# counterpart and shouldn't double-count the chapter.
    // Stage_table also flags them FOUR_STAR for UI purposes
    if id.contains("#f#") {
        return false;
    }
    true
}

fn is_permanent(zone_id: &str, zone_type: &ZoneType, stage_type: &StageType) -> bool {
    if zone_id.starts_with("permanent_sidestory") {
        return true;
    }
    if *stage_type == StageType::Activity {
        return false;
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
