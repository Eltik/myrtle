#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use myrtle_parity::corpus::{ExpandOptions, expand_openapi, skipped_by_reason};
use myrtle_parity::ids::{IdSet, extract_from_excel};
use myrtle_parity::openapi::check_superset;

fn fixture_excel() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/excel")
}

fn snapshot() -> serde_json::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/openapi.json");
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

#[test]
fn ids_from_flatbuffer_excel() {
    let (ids, report) = extract_from_excel(&fixture_excel()).unwrap();
    assert_eq!(report.tables_read.len(), 11);
    assert!(
        report.tables_missing.is_empty(),
        "{:?}",
        report.tables_missing
    );
    assert_eq!(
        ids.values("operator"),
        ["char_002_amiya", "char_003_kalts", "char_1001_amiya2"]
    );
    assert_eq!(
        ids.values("skin"),
        ["char_002_amiya@winter#1", "char_003_kalts@test#2"]
    );
    assert_eq!(ids.values("brand"), ["winter"]);
    assert_eq!(ids.values("brand_kv"), ["kv_winter_1", "kv_winter_2"]);
    assert_eq!(
        ids.values("skill_icon"),
        ["skchr_amiya_2", "skcom_magic_rage[2]"]
    );
    assert_eq!(
        ids.values("module_icon"),
        ["original", "uniequip_002_amiya"]
    );
    assert_eq!(ids.values("item"), ["30012", "4002"]);
    assert_eq!(ids.values("medal"), ["medal_activity_1", "medal_story_2"]);
    assert_eq!(
        ids.values("enemy"),
        ["enemy_1000_gopro", "enemy_1007_slime"]
    );
    assert_eq!(ids.values("stage"), ["act13side_01", "main_01-07"]);
    assert_eq!(ids.values("story_group"), ["1stact", "act13mini", "main_0"]);
    assert_eq!(ids.values("activity"), ["1stact", "act13mini"]);
    assert_eq!(
        ids.values("story"),
        [
            "1stact_level_a001_01_beg",
            "1stact_level_a001_01_end",
            "main_00-01_beg"
        ]
    );
    assert_eq!(ids.values("gacha_pool"), ["LIMITED_76_0_1", "NORM_1_0_1"]);
    assert_eq!(ids.values("server"), ["cn", "en", "jp", "kr", "tw"]);
    assert_eq!(ids.values("static_resource").len(), 21);
    assert!(ids.kinds.contains_key("uid") && ids.values("uid").is_empty());

    let dir = std::env::temp_dir().join(format!("parity-ids-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("ids.json");
    std::fs::write(&file, serde_json::to_vec_pretty(&ids).unwrap()).unwrap();
    assert_eq!(IdSet::load(&file).unwrap(), ids);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn missing_tables_leave_kinds_empty() {
    let dir = std::env::temp_dir().join(format!("parity-empty-excel-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (ids, report) = extract_from_excel(&dir).unwrap();
    assert!(report.tables_read.is_empty());
    assert_eq!(report.tables_missing.len(), 11);
    assert!(ids.values("operator").is_empty());
    assert_eq!(ids.values("server").len(), 5);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn snapshot_shape_and_superset_of_itself() {
    let doc = snapshot();
    let report = check_superset(&doc, &doc);
    assert!(report.ok());
    assert_eq!(report.old_operations, 303);
    assert_eq!(report.old_schemas, 578);
}

#[test]
fn snapshot_expansion_is_deterministic_and_explains_every_skip() {
    let doc = snapshot();
    let (mut ids, _) = extract_from_excel(&fixture_excel()).unwrap();
    let mut extra = IdSet::default();
    extra.kinds.insert("uid".to_owned(), vec!["1".to_owned()]);
    ids.merge(extra);

    let opts = ExpandOptions {
        sample: Some(3),
        seed: 11,
        roles: vec!["user".to_owned()],
        ..ExpandOptions::default()
    };
    let first = expand_openapi(&doc, &ids, &opts);
    let second = expand_openapi(&doc, &ids, &opts);
    assert_eq!(first.entries, second.entries);

    let gets = doc["paths"]
        .as_object()
        .unwrap()
        .values()
        .filter(|i| i.get("get").is_some())
        .count();
    assert_eq!(first.plans.len(), gets);
    assert_eq!(gets, 213);
    for plan in &first.plans {
        assert!(plan.emitted > 0 || plan.skipped.is_some(), "{plan:?}");
        assert!(plan.emitted <= 3 * 2, "{plan:?}");
    }
    for route in [
        "/api/operators/{id}",
        "/api/{server}/operators/{id}",
        "/api/skin-portrait/{id}",
        "/api/{server}/static/{resource}",
        "/api/story/{story_id}",
        "/api/story/art-gallery/{kind}",
        "/api/get-user",
        "/api/gacha/stats",
        "/api/admin/stats",
        "/api/leaderboard/items/standing",
    ] {
        let plan = first.plans.iter().find(|p| p.route == route).unwrap();
        assert!(plan.skipped.is_none(), "{plan:?}");
    }
    let tier = first
        .plans
        .iter()
        .find(|p| p.route == "/api/tier-lists/{slug}")
        .unwrap();
    assert!(tier.skipped.as_deref().unwrap().contains("tier_list_slug"));
    assert!(
        first
            .entries
            .iter()
            .filter(|e| e.route == "/api/skin-portrait/{id}")
            .all(|e| !e.path.contains('#'))
    );
    let get_user: Vec<_> = first
        .entries
        .iter()
        .filter(|e| e.route == "/api/get-user")
        .collect();
    assert_eq!(get_user.len(), 2);
    assert!(
        get_user.iter().any(|e| e.role.is_none())
            && get_user.iter().any(|e| e.role.as_deref() == Some("user"))
    );

    let full = expand_openapi(&doc, &ids, &ExpandOptions::default());
    let per_server = full
        .entries
        .iter()
        .filter(|e| e.route == "/api/{server}/operators/{id}")
        .count();
    assert_eq!(per_server, 5 * 3);
}

#[test]
fn every_snapshot_parameter_has_an_id_kind() {
    let doc = snapshot();
    let (mut ids, _) = extract_from_excel(&fixture_excel()).unwrap();
    for values in ids.kinds.values_mut() {
        if values.is_empty() {
            values.push("x".to_owned());
        }
    }
    let opts = ExpandOptions {
        sample: Some(1),
        roles: vec!["user".to_owned()],
        ..ExpandOptions::default()
    };
    let exp = expand_openapi(&doc, &ids, &opts);
    let skipped = skipped_by_reason(&exp.plans);
    assert!(skipped.is_empty(), "{skipped:#?}");
    assert_eq!(exp.plans.len(), 213);
}
