use std::collections::HashSet;
use std::sync::{LazyLock, Mutex, PoisonError};

use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::sandbox_universe::{SandboxTemplate, SandboxUniverse};
use crate::database::queries::sandbox::get_user_sandbox_progress;

use super::types::SandboxGradeDetail;

pub const ACHIEVEMENT_WEIGHT: f64 = 0.30;
pub const EXPLORATION_WEIGHT: f64 = 0.20;
pub const TECH_WEIGHT: f64 = 0.15;
pub const QUEST_WEIGHT: f64 = 0.15;
pub const BASE_WEIGHT: f64 = 0.10;
pub const CONTENT_WEIGHT: f64 = 0.10;

/// Kill switch for grading `SANDBOX_V3` topics (`sandbox_2`, "Relaunch
/// Anchor"). Ships `false`, INERT: with it off the RA grade is the V2-only
/// grade for every player, V3 progress or not. The V3 reader below is a
/// hypothesis (no V3 progress blob has been seen), and a path that exists
/// under V3 with another meaning can only OVER-credit, which `max()` would
/// then hide behind a higher number. Price of a wrong guess left on: a player
/// at V2 0.30 whose V3 reads 0.80 moves RA 0.30 -> 0.80, which is +0.04 on the
/// overall score (RA weighs 0.2 of 2.5, 8%). Flip it once the shapes logged by
/// `log_unread_template_shapes` confirm the paths in `best_v3_topic`.
pub const GRADE_SANDBOX_V3: bool = false;

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
        .map(|d| d.total)
}

pub async fn grade_sandbox_detail(
    pool: &PgPool,
    user_id: Uuid,
    game_data: &GameData,
) -> Result<SandboxGradeDetail, sqlx::Error> {
    let progress = get_user_sandbox_progress(pool, user_id).await?;
    Ok(grade_progress(
        progress.as_ref(),
        &game_data.sandbox_universe,
        GRADE_SANDBOX_V3,
    ))
}

/// The RA grade of a stored `sandboxPerm` blob.
///
/// `sandbox_1` (`template.SANDBOX_V2.sandbox_1`) is graded exactly as before.
/// Each `SANDBOX_V3` topic is graded against its own totals on every call, but
/// the result is used only with `grade_v3`; then the higher of the V2 grade and
/// the best V3 grade is the RA grade. Taking the max is a TRADE: it never drops
/// a player below their V2 grade, and it grades a player who finished both
/// topics the same as one who finished one; with an unconfirmed reader it can
/// also let an over-counted V3 replace a correct V2 (priced at
/// `GRADE_SANDBOX_V3`). The derived answer is a per-topic blend weighted like
/// `theme_weight` in the IS grade, calibrated on real V3 blobs.
///
/// Every template key other than `SANDBOX_V2` is logged by shape, whether or
/// not the switch is on and whether or not the game data lists the topic.
pub(crate) fn grade_progress(
    progress: Option<&Value>,
    universe: &SandboxUniverse,
    grade_v3: bool,
) -> SandboxGradeDetail {
    let v2 = progress
        .and_then(extract_sandbox_v2)
        .map(|sandbox| score_topic(&sandbox, universe, Some("sandbox_1"), false));
    let v3 = progress.and_then(|p| best_v3_topic(p, universe));
    if let Some(p) = progress {
        log_unread_template_shapes(p, v3.as_ref());
    }
    let v3 = v3.filter(|_| grade_v3);
    match (v2, v3) {
        (Some(v2), Some(v3)) if v3.total > v2.total => v3,
        (Some(v2), _) => v2,
        (None, Some(v3)) => v3,
        (None, None) => SandboxGradeDetail {
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
        },
    }
}

