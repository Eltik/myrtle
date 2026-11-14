use std::collections::{BTreeSet, HashSet};

use serde_json::Value;

use myrtle_gamedata_raw::tables::sandbox_universe::{SandboxTemplate, SandboxUniverse};

use crate::GradeData;

use super::types::{SandboxGrade, SandboxGradeDetail};

pub const ACHIEVEMENT_WEIGHT: f64 = 0.30;
pub const EXPLORATION_WEIGHT: f64 = 0.20;
pub const TECH_WEIGHT: f64 = 0.15;
pub const QUEST_WEIGHT: f64 = 0.15;
pub const BASE_WEIGHT: f64 = 0.10;
pub const CONTENT_WEIGHT: f64 = 0.10;

/// Kill switch for grading every RA topic. On, the RA grade is the mean of
/// every topic the game data lists with a reader (`SANDBOX_V2` and
/// `SANDBOX_V3`), each weighted the same, so a topic the player never opened
/// counts 0. Off, it is the `sandbox_1` grade alone, bit for bit what it was
/// before topics were graded. With a table that lists only `sandbox_1` (EN
/// before the 2026-10-28 update) both settings give the same bits.
pub const GRADE_SANDBOX_V3: bool = true;

/// Most distinct `sandboxPerm.template` shapes logged per process.
pub const MAX_TEMPLATE_SHAPE_LOGS: usize = 8;
/// Most child keys listed under one top-level key of a logged shape.
const MAX_CHILD_KEYS_LOGGED: usize = 12;

/// Main's `grade_sandbox_detail` after its query: the stored `sandboxPerm` blob
/// (`None` when the user has none) is an input.
#[must_use]
pub fn grade_sandbox_detail(game_data: &GradeData, progress: Option<&Value>) -> SandboxGrade {
    grade_progress(progress, &game_data.sandbox_universe, GRADE_SANDBOX_V3)
}

#[must_use]
pub fn grade_sandbox(game_data: &GradeData, progress: Option<&Value>) -> f64 {
    grade_sandbox_detail(game_data, progress).total
}

/// The RA grade of a stored `sandboxPerm` blob.
///
/// With `all_topics`, every topic in `universe.topics` that has a reader is
/// graded against its own totals and the RA grade is their plain mean: two
/// topics weigh 0.5 each, so a player who finished `sandbox_1` and never opened
/// `sandbox_2` sits at half. Without it, or with no topic list, the grade is
/// `sandbox_1` alone, as before topics existed.
///
/// Template topics no grader reads are logged by shape either way.
pub fn grade_progress(
    progress: Option<&Value>,
    universe: &SandboxUniverse,
    all_topics: bool,
) -> SandboxGrade {
    let graded: Vec<SandboxGradeDetail> = if all_topics {
        universe
            .topics
            .iter()
            .filter_map(|t| {
                let blob = progress
                    .and_then(|p| p.get("template"))
                    .and_then(|tpl| tpl.get(t.template.key()))
                    .and_then(|topics| topics.get(&t.topic_id));
                let mut d = match t.template {
                    SandboxTemplate::V2 => score_topic_v2(blob, &t.counts),
                    SandboxTemplate::V3 => score_topic_v3(blob, &t.counts),
                    SandboxTemplate::Other(_) => return None,
                };
                d.topic_id = Some(t.topic_id.clone());
                d.topic_name.clone_from(&t.name);
                Some(d)
            })
            .collect()
    } else {
        Vec::new()
    };
    if graded.is_empty() {
        let mut d = score_topic_v2(progress.and_then(extract_sandbox_v2), universe);
        if let Some(t) = universe.topics.iter().find(|t| t.topic_id == "sandbox_1") {
            d.topic_name.clone_from(&t.name);
        }
        d.topic_id = Some("sandbox_1".to_owned());
        return SandboxGrade {
            total: d.total,
            topics: vec![d],
        };
    }
    let total = graded.iter().map(|d| d.total).sum::<f64>() / graded.len() as f64;
    SandboxGrade {
        total,
        topics: graded,
    }
}

/// The shape of each `sandboxPerm.template` topic no grader reads (a template
/// other than V2/V3, or a topic the game data does not list). Keys only: no
/// values, no array lengths. Main logged these from inside the grade with a
/// process-wide dedupe; an engine takes no global state, so the caller logs them,
/// at most `MAX_TEMPLATE_SHAPE_LOGS` per process through [`admit_new_shapes`].
#[must_use]
pub fn unread_template_shapes(progress: &Value, universe: &SandboxUniverse) -> Vec<String> {
    let Some(template) = progress.get("template") else {
        return Vec::new();
    };
    let read = |key: &str, topic: &str| {
        (key == "SANDBOX_V2" && topic == "sandbox_1")
            || universe.topics.iter().any(|t| {
                t.template.key() == key
                    && t.topic_id == topic
                    && !matches!(t.template, SandboxTemplate::Other(_))
            })
    };
    template_shapes(template, read)
}

