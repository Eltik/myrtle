#![allow(clippy::unwrap_used, clippy::expect_used)]
use myrtle_engine_grade::sandbox::score::{
    GRADE_SANDBOX_V3, admit_new_shapes, grade_progress, score_content_depth, score_exploration,
    template_shapes,
};
use myrtle_gamedata_raw::tables::sandbox_universe::{
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
    const { assert!(GRADE_SANDBOX_V3, "shipped on") };
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
