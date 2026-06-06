use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::sandbox_universe::SandboxUniverse;
use crate::database::queries::sandbox::get_user_sandbox_progress;

use super::types::SandboxGradeDetail;

pub const ACHIEVEMENT_WEIGHT: f64 = 0.30;
pub const EXPLORATION_WEIGHT: f64 = 0.20;
pub const TECH_WEIGHT: f64 = 0.15;
pub const QUEST_WEIGHT: f64 = 0.15;
pub const BASE_WEIGHT: f64 = 0.10;
pub const CONTENT_WEIGHT: f64 = 0.10;

pub async fn grade_sandbox(
    pool: &PgPool,
    user_id: Uuid,
    game_data: &GameData,
) -> Result<f64, sqlx::Error> {
    grade_sandbox_detail(pool, user_id, game_data)
        .await
        .map(|d| d.total)
}

pub async fn grade_sandbox_detail(
    pool: &PgPool,
    user_id: Uuid,
    game_data: &GameData,
) -> Result<SandboxGradeDetail, sqlx::Error> {
    let progress = get_user_sandbox_progress(pool, user_id).await?;

    let universe = &game_data.sandbox_universe;

    let Some(sandbox) = progress.and_then(|p| extract_sandbox_v2(&p)) else {
        return Ok(SandboxGradeDetail {
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
        });
    };

    let (achievements, achievements_completed) = score_achievements(&sandbox, universe);
    let (exploration, nodes_explored, zones_unlocked) = score_exploration(&sandbox, universe);
    let (tech_tree, tech_unlocked) = score_tech(&sandbox, universe);
    let (quests, quests_completed) = score_quests(&sandbox, universe);
    let (base_building, base_level, blueprints) = score_base_building(&sandbox, universe);
    let (content_depth, recipes, music, rift_levels, rift_levels_max) =
        score_content_depth(&sandbox, universe);

    let total = (ACHIEVEMENT_WEIGHT * achievements
        + EXPLORATION_WEIGHT * exploration
        + TECH_WEIGHT * tech_tree
        + QUEST_WEIGHT * quests
        + BASE_WEIGHT * base_building
        + CONTENT_WEIGHT * content_depth)
        .clamp(0.0, 1.0);

    Ok(SandboxGradeDetail {
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
    })
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

/// A sandbox map node/zone counts as unlocked once its `state` reaches 1.
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

    let score = (ratio(nodes_explored, universe.max_nodes)
        + ratio(zones_unlocked, universe.max_zones))
        / 2.0;

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
    // `collect.complete.quest` is the player's completed Records/Archive quests
    // (`story_*` ids), the same set `max_quests` (`ArchiveQuestData`) counts.
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
        ((base_score + blueprint_score) / 2.0).min(1.0),
        base_lv,
        blueprints,
    )
}

/// Returns `(score, recipes, music, rift_levels, rift_levels_max)`. The rift
/// piece is the fixed rift dungeons cleared over the total; the raw
/// `(rift_levels, rift_levels_max)` are surfaced for the breakdown.
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

#[cfg(test)]
mod tests {
    use super::{score_content_depth, score_exploration};
    use crate::core::gamedata::types::sandbox_universe::SandboxUniverse;
    use serde_json::json;

    #[test]
    fn exploration_counts_zones_under_main_map_zone() {
        let sandbox = json!({
            "main": { "map": {
                "node": { "n1": {"state": 2}, "n2": {"state": 1}, "n3": {"state": 0} },
                "zone": { "z1": {"state": 1}, "z2": {"state": 1}, "z3": {"state": 0} },
            }},
        });
        let universe = SandboxUniverse {
            max_nodes: 3,
            max_zones: 3,
            ..Default::default()
        };
        let (_, nodes, zones) = score_exploration(&sandbox, &universe);
        assert_eq!(nodes, 2, "two unlocked nodes (state >= 1)");
        assert_eq!(zones, 2, "two unlocked zones, read from main.map.zone");
    }

    #[test]
    fn content_depth_counts_cleared_fixed_rifts() {
        let sandbox = json!({
            "riftInfo": {
                "fixFinish": ["fixed_dungeon_1", "fixed_dungeon_2"],
                "difficultyLvMax": { "hunt_dungeon_1": 1 },
            },
            "cook": { "book": {} },
            "collect": { "complete": { "music": [] } },
        });
        let universe = SandboxUniverse {
            max_rifts: 5,
            ..Default::default()
        };
        let (_, _, _, rift_levels, rift_levels_max) = score_content_depth(&sandbox, &universe);
        assert_eq!(rift_levels, 2, "two fixed rifts cleared (fixFinish length)");
        assert_eq!(rift_levels_max, 5, "out of the total fixed rifts");
    }
}
