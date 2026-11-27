use std::borrow::Cow;
use std::collections::BTreeMap;

use myrtle_parity::ids::{
    Binding, IdSet, MANUAL_KINDS, builtin_kind, extract_from_excel, sanitize_lone_surrogates,
};
use serde_json::Value;

#[test]
fn builtin_kinds_follow_route_context() {
    assert_eq!(builtin_kind("/api/operators/{id}", "id"), Some("operator"));
    assert_eq!(
        builtin_kind("/api/{server}/operators/{id}/build-stats", "id"),
        Some("operator")
    );
    assert_eq!(builtin_kind("/api/skins/{id}", "id"), Some("operator"));
    assert_eq!(builtin_kind("/api/skin-portrait/{id}", "id"), Some("skin"));
    assert_eq!(
        builtin_kind("/api/enemies/{id}/stages", "id"),
        Some("enemy")
    );
    assert_eq!(
        builtin_kind("/api/{server}/story/art-gallery/{kind}/{id}/{size}", "id"),
        Some("story_art")
    );
    assert_eq!(
        builtin_kind("/api/story/gallery/{id}/{size}", "id"),
        Some("story_gallery")
    );
    assert_eq!(
        builtin_kind("/api/music/siren/albums/{cid}", "cid"),
        Some("siren_album")
    );
    assert_eq!(
        builtin_kind("/api/music/siren/songs/{cid}/lyrics", "cid"),
        Some("siren_song")
    );
    assert_eq!(
        builtin_kind("/api/tier-lists/{slug}/stats", "slug"),
        Some("tier_list_slug")
    );
    assert_eq!(builtin_kind("/api/grids/{slug}", "slug"), Some("grid_slug"));
    assert_eq!(
        builtin_kind("/api/{server}/assets/{*path}", "path"),
        Some("asset_path")
    );
    assert_eq!(builtin_kind("/api/x/{id}", "id"), None);
}

#[test]
fn lookback_snapshot_ids_are_a_manual_kind() {
    assert_eq!(
        builtin_kind("/api/lookback/snapshots/{id}", "id"),
        Some("lookback_snapshot")
    );
    assert!(MANUAL_KINDS.contains(&"lookback_snapshot"));
    let dir = std::env::temp_dir().join(format!("parity-ids-lookback-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (ids, _) = extract_from_excel(&dir).unwrap();
    assert!(ids.kinds.contains_key("lookback_snapshot"));
    assert!(ids.values("lookback_snapshot").is_empty());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn bindings_override_builtin() {
    let ids = IdSet {
        kinds: BTreeMap::new(),
        bindings: vec![Binding {
            route: "/api/avatar/{id}".to_owned(),
            param: "id".to_owned(),
            kind: "skin".to_owned(),
        }],
    };
    assert_eq!(
        ids.kind_for("/api/avatar/{id}", "id").as_deref(),
        Some("skin")
    );
    assert_eq!(
        ids.kind_for("/api/portrait/{id}", "id").as_deref(),
        Some("operator")
    );
}

#[test]
fn merge_unions_and_sorts() {
    let mut a = IdSet::default();
    a.kinds
        .insert("uid".to_owned(), vec!["2".to_owned(), "1".to_owned()]);
    let mut b = IdSet::default();
    b.kinds.insert(
        "uid".to_owned(),
        vec!["1".to_owned(), "3".to_owned(), String::new()],
    );
    a.merge(b);
    assert_eq!(a.values("uid"), ["1", "2", "3"]);
}

#[test]
fn sanitizer_replaces_only_lone_surrogates() {
    let ok = r#"{"a":"\ud83d\ude00","b":"\\ud800 is text"}"#;
    assert!(matches!(sanitize_lone_surrogates(ok), Cow::Borrowed(_)));
    let bad = r#"{"a":"x\ud83dy","b":"\udc00","c":"\ud83d\ude00"}"#;
    let fixed = sanitize_lone_surrogates(bad);
    assert_eq!(fixed, r#"{"a":"x\uFFFDy","b":"\uFFFD","c":"\ud83d\ude00"}"#);
    let v: Value = serde_json::from_str(&fixed).unwrap();
    assert_eq!(v["a"], "x\u{FFFD}y");
    assert!(serde_json::from_str::<Value>(bad).is_err());
}

#[test]
fn entries_reads_objects_and_flatbuffer_arrays() {
    let dir = std::env::temp_dir().join(format!("parity-ids-shapes-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let write = |name: &str, body: &str| std::fs::write(dir.join(name), body).unwrap();
    write(
        "character_table.json",
        r#"{"characters": {"char_b": {}, "char_a": {}, "token_x": {}}}"#,
    );
    write(
        "item_table.json",
        r#"{"Items": [{"Key": "a", "Value": 1}, {"key": 7, "value": 2}]}"#,
    );
    write(
        "story_review_table.json",
        r#"{"Story_reviews": [{"Key": "main_0", "Value": {"EntryType": "MAINLINE"}}]}"#,
    );
    let (ids, report) = extract_from_excel(&dir).unwrap();
    assert_eq!(report.tables_read.len(), 3);
    assert_eq!(ids.values("operator"), ["char_a", "char_b"]);
    assert_eq!(ids.values("item"), ["7", "a"]);
    assert_eq!(ids.values("story_group"), ["main_0"]);
    assert!(ids.values("activity").is_empty());
    std::fs::remove_dir_all(&dir).unwrap();
}