/// The best-graded `SANDBOX_V3` topic the player has progress in.
///
/// HYPOTHESIS, open until real V3 blobs are logged: a V3 topic's progress sits
/// at `template.SANDBOX_V3.<topic id>` and uses V2's own paths
/// (`collect.complete.achievement`, `main.map.node`, `tech.unlock`,
/// `base.baseLv`, `build.book`, `cook.book`, ...). V2's sits at
/// `template.SANDBOX_V2.sandbox_1`, measured on the EN dump and in 2,566 of
/// 2,629 rows of the 2026-08-24 backup; no V3 blob exists in either (0 rows,
/// and the backup holds no CN account). An absent path reads as 0; a path that
/// exists with another meaning over-counts, which is why `GRADE_SANDBOX_V3`
/// ships off.
fn best_v3_topic(progress: &Value, universe: &SandboxUniverse) -> Option<SandboxGradeDetail> {
    universe
        .topics
        .iter()
        .filter(|t| t.template == SandboxTemplate::V3)
        .filter_map(|t| {
            let sandbox = progress
                .get("template")?
                .get(t.template.key())?
                .get(&t.topic_id)?;
            Some(score_topic(sandbox, &t.counts, Some(&t.topic_id), true))
        })
        .max_by(|a, b| a.total.total_cmp(&b.total))
}

