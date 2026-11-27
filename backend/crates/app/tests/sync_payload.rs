#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use myrtle_app::services::sync::payload::{
    EXPEDITED_PLAN_ITEM, UNIVERSAL_CERTIFICATE_ITEM, extract_game_squads, extract_items,
    extract_nick_number, extract_originite, is_medal_earned, merge_campaign_clears,
    merge_cow_level,
};
use myrtle_app::services::sync::sandbox::{MissingTopic, blob_from_change, missing_topics};
use myrtle_gamedata_raw::tables::campaign::{CampaignRotations, CampaignTableFile};
use myrtle_gamedata_raw::tables::module::{Module, ModuleType, Modules};
use serde_json::{Value, json};

const NOW: i64 = 1_800_000_000;

fn rotations() -> CampaignRotations {
    let table: CampaignTableFile = serde_json::from_value(json!({
        "Campaigns": [
            {"key": "camp_01", "value": {"BreakLadders": [{"KillCnt": 100}, {"KillCnt": 400}]}},
            {"key": "camp_r_01", "value": {"BreakLadders": [{"KillCnt": 400}]}}
        ]
    }))
    .unwrap();
    CampaignRotations::from_table(table)
}

#[test]
fn cow_level_first_opens_ride_on_the_stage_record() {
    let raw = json!({ "user": { "dungeon": { "cowLevel": {
        "spst_08-01": { "id": "spst_08-01", "type": "STAGE", "val": [true], "fts": 1_708_906_788, "rts": 1_708_907_776 },
        "spst_16-01": { "id": "spst_16-01", "type": "STAGE", "val": [], "fts": -1, "rts": -1 },
        "spst_99-01": { "id": "spst_99-01", "type": "STAGE", "val": [true], "fts": 5, "rts": 6 }
    } } } });
    let mut stages = json!({
        "spst_08-01": {"stageId": "spst_08-01", "state": 3, "startTimes": 0, "completeTimes": 0},
        "spst_16-01": {"stageId": "spst_16-01", "state": 3, "startTimes": 0, "completeTimes": 0},
        "main_00-01": {"stageId": "main_00-01", "state": 3, "startTimes": 2, "completeTimes": 1}
    });
    merge_cow_level(&mut stages, &raw);
    assert_eq!(stages["spst_08-01"]["cowFirstTs"], 1_708_906_788);
    assert!(
        stages["spst_16-01"].get("cowFirstTs").is_none(),
        "never opened"
    );
    assert!(stages.get("spst_99-01").is_none(), "nothing is invented");
    assert!(stages["main_00-01"].get("cowFirstTs").is_none());
    let before = stages.clone();
    merge_cow_level(&mut stages, &json!({ "user": {} }));
    assert_eq!(stages, before);
}

#[test]
fn campaign_clears_merge_into_stages() {
    let raw = json!({ "user": { "campaignsV2": { "instances": {
        "camp_01": {"maxKills": 400, "rewardStatus": [1,1,1,1,1,1,1,1]},
        "camp_r_01": {"maxKills": 250, "rewardStatus": [1,1,1,0,0,0,0,0]},
        "camp_r_02": {"maxKills": 0, "rewardStatus": [0,0,0,0,0,0,0,0]}
    } } } });
    let mut stages = json!({
        "main_00-01": {"stageId": "main_00-01", "state": 3, "completeTimes": 1, "practiceTimes": 0}
    });
    merge_campaign_clears(&mut stages, &raw, &rotations());
    assert_eq!(stages["camp_01"]["state"], 3, "kill target reached");
    assert_eq!(stages["camp_01"]["campaignMaxKills"], 400);
    assert_eq!(stages["camp_r_01"]["state"], 2, "partial kills");
    assert!(stages.get("camp_r_02").is_none(), "never played");
    assert_eq!(stages["main_00-01"]["state"], 3);
}

#[test]
fn campaign_merge_handles_missing_stages_and_unknown_maps() {
    let raw = json!({ "user": {"campaignsV2": {"instances": { "camp_r_99": {"maxKills": 400} }}} });
    let mut stages = Value::Null;
    merge_campaign_clears(&mut stages, &raw, &rotations());
    assert_eq!(
        stages["camp_r_99"]["state"], 2,
        "no kill max: partial credit"
    );
}

#[test]
fn campaign_merge_prefers_existing_dungeon_entry() {
    let raw = json!({ "user": {"campaignsV2": {"instances": { "camp_01": {"maxKills": 400} }}} });
    let mut stages = json!({
        "camp_01": {"stageId": "camp_01", "state": 2, "completeTimes": 5, "practiceTimes": 0}
    });
    merge_campaign_clears(&mut stages, &raw, &rotations());
    assert_eq!(stages["camp_01"]["completeTimes"], 5);
    assert_eq!(stages["camp_01"]["state"], 2);
}

