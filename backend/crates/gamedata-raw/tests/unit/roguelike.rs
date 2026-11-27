use serde_json::json;

use crate::tables::roguelike::{
    RoguelikeGameData, RoguelikeTopicTableFile, SpecialEnding, special_endings,
};

fn detail(endings: serde_json::Value) -> serde_json::Value {
    json!({
        "Endings": endings,
        "Difficulties": [
            { "Name": "Regular", "ModeDifficulty": "NORMAL" },
            { "Name": "Deep Investigation", "ModeDifficulty": "CHALLENGE" },
        ],
    })
}

fn ending(id: &str, name: &str, priority: i64) -> serde_json::Value {
    json!({ "key": id, "value": { "Id": id, "Name": name, "Priority": priority } })
}

#[test]
fn an_ending_ranked_past_the_list_is_special() {
    let d = detail(json!([
        ending("e1", "One", 0),
        ending("e2", "Two", 1),
        ending("e_c", "Extra", 99),
        ending("e3", "Three", 2),
    ]));
    assert_eq!(
        special_endings(&d),
        vec![SpecialEnding {
            id: "e_c".into(),
            name: "Extra".into(),
            mode_name: Some("Deep Investigation".into()),
        }]
    );
}

#[test]
fn a_contiguous_ranking_has_no_special_ending() {
    let d = detail(json!([
        ending("e1", "One", 0),
        ending("e2", "Two", 1),
        ending("e3", "Three", 2)
    ]));
    assert!(special_endings(&d).is_empty());
}

#[test]
fn a_missing_rank_reads_as_regular_and_cn_dicts_parse() {
    // CN-gamedata shape: camelCase keys and `{id: ending}` dicts, no difficulties.
    let d = json!({ "endings": {
        "e1": { "name": "One", "priority": 0 },
        "e2": { "name": "Two" },
        "e3": { "name": "Extra", "priority": 7 },
    }});
    assert_eq!(
        special_endings(&d),
        vec![SpecialEnding {
            id: "e3".into(),
            name: "Extra".into(),
            mode_name: None
        }]
    );
}

/// The EN table under `MYRTLE_TEST_ASSETS`, else the repo's `assets/output`
fn load_en() -> Option<RoguelikeGameData> {
    let assets = std::env::var_os("MYRTLE_TEST_ASSETS").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../assets/output"),
        std::path::PathBuf::from,
    );
    let dir = assets.join("en/gamedata/excel");
    if !dir.join("roguelike_topic_table.json").exists() {
        return None;
    }
    let file: RoguelikeTopicTableFile =
        serde_json::from_slice(&std::fs::read(dir.join("roguelike_topic_table.json")).ok()?)
            .ok()?;
    Some(RoguelikeGameData::from_table(&file))
}

#[test]
fn en_lists_one_special_ending_on_rogue_3_and_keeps_every_count() {
    let Some(data) = load_en() else {
        eprintln!("skipping: EN roguelike_topic_table.json not present");
        return;
    };
    let rogue_3 = &data.themes["rogue_3"];
    assert_eq!(
        rogue_3.special_endings,
        vec![SpecialEnding {
            id: "ro3_ending_c".into(),
            name: "The Fleetest of Light".into(),
            mode_name: Some("Deep Investigation".into()),
        }]
    );
    // The count is the array length either way: 4 regular + 1 special.
    assert_eq!(rogue_3.max_endings, 5);
    // `rogue_4`/`rogue_5` list five REGULAR endings each (Priority 0..4),
    // so they get no note.
    for (theme_id, max) in [
        ("rogue_1", 4),
        ("rogue_2", 4),
        ("rogue_4", 5),
        ("rogue_5", 5),
    ] {
        let theme = &data.themes[theme_id];
        assert!(
            theme.special_endings.is_empty(),
            "{theme_id}: {:?}",
            theme.special_endings
        );
        assert_eq!(theme.max_endings, max, "{theme_id}");
    }
}
