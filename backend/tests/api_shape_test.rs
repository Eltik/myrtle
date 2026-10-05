//! Snapshot of the JSON shape of every `/static/{resource}` payload.
//!
//! The frontend hand-writes ~300 interfaces and casts with `as IWhatever`, so a
//! one-sided rename is invisible to both compilers. `voice_url` -> `voiceURL` was
//! renamed in the TS type alone and silently killed every voice line on both
//! servers; nothing failed, nothing logged.
//!
//! This pins the BACKEND half (all 257 `Serialize` types): a rename, removal or
//! type change shows up as a snapshot diff. A TS-only rename needs generated TS
//! types to catch; this snapshot is their future input.
//!
//! IMPORTANT: this is the WIRE format. `frontend/src/lib/api/operators.ts` runs
//! `deepCamelize` over the operator payloads (`AttributesKeyFrames`, `MaxHp`,
//! `Type_` -> camelCase), so `PascalCase` paths here are expected for the
//! `character_table` mirrors; do not "fix" them by renaming Rust fields. Endpoints
//! WITHOUT that pass (voices, materials) must match the TS types key-for-key.
//!
//! The shape also depends on which assets are on disk: `Option` fields go `null`
//! when their image is absent, the chibi index is empty without spine files, and
//! the banner rate tables come from a file the live `gacha/getPoolDetail` job
//! writes. One snapshot per asset profile, chosen by `API_SHAPE_PROFILE`:
//!
//! ```text
//!   (unset)         tests/snapshots/api_shape.txt
//!                   a full local install (images, spine, pool details)
//!   gamedata-only   tests/snapshots/api_shape.gamedata-only.txt
//!                   the `game-data` artifact assets-ci.yml uploads (tables
//!                   only), which is what backend-ci.yml runs against
//! ```
//!
//! The diff between the two files is exactly the asset-derived wire fields.
//! Refresh after an intentional change:
//!
//! ```text
//!   UPDATE_API_SHAPE=1 cargo test --test api_shape_test
//!   UPDATE_API_SHAPE=1 API_SHAPE_PROFILE=gamedata-only \
//!     GAME_DATA_DIR=<artifact>/gamedata/excel ASSETS_DIR=<artifact> \
//!     cargo test --test api_shape_test
//! ```
//!
//! (`gh run download <assets-ci run id> -n game-data -D <artifact>/gamedata`
//! fetches the artifact) and review both diffs in the PR. A line disappearing
//! from either is a breaking change for that field's consumers.

mod common;

use std::collections::BTreeSet;
use std::fmt::Write as _;

use serde_json::Value;

// No sampling: every container is walked in full. A cap of 200 children made
// the test FLAKY: several payloads iterate a `HashMap`, so whether the first 200
// held a `null` for an `Option` field changed run to run (phantom
// `stage-index[*].name: null` diffs). Walking everything costs seconds of CPU and
// a few thousand paths of memory; determinism is worth it.

const fn type_name(v: &Value) -> &'static str {
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
/// schema and stays stable when the game data itself changes. A new operator,
/// event, or item must not churn this file.
fn walk(path: &str, value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            out.insert(format!("{path}: object"));
            let record = is_record_map(map);
            for (k, v) in map {
                // A record keyed by id (operators, items) collapses to `*`; a
                // struct's own field names are the thing we are pinning, so they
                // are kept verbatim. The per-key fallback (long, `_`, `#`) is for
                // containers the map test cannot decide.
                let key = if record || k.len() > 24 || k.contains('_') || k.contains('#') {
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

/// Whether an object is a `HashMap` keyed by game data rather than a struct.
///
/// Decided for the whole container, not per key, because the per-key rule
/// cannot tell `act51side` (an activity id) from `endTime` (a field). Two
/// signals, either one suffices:
///
/// 1. A key no Rust field can have: digit-led (`30011`, `1stact`), or
///    containing `-`, `#`, or a space (`act2vmulti-tr03`, `char_002_amiya#1`).
///    `_` is deliberately NOT in this list: four struct fields in the operator
///    payload carry one, and a container-level `_` rule would erase the whole
///    operator shape.
/// 2. Every value is an object with the identical key set, and there are at
///    least two of them (`subProfDict`, `brandList`, `raceData`). A struct
///    whose fields are all same-shaped sub-objects would trip this too; there
///    is none in the twenty payloads (checked 2026-09-20: every hit was a
///    `HashMap<String, _>`).
fn is_record_map(map: &serde_json::Map<String, Value>) -> bool {
    if map.keys().any(|k| {
        k.starts_with(|c: char| c.is_ascii_digit())
            || k.contains('-')
            || k.contains('#')
            || k.contains(' ')
    }) {
        return true;
    }
    if map.len() < 2 {
        return false;
    }
    let mut shapes = map
        .values()
        .map(|v| v.as_object().map(|o| o.keys().collect::<BTreeSet<_>>()));
    match shapes.next() {
        Some(Some(first)) if !first.is_empty() => shapes.all(|s| s.as_ref() == Some(&first)),
        _ => false,
    }
}

/// Names of every resource `serialize_resource` in `app/services/static_data.rs`
/// serves. Keep in step with that match: a resource missing here is unpinned.
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
    "enemy-stage-table",
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
        "enemy-stage-table" => serde_json::to_value(
            backend::core::gamedata::types::enemy_stages::EnemyStageTable::from_index(
                &gd.enemy_stage_index,
            ),
        ),
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

    let profile = std::env::var("API_SHAPE_PROFILE")
        .ok()
        .filter(|p| !p.trim().is_empty());
    let snapshot_path = profile.as_deref().map_or_else(
        || concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots/api_shape.txt").to_string(),
        |p| {
            format!(
                "{}/tests/snapshots/api_shape.{p}.txt",
                env!("CARGO_MANIFEST_DIR")
            )
        },
    );
    eprintln!(
        "api shape profile: {} -> {snapshot_path}",
        profile.as_deref().unwrap_or("(full install)")
    );

    if std::env::var("UPDATE_API_SHAPE").is_ok() {
        std::fs::create_dir_all(
            std::path::Path::new(&snapshot_path)
                .parent()
                .expect("snapshot dir"),
        )
        .expect("create snapshot dir");
        std::fs::write(&snapshot_path, &rendered).expect("write snapshot");
        eprintln!("updated {snapshot_path}");
        return;
    }

    let Ok(expected) = std::fs::read_to_string(&snapshot_path) else {
        panic!(
            "no API shape snapshot at {snapshot_path}\n\
             create it with: UPDATE_API_SHAPE=1{} cargo test --test api_shape_test",
            profile
                .as_deref()
                .map_or_else(String::new, |p| format!(" API_SHAPE_PROFILE={p}"))
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
         If intended: UPDATE_API_SHAPE=1{} cargo test --test api_shape_test\n\
         (snapshot: {snapshot_path})",
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
        profile
            .as_deref()
            .map_or_else(String::new, |p| format!(" API_SHAPE_PROFILE={p}")),
    );
}