/// Logs each new shape of a `sandboxPerm.template` key the RA grade does not
/// read (anything but `SANDBOX_V2`), at most `MAX_TEMPLATE_SHAPE_LOGS` per
/// process, with what the V3 reader counted when the game data lists the
/// topic. Keys only: no values, no array lengths. This is the measurement that
/// settles the `best_v3_topic` hypothesis, so it does not depend on the switch.
fn log_unread_template_shapes(progress: &Value, v3: Option<&SandboxGradeDetail>) {
    let Some(template) = progress.get("template") else {
        return;
    };
    let shapes = template_shapes(template);
    if shapes.is_empty() {
        return;
    }
    static SEEN: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));
    let fresh = {
        let mut seen = SEEN.lock().unwrap_or_else(PoisonError::into_inner);
        admit_new_shapes(&mut seen, shapes, MAX_TEMPLATE_SHAPE_LOGS)
    };
    let counted = v3.map(|d| {
        (
            d.topic_id.clone().unwrap_or_default(),
            [
                d.achievements_completed,
                d.nodes_explored,
                d.zones_unlocked,
                d.tech_unlocked,
                d.quests_completed,
                d.base_level,
                d.blueprints,
                d.recipes,
                d.music,
            ],
        )
    });
    for shape in fresh {
        tracing::warn!(
            shape = %shape,
            v3_reader_counted = ?counted,
            grading_v3 = GRADE_SANDBOX_V3,
            "sandboxPerm template carries a key the RA grade does not read yet \
             (counted: achievements, nodes, zones, tech, quests, base level, \
             blueprints, recipes, music)"
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

/// One line per topic under every template key but `SANDBOX_V2`:
/// `SANDBOX_V3.sandbox_2 {base{baseLv}, collect{complete}, tech{unlock}}`.
/// A key whose value is not an object of topics is named with its JSON kind.
fn template_shapes(template: &Value) -> Vec<String> {
    let Some(map) = template.as_object() else {
        return vec![format!("template: {}", json_kind(template))];
    };
    let mut shapes = Vec::new();
    for (key, topics) in map {
        if key == "SANDBOX_V2" {
            continue;
        }
        match topics.as_object() {
            Some(t) if !t.is_empty() => {
                for (topic, blob) in t {
                    shapes.push(format!("{key}.{topic} {}", blob_shape(blob)));
                }
            }
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

fn json_kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// One topic's grade against its own totals. `v3` drops the rift piece when
/// the topic has no rifts (V3 has none): content depth averages recipes and
/// music instead of capping at 2/3, and `rift_levels` is reported as 0 so the
/// breakdown never shows cleared rifts out of 0. V2 keeps its formula bit for
/// bit.
fn score_topic(
    sandbox: &Value,
    universe: &SandboxUniverse,
    topic_id: Option<&str>,
    v3: bool,
) -> SandboxGradeDetail {
    let (achievements, achievements_completed) = score_achievements(sandbox, universe);
    let (exploration, nodes_explored, zones_unlocked) = score_exploration(sandbox, universe);
    let (tech_tree, tech_unlocked) = score_tech(sandbox, universe);
    let (quests, quests_completed) = score_quests(sandbox, universe);
    let (base_building, base_level, blueprints) = score_base_building(sandbox, universe);
    let (mut content_depth, recipes, music, mut rift_levels, rift_levels_max) =
        score_content_depth(sandbox, universe);
    if v3 && rift_levels_max == 0 {
        content_depth = ((ratio(recipes, universe.max_recipes) + ratio(music, universe.max_music))
            / 2.0)
            .min(1.0);
        rift_levels = 0;
    }

    let total = (ACHIEVEMENT_WEIGHT * achievements
        + EXPLORATION_WEIGHT * exploration
        + TECH_WEIGHT * tech_tree
        + QUEST_WEIGHT * quests
        + BASE_WEIGHT * base_building
        + CONTENT_WEIGHT * content_depth)
        .clamp(0.0, 1.0);

    SandboxGradeDetail {
        topic_id: topic_id.map(str::to_owned),
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

    /// CN's two topics, with small totals; V3 has no rifts.
    fn universe() -> SandboxUniverse {
        let v3 = SandboxUniverse {
            max_achievements: 4,
            max_nodes: 4,
            max_zones: 2,
            max_tech_nodes: 4,
            max_quests: 2,
            max_base_level: 4,
            max_blueprints: 4,
            max_recipes: 2,
            max_music: 2,
            max_rifts: 0,
            ..Default::default()
        };
        SandboxUniverse {
            topics: vec![
                SandboxTopicUniverse {
                    topic_id: "sandbox_1".into(),
                    template: SandboxTemplate::V2,
                    sort_id: 1,
                    counts: v2_counts(),
                },
                SandboxTopicUniverse {
                    topic_id: "sandbox_2".into(),
                    template: SandboxTemplate::V3,
                    sort_id: 2,
                    counts: v3,
                },
            ],
            ..v2_counts()
        }
    }

    /// A topic blob with `n` entries on every path the grader reads. The paths
    /// are V2's, measured; under `SANDBOX_V3` they are the hypothesis
    /// `best_v3_topic` documents.
    fn blob(n: usize) -> Value {
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

    #[test]
    fn v2_only_progress_is_bit_identical_with_v3_grading_on_or_off() {
        let p = progress(Some(blob(2)), None);
        let on = grade_progress(Some(&p), &universe(), true);
        let off = grade_progress(Some(&p), &universe(), false);
        assert_eq!(on.total.to_bits(), off.total.to_bits());
        // 2/10 achievements, nodes 2/10 + zones 2/4, tech 2/10, quests 2/4,
        // base 2/8 + blueprints 2/10, recipes 2/4 + music 2/4 + rifts 2/4.
        let want = 0.30 * 0.2
            + 0.20 * ((0.2 + 0.5) / 2.0)
            + 0.15 * 0.2
            + 0.15 * 0.5
            + 0.10 * ((0.25 + 0.2) / 2.0)
            + 0.10 * ((0.5 + 0.5 + 0.5) / 3.0);
        assert!((off.total - want).abs() < 1e-12, "{} vs {want}", off.total);
        assert_eq!(on.achievements_total, 10, "V2 totals reported");
    }

    #[test]
    fn v3_progress_replaces_a_lower_v2_grade() {
        let p = progress(Some(blob(1)), Some(blob(2)));
        let v2_only = grade_progress(Some(&p), &universe(), false);
        let d = grade_progress(Some(&p), &universe(), true);
        assert!(d.total > v2_only.total, "{} vs {}", d.total, v2_only.total);
        assert_eq!(d.achievements_total, 4, "V3 totals reported");
        assert_eq!(d.rift_levels_max, 0);
    }

    #[test]
    fn v3_progress_never_lowers_the_v2_grade() {
        let v2_only = grade_progress(Some(&progress(Some(blob(3)), None)), &universe(), true);
        // A V3 blob in a shape no hypothesised path matches reads as zero.
        let unknown = json!({ "somethingNew": { "progress": [1, 2, 3] } });
        for v3 in [blob(0), blob(1), unknown] {
            let d = grade_progress(Some(&progress(Some(blob(3)), Some(v3))), &universe(), true);
            assert_eq!(d.total.to_bits(), v2_only.total.to_bits());
            assert_eq!(d.achievements_total, 10, "still the V2 detail");
        }
    }

    #[test]
    fn a_v3_only_player_is_graded_on_v3() {
        let d = grade_progress(Some(&progress(None, Some(blob(2)))), &universe(), true);
        // 2/4 achievements, nodes 2/4 + zones 2/2, tech 2/4, quests 2/2,
        // base 2/4 + blueprints 2/4, recipes 2/2 + music 2/2 with no rift piece.
        let want = 0.30 * 0.5
            + 0.20 * ((0.5 + 1.0) / 2.0)
            + 0.15 * 0.5
            + 0.15 * 1.0
            + 0.10 * 0.5
            + 0.10 * 1.0;
        assert!((d.total - want).abs() < 1e-12, "{} vs {want}", d.total);
        assert!(
            (d.content_depth - 1.0).abs() < 1e-12,
            "2/2 recipes, 2/2 music, no rift piece"
        );
        let off = grade_progress(Some(&progress(None, Some(blob(2)))), &universe(), false);
        assert_eq!(
            off.total.to_bits(),
            0.0_f64.to_bits(),
            "kill switch: V2-only grade"
        );
    }

    #[test]
    fn the_shipped_default_is_inert() {
        // V3 progress that would win if graded: the default ignores it.
        let p = progress(Some(blob(1)), Some(blob(2)));
        let shipped = grade_progress(Some(&p), &universe(), GRADE_SANDBOX_V3);
        let v2_only = grade_progress(Some(&progress(Some(blob(1)), None)), &universe(), true);
        assert_eq!(shipped.total.to_bits(), v2_only.total.to_bits());
        assert_eq!(shipped.topic_id.as_deref(), Some("sandbox_1"));
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
            topics: Vec::new(),
        };
        let p = progress(Some(blob), None);
        for switch in [false, true] {
            let d = grade_progress(Some(&p), &en, switch);
            assert_eq!(d.total.to_bits(), 0x3fe8_6267_12af_74ae, "{}", d.total);
        }
    }

    #[test]
    fn a_v3_breakdown_reports_no_rifts_and_names_its_topic() {
        // `blob` fills `riftInfo.fixFinish` too; V3 has no rift table.
        let d = grade_progress(Some(&progress(None, Some(blob(2)))), &universe(), true);
        assert_eq!(d.topic_id.as_deref(), Some("sandbox_2"));
        assert_eq!((d.rift_levels, d.rift_levels_max), (0, 0));
        let v2 = grade_progress(Some(&progress(Some(blob(2)), None)), &universe(), true);
        assert_eq!(
            (v2.rift_levels, v2.rift_levels_max),
            (2, 4),
            "V2 rifts unchanged"
        );
        assert_eq!(grade_progress(None, &universe(), true).topic_id, None);
    }

    #[test]
    fn template_shapes_name_every_key_but_v2_by_structure_only() {
        let template = json!({
            "SANDBOX_V2": { "sandbox_1": blob(3) },
            "SANDBOX_V3": { "sandbox_2": {
                "base": { "baseLv": 4 },
                "collect": { "complete": { "achievement": ["x"] } },
                "tasks": [1, 2, 3],
                "day": 12,
            } },
            "SANDBOX_V9": [1, 2],
        });
        assert_eq!(
            template_shapes(&template),
            vec![
                "SANDBOX_V3.sandbox_2 {base{baseLv}, collect{complete}, day, tasks[]}".to_owned(),
                "SANDBOX_V9: array".to_owned(),
            ]
        );
        let v2_only = json!({ "SANDBOX_V2": { "sandbox_1": blob(3) } });
        assert!(
            template_shapes(&v2_only).is_empty(),
            "a V2-only save logs nothing"
        );
        assert_eq!(
            template_shapes(&json!([])),
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
    fn no_progress_reports_the_v2_totals() {
        let d = grade_progress(None, &universe(), true);
        assert_eq!(d.total.to_bits(), 0.0_f64.to_bits());
        assert_eq!(d.achievements_total, 10);
        assert_eq!(d.nodes_total, 10);
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
