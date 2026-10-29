use std::collections::{BTreeSet, HashSet};
use std::sync::{LazyLock, Mutex, PoisonError};

use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::sandbox_universe::{SandboxTemplate, SandboxUniverse};
use crate::database::queries::sandbox::get_user_sandbox_progress;

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
const MAX_TEMPLATE_SHAPE_LOGS: usize = 8;
/// Most child keys listed under one top-level key of a logged shape.
const MAX_CHILD_KEYS_LOGGED: usize = 12;

pub async fn grade_sandbox(
    pool: &PgPool,
    user_id: Uuid,
    game_data: &GameData,
) -> Result<f64, sqlx::Error> {
    grade_sandbox_detail(pool, user_id, game_data)
        .await
        .map(|g| g.total)
}

pub async fn grade_sandbox_detail(
    pool: &PgPool,
    user_id: Uuid,
    game_data: &GameData,
) -> Result<SandboxGrade, sqlx::Error> {
    let progress = get_user_sandbox_progress(pool, user_id).await?;
    Ok(grade_progress(
        progress.as_ref(),
        &game_data.sandbox_universe,
        GRADE_SANDBOX_V3,
    ))
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
pub(crate) fn grade_progress(
    progress: Option<&Value>,
    universe: &SandboxUniverse,
    all_topics: bool,
) -> SandboxGrade {
    if let Some(p) = progress {
        log_unread_template_shapes(p, universe);
    }
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

/// Logs each new shape of a `sandboxPerm.template` topic no grader reads (a
/// template other than V2/V3, or a topic the game data does not list), at most
/// `MAX_TEMPLATE_SHAPE_LOGS` per process. Keys only: no values, no array
/// lengths.
fn log_unread_template_shapes(progress: &Value, universe: &SandboxUniverse) {
    let Some(template) = progress.get("template") else {
        return;
    };
    let read = |key: &str, topic: &str| {
        (key == "SANDBOX_V2" && topic == "sandbox_1")
            || universe.topics.iter().any(|t| {
                t.template.key() == key
                    && t.topic_id == topic
                    && !matches!(t.template, SandboxTemplate::Other(_))
            })
    };
    let shapes = template_shapes(template, read);
    if shapes.is_empty() {
        return;
    }
    static SEEN: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));
    let fresh = {
        let mut seen = SEEN.lock().unwrap_or_else(PoisonError::into_inner);
        admit_new_shapes(&mut seen, shapes, MAX_TEMPLATE_SHAPE_LOGS)
    };
    for shape in fresh {
        tracing::warn!(
            shape = %shape,
            "sandboxPerm template carries a topic the RA grade does not read yet"
        );
    }
}

