#![allow(clippy::unwrap_used, clippy::expect_used)]
use myrtle_engine_grade::operators::parse_modules;

#[test]
fn locked_modules_are_ignored() {
    let json = serde_json::json!([
        { "id": "uniequip_002_a", "level": 1, "locked": true },
        { "id": "uniequip_003_a", "level": 2, "locked": false },
        { "id": "uniequip_001_a", "level": 1 }
    ]);
    let ids: Vec<String> = parse_modules(&json).into_iter().map(|m| m.id).collect();
    assert_eq!(ids, ["uniequip_003_a", "uniequip_001_a"]);
}
