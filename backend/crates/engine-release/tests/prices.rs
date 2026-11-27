#![allow(clippy::unwrap_used, clippy::expect_used)]
use myrtle_engine_release::prices::*;
use myrtle_gamedata_raw::tables::character::{DisplaySkin, Skin};

fn skin(approach: &str, dynamic: bool, voice: Option<&str>, special: bool) -> Skin {
    Skin {
        skin_id: "char_1_test@brand#1".into(),
        dyn_illust_id: dynamic.then(|| "dyn".into()),
        voice_id: voice.map(str::to_string),
        sp_dyn_illust_id: special.then(|| "sp".into()),
        display_skin: DisplaySkin {
            obtain_approach: Some(approach.into()),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn price(approach: &str, dynamic: bool, voice: Option<&str>, special: bool) -> i32 {
    skin_price(&skin(approach, dynamic, voice, special), approach.into()).price
}

#[test]
fn store_outfits_add_three_per_feature_and_the_rest_cost_nothing() {
    assert_eq!(price("采购中心", false, None, false), 18);
    assert_eq!(price("Store, Redeem Code", false, None, false), 18);
    assert_eq!(price("采购中心", true, None, false), 21);
    assert_eq!(
        price("采购中心", true, Some("char_1_test_brand#1"), false),
        24
    );
    assert_eq!(price("采购中心", true, None, true), 24);
    assert_eq!(
        price("采购中心", true, Some("char_1_test_brand#1"), true),
        27
    );
    assert_eq!(
        price("采购中心", false, Some("char_1_test#1"), false),
        18,
        "a borrowed voice set is not a feature"
    );
    let early = Skin {
        skin_id: "char_103_angel@wild#1".into(),
        ..skin("Store", false, None, false)
    };
    assert_eq!(store_price(&early), 15, "the 15 tier is a closed list");
    assert!(plain_store(&early));
    assert!(!plain_store(&skin("采购中心", true, None, false)));
    assert!(!plain_store(&skin("活动获得", false, None, false)));
    let event = skin_price(&skin("活动获得", true, None, false), "Event Reward".into());
    assert_eq!(
        (event.price, event.store, event.obtain.as_str()),
        (0, false, "Event Reward")
    );
}

#[test]
fn the_early_list_is_58_distinct_ids() {
    let mut ids: Vec<&str> = EARLY.to_vec();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 58);
    assert_eq!(EARLY.len(), 58);
}