/// The shapes `seen` has not held yet, recorded in it, while it holds fewer
/// than `cap`.
pub fn admit_new_shapes(
    seen: &mut HashSet<String>,
    shapes: Vec<String>,
    cap: usize,
) -> Vec<String> {
    let mut fresh = Vec::new();
    for shape in shapes {
        if seen.len() >= cap {
            break;
        }
        if seen.insert(shape.clone()) {
            fresh.push(shape);
        }
    }
    fresh
}

/// One line per topic `read` rejects:
/// `SANDBOX_V4.sandbox_3 {base{level}, collect{complete}, tech{unlock}}`.
/// A template key whose value is not an object of topics is named with its
/// JSON kind.
pub fn template_shapes(template: &Value, read: impl Fn(&str, &str) -> bool) -> Vec<String> {
    let Some(map) = template.as_object() else {
        return vec![format!("template: {}", json_kind(template))];
    };
    let mut shapes = Vec::new();
    for (key, topics) in map {
        match topics.as_object() {
            Some(t) if !t.is_empty() => {
                for (topic, blob) in t {
                    if !read(key, topic) {
                        shapes.push(format!("{key}.{topic} {}", blob_shape(blob)));
                    }
                }
            }
            _ if key == "SANDBOX_V2" || key == "SANDBOX_V3" => {}
            _ => shapes.push(format!("{key}: {}", json_kind(topics))),
        }
    }
    shapes.sort();
    shapes
}

/// `{a{x, y}, b[], c}`: sorted top-level keys, each object's sorted child keys
/// (at most `MAX_CHILD_KEYS_LOGGED`, then `+N`), `[]` for an array.
fn blob_shape(blob: &Value) -> String {
    let Some(map) = blob.as_object() else {
        return json_kind(blob).to_owned();
    };
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    let parts: Vec<String> = keys
        .into_iter()
        .map(|k| match &map[k.as_str()] {
            Value::Object(child) => {
                let mut ck: Vec<&str> = child.keys().map(String::as_str).collect();
                ck.sort_unstable();
                let more = ck.len().saturating_sub(MAX_CHILD_KEYS_LOGGED);
                ck.truncate(MAX_CHILD_KEYS_LOGGED);
                let tail = if more > 0 {
                    format!(", +{more}")
                } else {
                    String::new()
                };
                format!("{k}{{{}{tail}}}", ck.join(", "))
            }
            Value::Array(_) => format!("{k}[]"),
            _ => k.clone(),
        })
        .collect();
    format!("{{{}}}", parts.join(", "))
}

const fn json_kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn weighted_total(d: &SandboxGradeDetail) -> f64 {
    (BASE_WEIGHT.mul_add(d.base_building, QUEST_WEIGHT.mul_add(d.quests, TECH_WEIGHT.mul_add(d.tech_tree, EXPLORATION_WEIGHT.mul_add(d.exploration, ACHIEVEMENT_WEIGHT * d.achievements))))
        + CONTENT_WEIGHT * d.content_depth)
        .clamp(0.0, 1.0)
}

/// A detail carrying every total of `universe` and no progress.
fn empty_detail(universe: &SandboxUniverse) -> SandboxGradeDetail {
    SandboxGradeDetail {
        achievements_total: universe.max_achievements,
        nodes_total: universe.max_nodes,
        zones_total: universe.max_zones,
        tech_total: universe.max_tech_nodes,
        quests_total: universe.max_quests,
        base_level_max: universe.max_base_level,
        blueprints_total: universe.max_blueprints,
        recipes_total: universe.max_recipes,
        music_total: universe.max_music,
        band_levels_max: universe.max_band_levels,
        ..SandboxGradeDetail::default()
    }
}

/// One `SANDBOX_V2` topic (`template.SANDBOX_V2.<topic>`) against its totals.
fn score_topic_v2(sandbox: Option<&Value>, universe: &SandboxUniverse) -> SandboxGradeDetail {
    let Some(sandbox) = sandbox else {
        return empty_detail(universe);
    };
    let (achievements, achievements_completed) = score_achievements(sandbox, universe);
    let (exploration, nodes_explored, zones_unlocked) = score_exploration(sandbox, universe);
    let (tech_tree, tech_unlocked) = score_tech(sandbox, universe);
    let (quests, quests_completed) = score_quests(sandbox, universe);
    let (base_building, base_level, blueprints) = score_base_building(sandbox, universe);
    let (content_depth, recipes, music, rift_levels, rift_levels_max) =
        score_content_depth(sandbox, universe);

    let mut d = SandboxGradeDetail {
        achievements,
        exploration,
        tech_tree,
        quests,
        base_building,
        content_depth,
        achievements_completed,
        nodes_explored,
        zones_unlocked,
        tech_unlocked,
        quests_completed,
        base_level,
        blueprints,
        recipes,
        music,
        rift_levels,
        rift_levels_max,
        ..empty_detail(universe)
    };
    d.total = weighted_total(&d);
    d
}

