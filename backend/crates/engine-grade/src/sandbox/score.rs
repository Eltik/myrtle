use myrtle_gamedata_raw::tables::sandbox_universe::SandboxUniverse;
use serde_json::Value;

use crate::GradeData;

use super::types::SandboxGradeDetail;

pub const ACHIEVEMENT_WEIGHT: f64 = 0.30;
pub const EXPLORATION_WEIGHT: f64 = 0.20;
pub const TECH_WEIGHT: f64 = 0.15;
pub const QUEST_WEIGHT: f64 = 0.15;
pub const BASE_WEIGHT: f64 = 0.10;
pub const CONTENT_WEIGHT: f64 = 0.10;

#[must_use]
pub fn grade_sandbox(game_data: &GradeData, progress: Option<&Value>) -> f64 {
    grade_sandbox_detail(game_data, progress).total
}

pub fn grade_sandbox_detail(game_data: &GradeData, progress: Option<&Value>) -> SandboxGradeDetail {
    let universe = &game_data.sandbox_universe;

    let Some(sandbox) = progress.and_then(extract_sandbox_v2) else {
        return SandboxGradeDetail {
            achievements_total: universe.max_achievements,
            nodes_total: universe.max_nodes,
            zones_total: universe.max_zones,
            tech_total: universe.max_tech_nodes,
            quests_total: universe.max_quests,
            base_level_max: universe.max_base_level,
            blueprints_total: universe.max_blueprints,
            recipes_total: universe.max_recipes,
            music_total: universe.max_music,
            ..SandboxGradeDetail::default()
        };
    };

    let (achievements, achievements_completed) = score_achievements(&sandbox, universe);
    let (exploration, nodes_explored, zones_unlocked) = score_exploration(&sandbox, universe);
    let (tech_tree, tech_unlocked) = score_tech(&sandbox, universe);
    let (quests, quests_completed) = score_quests(&sandbox, universe);
    let (base_building, base_level, blueprints) = score_base_building(&sandbox, universe);
    let (content_depth, recipes, music, rift_levels, rift_levels_max) =
        score_content_depth(&sandbox, universe);

    let total = (BASE_WEIGHT.mul_add(base_building, QUEST_WEIGHT.mul_add(quests, TECH_WEIGHT.mul_add(tech_tree, EXPLORATION_WEIGHT.mul_add(exploration, ACHIEVEMENT_WEIGHT * achievements))))
        + CONTENT_WEIGHT * content_depth)
        .clamp(0.0, 1.0);

    SandboxGradeDetail {
        total,
        achievements,
        exploration,
        tech_tree,
        quests,
        base_building,
        content_depth,
        achievements_completed,
        achievements_total: universe.max_achievements,
        nodes_explored,
        nodes_total: universe.max_nodes,
        zones_unlocked,
        zones_total: universe.max_zones,
        tech_unlocked,
        tech_total: universe.max_tech_nodes,
        quests_completed,
        quests_total: universe.max_quests,
        base_level,
        base_level_max: universe.max_base_level,
        blueprints,
        blueprints_total: universe.max_blueprints,
        recipes,
        recipes_total: universe.max_recipes,
        music,
        music_total: universe.max_music,
        rift_levels,
        rift_levels_max,
    }
}

fn extract_sandbox_v2(progress: &Value) -> Option<Value> {
    progress
        .get("template")?
        .get("SANDBOX_V2")?
        .get("sandbox_1")
        .cloned()
}

fn ratio(num: usize, den: usize) -> f64 {
    if den == 0 {
        return 0.0;
    }
    (num as f64 / den as f64).min(1.0)
}

fn score_achievements(sandbox: &Value, universe: &SandboxUniverse) -> (f64, usize) {
    let completed = sandbox
        .get("collect")
        .and_then(|c| c.get("complete"))
        .and_then(|c| c.get("achievement"))
        .and_then(|a| a.as_array())
        .map_or(0, std::vec::Vec::len);

    (ratio(completed, universe.max_achievements), completed)
}

fn is_unlocked(v: &Value) -> bool {
    v.get("state")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0)
        >= 1
}

fn score_exploration(sandbox: &Value, universe: &SandboxUniverse) -> (f64, usize, usize) {
    let nodes_explored = sandbox
        .get("main")
        .and_then(|m| m.get("map"))
        .and_then(|m| m.get("node"))
        .and_then(|n| n.as_object())
        .map_or(0, |obj| obj.values().filter(|v| is_unlocked(v)).count());

    let zones_unlocked = sandbox
        .get("main")
        .and_then(|m| m.get("map"))
        .and_then(|m| m.get("zone"))
        .and_then(|z| z.as_object())
        .map_or(0, |obj| obj.values().filter(|v| is_unlocked(v)).count());

    let score = f64::midpoint(ratio(nodes_explored, universe.max_nodes), ratio(zones_unlocked, universe.max_zones));

    (score.min(1.0), nodes_explored, zones_unlocked)
}

fn score_tech(sandbox: &Value, universe: &SandboxUniverse) -> (f64, usize) {
    let unlocked = sandbox
        .get("tech")
        .and_then(|t| t.get("unlock"))
        .and_then(|u| u.as_array())
        .map_or(0, std::vec::Vec::len);

    (ratio(unlocked, universe.max_tech_nodes), unlocked)
}

fn score_quests(sandbox: &Value, universe: &SandboxUniverse) -> (f64, usize) {
    let completed = sandbox
        .get("collect")
        .and_then(|c| c.get("complete"))
        .and_then(|c| c.get("quest"))
        .and_then(|q| q.as_array())
        .map_or(0, std::vec::Vec::len);

    (ratio(completed, universe.max_quests), completed)
}

fn score_base_building(sandbox: &Value, universe: &SandboxUniverse) -> (f64, usize, usize) {
    let base_lv = sandbox
        .get("base")
        .and_then(|b| b.get("baseLv"))
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0) as usize;

    let blueprints = sandbox
        .get("build")
        .and_then(|b| b.get("book"))
        .and_then(|b| b.as_object())
        .map_or(0, serde_json::Map::len);

    let base_score = ratio(base_lv, universe.max_base_level);
    let blueprint_score = ratio(blueprints, universe.max_blueprints);

    (
        f64::midpoint(base_score, blueprint_score).min(1.0),
        base_lv,
        blueprints,
    )
}

fn score_content_depth(
    sandbox: &Value,
    universe: &SandboxUniverse,
) -> (f64, usize, usize, usize, usize) {
    let recipes = sandbox
        .get("cook")
        .and_then(|c| c.get("book"))
        .and_then(|b| b.as_object())
        .map_or(0, serde_json::Map::len);

    let music = sandbox
        .get("collect")
        .and_then(|c| c.get("complete"))
        .and_then(|c| c.get("music"))
        .and_then(|m| m.as_array())
        .map_or(0, std::vec::Vec::len);

    let rift_levels = sandbox
        .get("riftInfo")
        .and_then(|r| r.get("fixFinish"))
        .and_then(|f| f.as_array())
        .map_or(0, std::vec::Vec::len);
    let rift_levels_max = universe.max_rifts;
    let rift_score = ratio(rift_levels, rift_levels_max);

    let recipe_score = ratio(recipes, universe.max_recipes);
    let music_score = ratio(music, universe.max_music);

    (
        ((recipe_score + music_score + rift_score) / 3.0).min(1.0),
        recipes,
        music,
        rift_levels,
        rift_levels_max,
    )
}