/// The shapes `seen` has not held yet, recorded in it, while it holds fewer
/// than `cap`.
fn admit_new_shapes(seen: &mut HashSet<String>, shapes: Vec<String>, cap: usize) -> Vec<String> {
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
fn template_shapes(template: &Value, read: impl Fn(&str, &str) -> bool) -> Vec<String> {
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
    (ACHIEVEMENT_WEIGHT * d.achievements
        + EXPLORATION_WEIGHT * d.exploration
        + TECH_WEIGHT * d.tech_tree
        + QUEST_WEIGHT * d.quests
        + BASE_WEIGHT * d.base_building
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
    let exploration = ((ratio(nodes_explored, universe.max_nodes)
        + ratio(zones_unlocked, universe.max_zones))
        / 2.0)
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
    let base_building = ((ratio(base_level, universe.max_base_level)
        + ratio(band_levels, universe.max_band_levels))
        / 2.0)
        .min(1.0);

    let recipes = sandbox
        .get("inventory")
        .and_then(|i| i.get("cookbook"))
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let music = completed_music(sandbox);
    let content_depth =
        ((ratio(recipes, universe.max_recipes) + ratio(music, universe.max_music)) / 2.0).min(1.0);

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

/// Returns `(score, recipes, music, rift_levels, rift_levels_max)`. Rift piece =
/// fixed rift dungeons cleared over the total; the raw pair feeds the breakdown.
fn score_content_depth(
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

#[cfg(test)]
mod tests {
    use super::{
        GRADE_SANDBOX_V3, admit_new_shapes, grade_progress, score_content_depth, score_exploration,
        template_shapes,
    };
    use crate::core::gamedata::types::sandbox_universe::{
        SandboxTemplate, SandboxTopicUniverse, SandboxUniverse,
    };
    use serde_json::{Value, json};
    use std::collections::HashSet;

    fn v2_counts() -> SandboxUniverse {
        SandboxUniverse {
            max_achievements: 10,
            max_nodes: 10,
            max_zones: 4,
            max_tech_nodes: 10,
            max_quests: 4,
            max_base_level: 8,
            max_blueprints: 10,
            max_recipes: 4,
            max_music: 4,
            max_rifts: 4,
            ..Default::default()
        }
    }

    /// Small V3 totals in CN `sandbox_2`'s shape: nodes `n_0`..`n_3` over
    /// zones `z_a` (`n_0`, `n_1`) and `z_b` (`n_2`, `n_3`), HOME excluded.
    fn v3_counts() -> SandboxUniverse {
        SandboxUniverse {
            max_achievements: 4,
            max_nodes: 4,
            max_zones: 2,
            max_tech_nodes: 4,
            max_quests: 2,
            max_base_level: 4,
            max_recipes: 2,
            max_music: 2,
            max_band_levels: 6,
            node_zones: [
                ("n_0", "z_a"),
                ("n_1", "z_a"),
                ("n_2", "z_b"),
                ("n_3", "z_b"),
            ]
            .into_iter()
            .map(|(n, z)| (n.to_owned(), z.to_owned()))
            .collect(),
            ..Default::default()
        }
    }

    fn topic(
        id: &str,
        template: SandboxTemplate,
        sort_id: i64,
        counts: SandboxUniverse,
    ) -> SandboxTopicUniverse {
        SandboxTopicUniverse {
            topic_id: id.into(),
            name: format!("name of {id}"),
            template,
            sort_id,
            counts,
        }
    }

    /// CN's two topics.
    fn universe() -> SandboxUniverse {
        SandboxUniverse {
            topics: vec![
                topic("sandbox_1", SandboxTemplate::V2, 1, v2_counts()),
                topic("sandbox_2", SandboxTemplate::V3, 2, v3_counts()),
            ],
            ..v2_counts()
        }
    }

    /// EN before the 2026-10-28 update: `sandbox_1` only.
    fn v2_only_universe() -> SandboxUniverse {
        SandboxUniverse {
            topics: vec![topic("sandbox_1", SandboxTemplate::V2, 1, v2_counts())],
            ..v2_counts()
        }
    }

    /// A V2 topic blob with `n` entries on every path the V2 grader reads.
    fn v2_blob(n: usize) -> Value {
        let list: Vec<String> = (0..n).map(|i| format!("id_{i}")).collect();
        let states: serde_json::Map<String, Value> = list
            .iter()
            .map(|id| (id.clone(), json!({ "state": 1 })))
            .collect();
        let book: serde_json::Map<String, Value> =
            list.iter().map(|id| (id.clone(), json!(1))).collect();
        json!({
            "collect": { "complete": { "achievement": list, "quest": list, "music": list } },
            "main": { "map": { "node": states, "zone": states } },
            "tech": { "unlock": list },
            "base": { "baseLv": n },
            "build": { "book": book },
            "cook": { "book": book },
            "riftInfo": { "fixFinish": list },
        })
    }

    /// A V3 topic blob in the shape of the EN sync of 2026-10-29 (the paths
    /// read and the decoys beside them).
    fn v3_blob() -> Value {
        json!({
            "map": {
                "node": {
                    "sandbox_2_node_home": { "state": 0 },
                    "n_0": { "state": 2 }, "n_1": { "state": 2 }, "n_2": { "state": 1 },
                    "n_3": { "state": 0 }, "not_in_table": { "state": 2 },
                },
                "zone": { "z_a": { "defend": { "main": -1, "sub": [] } } },
            },
            "base": { "level": 2, "score": 190 },
            "band": {
                "band_farmer": { "level": 1, "cond": [0, 1], "badge": false },
                "band_merchant": { "level": 2, "cond": [0, 1], "badge": false },
            },
            "inventory": { "cookbook": ["food_1"], "trap": { "x": 0 } },
            "tech": { "token": 1, "unlock": ["t_1", "t_2"] },
            "collect": {
                "pending": { "achievement": { "a_3": [0, 1] } },
                "complete": { "achievement": ["a_1"], "quest": ["story_1"], "music": ["m_1"] },
            },
            "current": { "bag": { "recipe": ["r_1", "r_2", "r_3"] } },
        })
    }

    fn progress(v2: Option<Value>, v3: Option<Value>) -> Value {
        let mut template = serde_json::Map::new();
        if let Some(b) = v2 {
            template.insert("SANDBOX_V2".into(), json!({ "sandbox_1": b }));
        }
        if let Some(b) = v3 {
            template.insert("SANDBOX_V3".into(), json!({ "sandbox_2": b }));
        }
        json!({ "template": template, "isClose": false })
    }

    fn v2_want_blob2() -> f64 {
        // 2/10 achievements, nodes 2/10 + zones 2/4, tech 2/10, quests 2/4,
        // base 2/8 + blueprints 2/10, recipes 2/4 + music 2/4 + rifts 2/4.
        0.30 * 0.2
            + 0.20 * ((0.2 + 0.5) / 2.0)
            + 0.15 * 0.2
            + 0.15 * 0.5
            + 0.10 * ((0.25 + 0.2) / 2.0)
            + 0.10 * ((0.5 + 0.5 + 0.5) / 3.0)
    }

    #[test]
    fn a_v2_only_table_grades_the_same_bits_on_or_off() {
        let p = progress(Some(v2_blob(2)), Some(v3_blob()));
        let on = grade_progress(Some(&p), &v2_only_universe(), true);
        let off = grade_progress(Some(&p), &v2_only_universe(), false);
        assert_eq!(on.total.to_bits(), off.total.to_bits());
        assert!((off.total - v2_want_blob2()).abs() < 1e-12, "{}", off.total);
        assert_eq!(on.topics.len(), 1);
        assert_eq!(on.topics[0].topic_id.as_deref(), Some("sandbox_1"));
        assert_eq!(on.topics[0].topic_name, "name of sandbox_1");
    }

    #[test]
    fn every_topic_weighs_the_same() {
        // Finished-looking V2, never opened V3: half the V2 grade.
        let d = grade_progress(Some(&progress(Some(v2_blob(2)), None)), &universe(), true);
        assert_eq!(d.topics.len(), 2);
        assert!(
            (d.total - v2_want_blob2() / 2.0).abs() < 1e-12,
            "{}",
            d.total
        );
        assert_eq!(d.topics[1].total.to_bits(), 0.0_f64.to_bits());
        assert_eq!(d.topics[1].achievements_total, 4, "V3 totals reported");
        assert!(d.topics[1].v3);

        let both = grade_progress(
            Some(&progress(Some(v2_blob(2)), Some(v3_blob()))),
            &universe(),
            true,
        );
        let mean = (both.topics[0].total + both.topics[1].total) / 2.0;
        assert_eq!(both.total.to_bits(), mean.to_bits());
    }

    #[test]
    fn the_v3_reader_reads_the_measured_paths() {
        let d = grade_progress(Some(&progress(None, Some(v3_blob()))), &universe(), true);
        let v3 = &d.topics[1];
        assert_eq!(v3.topic_id.as_deref(), Some("sandbox_2"));
        assert_eq!(
            v3.achievements_completed, 1,
            "collect.complete, not pending"
        );
        assert_eq!(
            v3.nodes_explored, 3,
            "state >= 1, HOME and unknown ids left out"
        );
        assert_eq!(v3.zones_unlocked, 2, "zones of the open nodes");
        assert_eq!(v3.tech_unlocked, 2);
        assert_eq!(v3.quests_completed, 1);
        assert_eq!(v3.base_level, 2, "base.level");
        assert_eq!(v3.band_levels, 3, "summed band.<id>.level");
        assert_eq!(
            (v3.blueprints, v3.blueprints_total),
            (0, 0),
            "run bag not graded"
        );
        assert_eq!(v3.recipes, 1, "inventory.cookbook");
        assert_eq!(v3.music, 1);
        assert_eq!((v3.rift_levels, v3.rift_levels_max), (0, 0));
        // 1/4 achievements, nodes 3/4 + zones 2/2, tech 2/4, quests 1/2,
        // base 2/4 + bands 3/6, recipes 1/2 + music 1/2.
        let want = 0.30 * 0.25
            + 0.20 * ((0.75 + 1.0) / 2.0)
            + 0.15 * 0.5
            + 0.15 * 0.5
            + 0.10 * 0.5
            + 0.10 * 0.5;
        assert!((v3.total - want).abs() < 1e-12, "{} vs {want}", v3.total);
        assert!((d.total - want / 2.0).abs() < 1e-12, "no V2 progress: half");
    }

    #[test]
    fn the_kill_switch_restores_the_v2_only_grade() {
        let p = progress(Some(v2_blob(2)), Some(v3_blob()));
        let off = grade_progress(Some(&p), &universe(), false);
        let v2_only = grade_progress(
            Some(&progress(Some(v2_blob(2)), None)),
            &v2_only_universe(),
            true,
        );
        assert_eq!(off.total.to_bits(), v2_only.total.to_bits());
        assert_eq!(off.topics.len(), 1);
        assert!(GRADE_SANDBOX_V3, "shipped on");
    }

    #[test]
    fn a_template_with_no_reader_is_not_graded() {
        let mut u = universe();
        u.topics.push(topic(
            "sandbox_9",
            SandboxTemplate::Other("SANDBOX_V9".into()),
            9,
            SandboxUniverse::default(),
        ));
        let d = grade_progress(Some(&progress(Some(v2_blob(2)), None)), &u, true);
        assert_eq!(d.topics.len(), 2, "the V9 topic neither scores nor dilutes");
    }

    /// A structure-only V2 blob sized like a mid-game EN save, against EN's
    /// real totals (71 / 127 / 6 / 54 / 7 / 8 / 96 / 53 / 10 / 5). The pinned
    /// bits are HEAD's: its `grade_sandbox_detail` body, run at 9c8f97b5 on
    /// this blob, returned 0.7620120396565164.
    #[test]
    fn a_v2_grade_matches_head_bit_for_bit() {
        let ids = |prefix: &str, n: usize| -> Vec<String> {
            (0..n).map(|i| format!("{prefix}_{i}")).collect()
        };
        let states = |unlocked: usize, locked: usize| -> serde_json::Map<String, Value> {
            (0..unlocked + locked)
                .map(|i| {
                    (
                        format!("s_{i}"),
                        json!({ "state": if i < unlocked { 1 } else { 0 } }),
                    )
                })
                .collect()
        };
        let book = |n: usize| -> serde_json::Map<String, Value> {
            (0..n).map(|i| (format!("b_{i}"), json!(1))).collect()
        };
        let blob = json!({
            "collect": { "complete": {
                "achievement": ids("a", 40), "quest": ids("q", 7), "music": ids("m", 8),
            } },
            "main": { "map": { "node": states(90, 10), "zone": states(5, 1) } },
            "tech": { "unlock": ids("t", 50) },
            "base": { "baseLv": 6 },
            "build": { "book": book(90) },
            "cook": { "book": book(30) },
            "riftInfo": { "fixFinish": ids("r", 3) },
        });
        let en = SandboxUniverse {
            max_achievements: 71,
            max_nodes: 127,
            max_zones: 6,
            max_tech_nodes: 54,
            max_quests: 7,
            max_stages: 106,
            max_recipes: 53,
            max_music: 10,
            max_base_level: 8,
            max_blueprints: 96,
            max_rifts: 5,
            ..Default::default()
        };
        let en_listed = SandboxUniverse {
            topics: vec![topic("sandbox_1", SandboxTemplate::V2, 1, en.clone())],
            ..en.clone()
        };
        let p = progress(Some(blob), None);
        for u in [&en, &en_listed] {
            for switch in [false, true] {
                let d = grade_progress(Some(&p), u, switch);
                assert_eq!(d.total.to_bits(), 0x3fe8_6267_12af_74ae, "{}", d.total);
            }
        }
    }

    #[test]
    fn template_shapes_name_every_unread_topic_by_structure_only() {
        let template = json!({
            "SANDBOX_V2": { "sandbox_1": v2_blob(3) },
            "SANDBOX_V3": {
                "sandbox_2": v3_blob(),
                "sandbox_3": { "base": { "level": 4 }, "tasks": [1, 2, 3], "day": 12 },
            },
            "SANDBOX_V9": [1, 2],
        });
        let read = |key: &str, topic: &str| {
            matches!(
                (key, topic),
                ("SANDBOX_V2", "sandbox_1") | ("SANDBOX_V3", "sandbox_2")
            )
        };
        assert_eq!(
            template_shapes(&template, read),
            vec![
                "SANDBOX_V3.sandbox_3 {base{level}, day, tasks[]}".to_owned(),
                "SANDBOX_V9: array".to_owned(),
            ]
        );
        let read_only = json!({ "SANDBOX_V2": { "sandbox_1": v2_blob(3) }, "SANDBOX_V3": {} });
        assert!(
            template_shapes(&read_only, read).is_empty(),
            "read topics log nothing"
        );
        assert_eq!(
            template_shapes(&json!([]), read),
            vec!["template: array".to_owned()]
        );
    }

    #[test]
    fn each_shape_is_admitted_once_up_to_the_cap() {
        let mut seen = HashSet::new();
        let shape = |i: usize| format!("SANDBOX_V3.sandbox_2 {{k{i}}}");
        assert_eq!(
            admit_new_shapes(&mut seen, vec![shape(0), shape(0)], 8),
            vec![shape(0)]
        );
        assert!(admit_new_shapes(&mut seen, vec![shape(0)], 8).is_empty());
        let fresh = admit_new_shapes(&mut seen, (1..20).map(shape).collect(), 8);
        assert_eq!(fresh.len(), 7, "8 shapes per process, 1 already seen");
        assert!(admit_new_shapes(&mut seen, vec![shape(99)], 8).is_empty());
    }

    #[test]
    fn no_progress_reports_every_topics_totals() {
        let d = grade_progress(None, &universe(), true);
        assert_eq!(d.total.to_bits(), 0.0_f64.to_bits());
        assert_eq!(d.topics[0].achievements_total, 10);
        assert_eq!(
            d.topics[0].rift_levels_max, 0,
            "as before: no rift total without progress"
        );
        assert_eq!(d.topics[1].band_levels_max, 6);
        let off = grade_progress(None, &universe(), false);
        assert_eq!(off.topics[0].nodes_total, 10);
    }

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