/// One `SANDBOX_V3` topic (`template.SANDBOX_V3.<topic>`) against its totals,
/// on the paths an EN sync of 2026-10-29 holds. Same six categories and
/// weights as V2; where V2 has a piece V3 lacks, the category averages what V3
/// has:
/// - exploration: `map.node.<id>.state >= 1` over the non-HOME nodes, and the
///   zones of those nodes (the save keeps no zone state);
/// - base building: `base.level`, and summed `band.<id>.level` in place of
///   blueprints (V3 blueprints live only in the current run's bag);
/// - content depth: `inventory.cookbook` and `collect.complete.music`, no rift
///   piece.
fn score_topic_v3(sandbox: Option<&Value>, universe: &SandboxUniverse) -> SandboxGradeDetail {
    let Some(sandbox) = sandbox else {
        return SandboxGradeDetail {
            v3: true,
            ..empty_detail(universe)
        };
    };
    let (achievements, achievements_completed) = score_achievements(sandbox, universe);
    let (tech_tree, tech_unlocked) = score_tech(sandbox, universe);
    let (quests, quests_completed) = score_quests(sandbox, universe);

    let open_nodes: Vec<&str> = sandbox
        .get("map")
        .and_then(|m| m.get("node"))
        .and_then(Value::as_object)
        .map(|nodes| {
            nodes
                .iter()
                .filter(|(id, v)| universe.node_zones.contains_key(id.as_str()) && is_unlocked(v))
                .map(|(id, _)| id.as_str())
                .collect()
        })
        .unwrap_or_default();
    let nodes_explored = open_nodes.len();
    let zones_unlocked = open_nodes
        .iter()
        .filter_map(|id| universe.node_zones.get(*id))
        .filter(|z| !z.is_empty())
        .collect::<BTreeSet<_>>()
        .len();
    let exploration = f64::midpoint(ratio(nodes_explored, universe.max_nodes), ratio(zones_unlocked, universe.max_zones))
        .min(1.0);

    let base_level = sandbox
        .get("base")
        .and_then(|b| b.get("level"))
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let band_levels = sandbox
        .get("band")
        .and_then(Value::as_object)
        .map_or(0, |bands| {
            bands
                .values()
                .filter_map(|b| b.get("level").and_then(Value::as_u64))
                .sum::<u64>() as usize
        });
    let base_building = f64::midpoint(ratio(base_level, universe.max_base_level), ratio(band_levels, universe.max_band_levels))
        .min(1.0);

    let recipes = sandbox
        .get("inventory")
        .and_then(|i| i.get("cookbook"))
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let music = completed_music(sandbox);
    let content_depth =
        f64::midpoint(ratio(recipes, universe.max_recipes), ratio(music, universe.max_music)).min(1.0);

    let mut d = SandboxGradeDetail {
        v3: true,
        achievements,
        exploration,
        tech_tree,
        quests,
        base_building,
        content_depth,
        achievements_completed,
        nodes_explored,
        zones_unlocked,
        tech_unlocked,
        quests_completed,
        base_level,
        band_levels,
        recipes,
        music,
        ..empty_detail(universe)
    };
    d.total = weighted_total(&d);
    d
}

fn extract_sandbox_v2(progress: &Value) -> Option<&Value> {
    progress
        .get("template")?
        .get("SANDBOX_V2")?
        .get("sandbox_1")
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

#[must_use]
pub fn score_exploration(sandbox: &Value, universe: &SandboxUniverse) -> (f64, usize, usize) {
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
        f64::midpoint(base_score, blueprint_score).min(1.0),
        base_lv,
        blueprints,
    )
}

/// Returns `(score, recipes, music, rift_levels, rift_levels_max)`. Rift piece =
/// fixed rift dungeons cleared over the total; the raw pair feeds the breakdown.
pub fn score_content_depth(
    sandbox: &Value,
    universe: &SandboxUniverse,
) -> (f64, usize, usize, usize, usize) {
    let recipes = sandbox
        .get("cook")
        .and_then(|c| c.get("book"))
        .and_then(|b| b.as_object())
        .map_or(0, serde_json::Map::len);

    let music = completed_music(sandbox);

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

fn completed_music(sandbox: &Value) -> usize {
    sandbox
        .get("collect")
        .and_then(|c| c.get("complete"))
        .and_then(|c| c.get("music"))
        .and_then(|m| m.as_array())
        .map_or(0, std::vec::Vec::len)
}
