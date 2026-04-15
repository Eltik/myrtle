use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::sandbox_universe::SandboxUniverse;
use crate::database::queries::sandbox::get_user_sandbox_progress;

use super::types::SandboxGradeDetail;

const ACHIEVEMENT_WEIGHT: f64 = 0.30;
const EXPLORATION_WEIGHT: f64 = 0.20;
const TECH_WEIGHT: f64 = 0.15;
const QUEST_WEIGHT: f64 = 0.15;
const BASE_WEIGHT: f64 = 0.10;
const CONTENT_WEIGHT: f64 = 0.10;

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

    let Some(sandbox) = progress.and_then(|p| extract_sandbox_v2(&p)) else {
        return Ok(SandboxGradeDetail::default());
    };

    let universe = &game_data.sandbox_universe;

    let (achievements, achievements_completed) = score_achievements(&sandbox, universe);
    let (exploration, nodes_explored) = score_exploration(&sandbox, universe);
    let (tech_tree, tech_unlocked) = score_tech(&sandbox, universe);
    let (quests, quests_completed) = score_quests(&sandbox, universe);
    let base_building = score_base_building(&sandbox, universe);
    let content_depth = score_content_depth(&sandbox, universe);

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
        tech_unlocked,
        tech_total: universe.max_tech_nodes,
        quests_completed,
        quests_total: universe.max_quests,
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
        .map(|a| a.len())
        .unwrap_or(0);

    (ratio(completed, universe.max_achievements), completed)
}

fn score_exploration(sandbox: &Value, universe: &SandboxUniverse) -> (f64, usize) {
    let nodes_explored = sandbox
        .get("main")
        .and_then(|m| m.get("map"))
        .and_then(|m| m.get("node"))
        .and_then(|n| n.as_object())
        .map(|obj| {
            obj.values()
                .filter(|v| v.get("state").and_then(|s| s.as_i64()).unwrap_or(0) >= 1)
                .count()
        })
        .unwrap_or(0);

    let zones_unlocked = sandbox
        .get("main")
        .and_then(|m| m.get("zone"))
        .and_then(|z| z.as_object())
        .map(|obj| {
            obj.values()
                .filter(|v| v.get("state").and_then(|s| s.as_i64()).unwrap_or(0) >= 1)
                .count()
        })
        .unwrap_or(0);

    let score = (ratio(nodes_explored, universe.max_nodes)
        + ratio(zones_unlocked, universe.max_zones))
        / 2.0;

    (score.min(1.0), nodes_explored)
}

fn score_tech(sandbox: &Value, universe: &SandboxUniverse) -> (f64, usize) {
    let unlocked = sandbox
        .get("tech")
        .and_then(|t| t.get("unlock"))
        .and_then(|u| u.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    (ratio(unlocked, universe.max_tech_nodes), unlocked)
}

fn score_quests(sandbox: &Value, universe: &SandboxUniverse) -> (f64, usize) {
    let completed = sandbox
        .get("collect")
        .and_then(|c| c.get("complete"))
        .and_then(|c| c.get("quest"))
        .and_then(|q| q.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    (ratio(completed, universe.max_quests), completed)
}

fn score_base_building(sandbox: &Value, universe: &SandboxUniverse) -> f64 {
    let base_lv = sandbox
        .get("base")
        .and_then(|b| b.get("baseLv"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as usize;

    let blueprints = sandbox
        .get("build")
        .and_then(|b| b.get("book"))
        .and_then(|b| b.as_object())
        .map(|o| o.len())
        .unwrap_or(0);

    let base_score = ratio(base_lv, universe.max_base_level);
    let blueprint_score = ratio(blueprints, universe.max_blueprints);

    ((base_score + blueprint_score) / 2.0).min(1.0)
}

fn score_content_depth(sandbox: &Value, universe: &SandboxUniverse) -> f64 {
    let recipes = sandbox
        .get("cook")
        .and_then(|c| c.get("book"))
        .and_then(|b| b.as_object())
        .map(|o| o.len())
        .unwrap_or(0);

    let music = sandbox
        .get("collect")
        .and_then(|c| c.get("complete"))
        .and_then(|c| c.get("music"))
        .and_then(|m| m.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    let rift_max = sandbox
        .get("riftInfo")
        .and_then(|r| r.get("difficultyLvMax"))
        .and_then(|d| d.as_object())
        .map(|obj| {
            if obj.is_empty() {
                return 0.0;
            }
            let total: i64 = obj.values().filter_map(|v| v.as_i64()).sum();
            let max_possible = obj.len() as f64 * 16.0;
            (total as f64 / max_possible).min(1.0)
        })
        .unwrap_or(0.0);

    let recipe_score = ratio(recipes, universe.max_recipes);
    let music_score = ratio(music, universe.max_music);

    ((recipe_score + music_score + rift_max) / 3.0).min(1.0)
}
