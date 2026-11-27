use myrtle_gamedata::model::sandbox_universe::{SandboxTemplate, SandboxUniverse};
use serde_json::{Value, json};

/// First rows of CN's `sandbox_perm_table`, both topics.
fn cn_slice() -> Value {
    serde_json::from_str(include_str!(
        "../../gamedata-raw/tests/fixtures/sandbox_perm_cn_slice.json"
    ))
    .expect("fixture parses")
}

#[test]
fn every_basic_info_topic_is_read_under_its_own_template() {
    let u = SandboxUniverse::build(&cn_slice());
    let ids: Vec<(&str, &SandboxTemplate, i64)> = u
        .topics
        .iter()
        .map(|t| (t.topic_id.as_str(), &t.template, t.sort_id))
        .collect();
    assert_eq!(
        ids,
        vec![
            ("sandbox_1", &SandboxTemplate::V2, 1),
            ("sandbox_2", &SandboxTemplate::V3, 2),
        ]
    );

    let v3 = &u.topics[1].counts;
    assert_eq!(v3.max_achievements, 3, "AchievementData rows");
    assert_eq!(v3.max_nodes, 4, "MainMapData.Nodes without HOME");
    assert_eq!(v3.max_zones, 2, "zones of those nodes, not the HOME zone");
    assert_eq!(v3.node_zones["sandbox_2_node_04"], "z_2_2");
    assert_eq!(v3.max_tech_nodes, 4, "DevelopmentData");
    assert_eq!(v3.max_quests, 2, "ArchiveQuestData");
    assert_eq!(v3.max_stages, 2, "StageData");
    assert_eq!(v3.max_recipes, 2, "CookbookData");
    assert_eq!(v3.max_music, 2, "ArchiveMusicUnlockData");
    assert_eq!(v3.max_base_level, 3, "BasementUpdateDatas");
    assert_eq!(v3.max_blueprints, 0, "run-scoped recipes are not graded");
    assert_eq!(v3.max_band_levels, 6, "2 bands x 3 levels");
    assert_eq!(v3.max_rifts, 0, "V3 has no fixed rifts");
    assert_eq!(u.topics[1].name, "重启锚点", "BasicInfo.TopicName");
    assert!(v3.topics.is_empty());
}

#[test]
fn flat_totals_stay_the_first_v2_topic() {
    let raw = cn_slice();
    let u = SandboxUniverse::build(&raw);
    // The same table without its V3 half: what the grader read before.
    let mut v2_only = raw;
    v2_only["Detail"]
        .as_object_mut()
        .expect("Detail")
        .remove("SANDBOX_V3");
    v2_only["BasicInfo"]
        .as_array_mut()
        .expect("BasicInfo")
        .truncate(1);
    let before = SandboxUniverse::build(&v2_only);

    let flat = |x: &SandboxUniverse| SandboxUniverse {
        topics: Vec::new(),
        ..x.clone()
    };
    assert_eq!(flat(&u), flat(&before), "V3 does not move a V2 total");
    assert_eq!(flat(&u), u.topics[0].counts, "flat fields are sandbox_1's");
    assert_eq!(u.max_nodes, 3, "the `_main` map only, not the hunt variant");
    assert_eq!(u.max_zones, 2, "tutorial zone excluded");
}

#[test]
fn a_template_with_no_reader_is_listed_with_zero_totals() {
    let raw = json!({
        "BasicInfo": [{ "key": "sandbox_9", "value": {
            "TopicId": "sandbox_9", "TopicTemplate": "SANDBOX_V4", "SortId": 9,
        } }],
        "Detail": { "SANDBOX_V4": [{ "key": "sandbox_9", "value": { "AchievementData": [{}] } }] },
    });
    let u = SandboxUniverse::build(&raw);
    assert_eq!(u.topics.len(), 1);
    assert_eq!(
        u.topics[0].template,
        SandboxTemplate::Other("SANDBOX_V4".into())
    );
    assert_eq!(u.topics[0].counts, SandboxUniverse::default());
}

#[test]
fn no_basic_info_falls_back_to_the_detail_entries() {
    let raw = json!({ "Detail": {
        "SANDBOX_V3": [{ "key": "sandbox_2", "value": { "AchievementData": [{}, {}] } }],
    } });
    let u = SandboxUniverse::build(&raw);
    assert_eq!(u.topics.len(), 1);
    assert_eq!(u.topics[0].topic_id, "sandbox_2");
    assert_eq!(u.topics[0].counts.max_achievements, 2);
}

#[test]
fn max_zones_excludes_the_tutorial_zone() {
    let raw = json!({ "Detail": { "SANDBOX_V2": [ { "value": {
        "ZoneData": [
            { "key": "z_1_0" }, { "key": "z_1_1" }, { "key": "z_1_tutorial" }, { "key": "z_1_2" },
        ],
    } } ] } });
    let u = SandboxUniverse::build(&raw);
    assert_eq!(
        u.max_zones, 3,
        "the tutorial zone is excluded from the zone total"
    );
}

#[test]
fn max_blueprints_does_not_double_count_buildings() {
    // `BuildingItemData` ids all reappear in `CraftItemData`; the total is
    // the `CraftItemData` count, never the sum of the two.
    let raw = json!({ "Detail": { "SANDBOX_V2": [ { "value": {
        "BuildingItemData": [
            { "key": "sandbox_1_building_1" }, { "key": "sandbox_1_building_2" },
        ],
        "CraftItemData": [
            { "key": "sandbox_1_building_1" }, { "key": "sandbox_1_building_2" },
            { "key": "sandbox_1_tactical_1" },
        ],
    } } ] } });
    let u = SandboxUniverse::build(&raw);
    assert_eq!(
        u.max_blueprints, 3,
        "blueprints total is the craftable set, not craft + building"
    );
}