fn quantities(items: &Value) -> BTreeMap<String, i64> {
    items
        .as_array()
        .unwrap()
        .iter()
        .map(|i| {
            (
                i["item_id"].as_str().unwrap().to_owned(),
                i["quantity"].as_i64().unwrap(),
            )
        })
        .collect()
}

#[test]
fn items_merge_inventory_consumables_and_status_counters() {
    let inventory = json!({ "30011": 12, "4006": 3 });
    let consumable = json!({
        "voucher_elite_II_4": {
            "1": { "ts": -1, "count": 2 },
            "2": { "ts": NOW + 86_400, "count": 1 },
            "3": { "ts": 1, "count": 7 }
        },
        "voucher_skin": { "9": { "ts": 1, "count": 1 } }
    });
    let got = quantities(&extract_items(
        inventory.as_object(),
        consumable.as_object(),
        &[
            (EXPEDITED_PLAN_ITEM, Some(912)),
            (UNIVERSAL_CERTIFICATE_ITEM, Some(513)),
        ],
        NOW,
    ));
    assert_eq!(got["30011"], 12);
    assert_eq!(got["4006"], 3);
    assert_eq!(
        got["voucher_elite_II_4"], 3,
        "two live instances, one expired"
    );
    assert!(
        !got.contains_key("voucher_skin"),
        "only an expired instance"
    );
    assert_eq!(got["7002"], 912);
    assert_eq!(got["classic_normal_ticket"], 513);
    assert_eq!(got.len(), 5);
}

