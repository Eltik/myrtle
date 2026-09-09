//! A committed snapshot of the JSON shape of every `/static/{resource}` payload.
//!
//! Motivation: the frontend hand-writes ~300 interfaces describing these
//! payloads and casts responses with `as IWhatever`, so a field renamed on one
//! side and not the other is invisible to both compilers. `voice_url` →
//! `voiceURL` was renamed in the TS type alone and silently killed every voice
//! line on both servers; nothing failed, nothing logged.
//!
//! This test pins the BACKEND half: any rename, removal, or type change in a
//! serialized field shows up as a snapshot diff at the moment it is made,
//! covering all 257 `Serialize` types automatically rather than a hand-picked
//! few. It does not, and cannot, catch a rename made only in the TypeScript —
//! for that the TS types have to be generated from these structs rather than
//! written by hand. The snapshot is the input that makes that generation
//! possible later.
//!
//! IMPORTANT: this pins the WIRE format, which is not what the frontend's types
//! describe. `frontend/src/lib/api/operators.ts` runs `deepCamelize` over the
//! operator payloads, converting PascalCase and trailing-underscore keys
//! (`AttributesKeyFrames`, `MaxHp`, `Type_`) to camelCase before any consumer
//! sees them. So a PascalCase path in this snapshot is correct and expected for
//! the structs that mirror `character_table`; do not "fix" it by renaming the
//! Rust fields. Endpoints WITHOUT that normalization — voices, materials — are
//! the ones where this snapshot and the TS types must agree key-for-key.
//!
//! Refresh after an intentional change:
//!   UPDATE_API_SHAPE=1 cargo test --test api_shape_test
//! and review the diff as part of the PR — a line disappearing from it is a
//! breaking change for every consumer of that field.

mod common;

use std::collections::BTreeSet;
use std::fmt::Write as _;

use serde_json::Value;

// No sampling: every container is walked in full.
//
// An earlier version capped children at 200 per container, on the theory that
// the shape comes from the Rust types rather than the data. That made the test
// FLAKY. Several of these payloads are built by iterating a `HashMap`, so their
// element order changes run to run; whether the first 200 entries happened to
// include a `null` for an `Option` field then decided whether that line made it
// into the snapshot. It surfaced as phantom diffs like `stage-index[*].name:
// null` appearing and disappearing with nothing having changed.
//
// Walking everything is O(payload) CPU and adds only the path set (a few
// thousand entries) to memory, since resources are built and dropped one at a
// time. Determinism is worth more than the seconds.

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Record every reachable key path and the set of JSON types seen at it.
///
/// Map keys and array indices collapse to `*`, so the snapshot describes the
/// schema and stays stable when the game data itself changes — a new operator
/// must not churn this file.
fn walk(path: &str, value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            out.insert(format!("{path}: object"));
            for (k, v) in map {
                // A record keyed by id (operators, items) collapses to `*`; a
                // struct's own field names are the thing we are pinning, so they
                // are kept verbatim. Heuristic: ids are long or contain `_`/`#`.
                let key = if k.len() > 24 || k.contains('_') || k.contains('#') {
                    "*"
                } else {
                    k.as_str()
                };
                walk(&format!("{path}.{key}"), v, out);
            }
        }
        Value::Array(items) => {
            out.insert(format!("{path}: array"));
            for v in items {
                walk(&format!("{path}[*]"), v, out);
            }
        }
        leaf => {
            out.insert(format!("{path}: {}", type_name(leaf)));
        }
    }
}

/// Names of every resource `serialize_resource` in `app/services/static_data.rs`
/// serves. Keep in step with that match — a resource missing here is unpinned.
const RESOURCES: &[&str] = &[
    "operators",
    "skills",
    "modules",
    "skins",
    "materials",
    "stages",
    "zones",
    "activities",
    "retro_acts",
    "enemies",
    "enemy-stages",
    "stage-index",
    "gacha",
    "banners",
    "voices",
    "handbook",
    "chibis",
    "enemy-chibis",
    "trust",
    "ranges",
];

/// Serialize one resource. Built on demand and dropped by the caller before the
/// next: these payloads run to hundreds of MB combined, and holding all twenty
/// as `Value` trees at once is enough to OOM a small CI box.
fn resource_value(gd: &backend::core::gamedata::types::GameData, name: &str) -> Value {
    match name {
        "operators" => serde_json::to_value(&gd.operators),
        "skills" => serde_json::to_value(&gd.skills),
        "modules" => serde_json::to_value(&gd.modules),
        "skins" => serde_json::to_value(&gd.skins),
        "materials" => serde_json::to_value(&gd.materials),
        "stages" => serde_json::to_value(&gd.stages),
        "zones" => serde_json::to_value(&gd.zones),
        "activities" => serde_json::to_value(&gd.activities),
        "retro_acts" => serde_json::to_value(&gd.retro_acts),
        "enemies" => serde_json::to_value(&gd.enemies),
        "enemy-stages" => serde_json::to_value(&gd.enemy_stage_index),
        "stage-index" => serde_json::to_value(&gd.stage_index),
        "gacha" => serde_json::to_value(&gd.gacha),
        "banners" => serde_json::to_value(&gd.gacha.gacha_pool_client),
        "voices" => serde_json::to_value(&gd.voices),
        "handbook" => serde_json::to_value(&gd.handbook),
        "chibis" => serde_json::to_value(&gd.chibis),
        "enemy-chibis" => serde_json::to_value(&gd.enemy_chibis),
        "trust" => serde_json::to_value(&gd.favor),
        "ranges" => serde_json::to_value(&gd.ranges),
        other => panic!("unknown resource {other}"),
    }
    .expect("serialize resource")
}

#[test]
fn static_payload_shapes_are_unchanged() {
    let gd = common::load_game_data();

    let mut rendered = String::new();
    for name in RESOURCES {
        let mut paths = BTreeSet::new();
        {
            let value = resource_value(gd, name);
            walk(name, &value, &mut paths);
        } // value dropped here, before the next resource is built
        for p in paths {
            let _ = writeln!(rendered, "{p}");
        }
    }

    let snapshot_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots/api_shape.txt");

    if std::env::var("UPDATE_API_SHAPE").is_ok() {
        std::fs::create_dir_all(
            std::path::Path::new(snapshot_path)
                .parent()
                .expect("snapshot dir"),
        )
        .expect("create snapshot dir");
        std::fs::write(snapshot_path, &rendered).expect("write snapshot");
        eprintln!("updated {snapshot_path}");
        return;
    }

    let Ok(expected) = std::fs::read_to_string(snapshot_path) else {
        panic!(
            "no API shape snapshot at {snapshot_path}\n\
             create it with: UPDATE_API_SHAPE=1 cargo test --test api_shape_test"
        );
    };

    if expected == rendered {
        return;
    }

    let old: BTreeSet<&str> = expected.lines().collect();
    let new: BTreeSet<&str> = rendered.lines().collect();
    let removed: Vec<&&str> = old.difference(&new).take(40).collect();
    let added: Vec<&&str> = new.difference(&old).take(40).collect();

    panic!(
        "API payload shape changed.\n\n\
         REMOVED (breaking for any consumer reading these):\n  {}\n\n\
         ADDED:\n  {}\n\n\
         If intended: UPDATE_API_SHAPE=1 cargo test --test api_shape_test",
        removed
            .iter()
            .map(|s| (**s).to_string())
            .collect::<Vec<_>>()
            .join("\n  "),
        added
            .iter()
            .map(|s| (**s).to_string())
            .collect::<Vec<_>>()
            .join("\n  "),
    );
}