#[test]
fn items_without_consumables_or_plans_are_the_inventory() {
    let inventory = json!({ "30011": 12 });
    assert_eq!(
        extract_items(inventory.as_object(), None, &[], NOW)
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let none = extract_items(None, None, &[(EXPEDITED_PLAN_ITEM, Some(0))], NOW);
    assert!(
        none.as_array().unwrap().is_empty(),
        "a zero counter is not a row"
    );
}

#[test]
fn game_squads_resolve_seats_and_pad_to_twelve() {
    let troop = json!({
        "chars": {
            "305": {"instId": 305, "charId": "char_4080_lin"},
            "12": {"instId": 12, "charId": "char_002_amiya"}
        },
        "squads": {
            "3": {
                "squadId": "3",
                "name": "Lin",
                "slots": [
                    {"charInstId": 305, "skillIndex": 1, "currentEquip": "uniequip_002_lin"},
                    null,
                    {"charInstId": 999, "skillIndex": 2, "currentEquip": null},
                    {"charInstId": 12, "skillIndex": null, "currentEquip": "uniequip_001_amiya"}
                ]
            },
            "7": {"squadId": "7", "name": "A fifth squad", "slots": []}
        }
    });
    let mut modules = Modules::default();
    for (id, kind) in [
        ("uniequip_001_amiya", ModuleType::Initial),
        ("uniequip_002_lin", ModuleType::Advanced),
    ] {
        let module = Module {
            module_type: kind,
            ..Default::default()
        };
        modules.equip_dict.insert(id.to_owned(), module);
    }
    let mut slots = vec![Value::Null; 12];
    slots[0] =
        json!({"char_id": "char_4080_lin", "skill_index": 1, "module_id": "uniequip_002_lin"});
    slots[3] = json!({"char_id": "char_002_amiya", "skill_index": 0, "module_id": null});
    assert_eq!(
        extract_game_squads(Some(&troop), &modules),
        json!([
            {"index": 3, "name": "Lin", "slots": slots},
            {"index": 7, "name": "A fifth squad", "slots": vec![Value::Null; 12]}
        ])
    );
    assert_eq!(extract_game_squads(None, &modules), json!([]));
}

#[test]
fn medal_earned_rules() {
    assert!(is_medal_earned(&json!(0), 0, 1_700_000_000), "claimed");
    assert!(is_medal_earned(&json!([[30, 30], [2, 1]]), 5, -1));
    assert!(is_medal_earned(&json!([]), 5, -1), "binary completion");
    assert!(!is_medal_earned(&json!([[29, 30]]), 5, -1), "partial");
    assert!(!is_medal_earned(&json!(0), 0, 0), "the unearned default");
    assert!(!is_medal_earned(&json!([]), 0, -1), "never stamped");
    assert!(!is_medal_earned(&Value::Null, 5, -1), "no val");
}

#[test]
fn nick_number_and_originite_paths() {
    assert_eq!(
        extract_nick_number(&json!({"user": {"status": {"nickNumber": "0042"}}})).as_deref(),
        Some("0042"),
        "leading zeros survive"
    );
    assert_eq!(
        extract_nick_number(&json!({"user": {"social": {"nickNumber": 42}}})).as_deref(),
        Some("42")
    );
    assert_eq!(
        extract_nick_number(&json!({"user": {"status": {"nickNumber": ""}, "nickNumber": 7}}))
            .as_deref(),
        Some("7")
    );
    assert_eq!(extract_nick_number(&json!({"user": {}})), None);
    assert_eq!(
        extract_originite(&json!({"user": {"status": {"payDiamond": 3, "freeDiamond": 4}}})),
        Some(7)
    );
    assert_eq!(
        extract_originite(&json!({"user": {"status": {"freeDiamond": 4}}})),
        Some(4)
    );
    assert_eq!(extract_originite(&json!({"user": {"status": {}}})), None);
}

fn ra_sync() -> Value {
    json!({ "user": {
        "status": { "flags": {
            "Obt/SandboxPerm/sandbox_1/sandbox_1_entry": 1,
            "Obt/SandboxPerm/sandbox_1/sandbox_1_ending_1": 1,
        } },
        "sandboxPerm": {
            "topic": "sandbox_2",
            "template": {
                "SANDBOX_V2": { "sandbox_1": null },
                "SANDBOX_V3": { "sandbox_2": { "base": { "level": 2 } } },
            },
            "summary": {
                "SANDBOX_V2": { "sandbox_1": { "day": 42, "inChallenge": false } },
                "SANDBOX_V3": { "sandbox_2": { "baseLv": 2, "inCurrent": true } },
            },
        },
    } })
}

fn sandbox_1() -> MissingTopic {
    MissingTopic {
        template: "SANDBOX_V2".into(),
        topic: "sandbox_1".into(),
    }
}

#[test]
fn an_entered_unloaded_topic_the_row_lacks_is_fetched() {
    let stored = json!({ "template": { "SANDBOX_V2": { "sandbox_1": null } } });
    assert_eq!(missing_topics(&ra_sync(), Some(&stored)), vec![sandbox_1()]);
    assert_eq!(missing_topics(&ra_sync(), None), vec![sandbox_1()]);
    let held = json!({ "template": { "SANDBOX_V2": { "sandbox_1": { "tech": {} } } } });
    assert!(missing_topics(&ra_sync(), Some(&held)).is_empty());
}

#[test]
fn a_topic_never_entered_is_never_loaded() {
    let mut s = ra_sync();
    s["user"]["status"]["flags"]
        .as_object_mut()
        .unwrap()
        .remove("Obt/SandboxPerm/sandbox_1/sandbox_1_entry");
    assert!(missing_topics(&s, None).is_empty(), "no entry flag");
    let mut s = ra_sync();
    s["user"]["sandboxPerm"]["summary"]["SANDBOX_V2"]["sandbox_1"] =
        json!({ "day": 0, "inChallenge": false });
    assert!(missing_topics(&s, None).is_empty(), "no progress");
}

#[test]
fn the_loaded_topic_and_a_pre_update_sync_are_left_alone() {
    let mut s = ra_sync();
    s["user"]["sandboxPerm"]["topic"] = json!("sandbox_1");
    assert!(missing_topics(&s, None).is_empty());
    let old = json!({ "user": { "sandboxPerm": {
        "template": { "SANDBOX_V2": { "sandbox_1": { "tech": {} } } },
    } } });
    assert!(missing_topics(&old, None).is_empty(), "no summary");
}

#[test]
fn the_blob_is_read_from_a_successful_switch_only() {
    let ok = json!({ "result": 0, "playerDataDelta": { "modified": { "sandboxPerm": {
        "template": { "SANDBOX_V2": { "sandbox_1": { "tech": { "unlock": [] } } },
                      "SANDBOX_V3": { "sandbox_2": null } },
        "topic": "sandbox_1",
    } }, "deleted": {} } });
    assert!(blob_from_change(&ok, &sandbox_1()).is_some());
    let mut refused = ok.clone();
    refused["result"] = json!(1);
    assert!(blob_from_change(&refused, &sandbox_1()).is_none());
    let mut empty = ok;
    empty["playerDataDelta"]["modified"]["sandboxPerm"]["template"]["SANDBOX_V2"]["sandbox_1"] =
        json!(null);
    assert!(blob_from_change(&empty, &sandbox_1()).is_none());
}
