#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::float_cmp)]
use std::collections::HashSet;

use myrtle_engine_release::{align::*, estimate::ActivityPair, types::AlignMethod};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::activity::ActivityBasicInfo;
use myrtle_gamedata_raw::tables::gacha::GachaPoolClient;

fn pool(id: &str, rule: &str, open: i64, f6: &[&str]) -> GachaPoolClient {
    GachaPoolClient {
        gacha_pool_id: id.into(),
        gacha_rule_type: rule.into(),
        open_time: open,
        end_time: open + 14 * 86_400,
        featured6: f6.iter().map(|s| (*s).to_string()).collect(),
        ..Default::default()
    }
}

fn act(id: &str, start: i64) -> ActivityBasicInfo {
    ActivityBasicInfo {
        id: id.into(),
        start_time: start,
        end_time: start + 21 * 86_400,
        has_stage: true,
        ..Default::default()
    }
}

fn game(pools: Vec<GachaPoolClient>, acts: Vec<ActivityBasicInfo>) -> GameModel {
    let mut g = GameModel::default();
    g.gacha.gacha_pool_client = pools;
    g.activities = acts.into_iter().map(|a| (a.id.clone(), a)).collect();
    g
}

const D: i64 = 86_400;

#[test]
fn content_match_is_containment_and_takes_the_earliest_later_en_pool_once() {
    let cn = game(
        vec![
            pool("LIMITED_1", "LIMITED", 100 * D, &["char_a"]),
            pool("LIMITED_2", "LIMITED", 500 * D, &["char_a"]),
        ],
        vec![],
    );
    let en = game(
        vec![
            pool(
                "LIMITED_EN_1",
                "LIMITED",
                260 * D,
                &["char_a", "char_cofeat"],
            ),
            pool("LIMITED_EN_2", "LIMITED", 660 * D, &["char_a"]),
        ],
        vec![],
    );
    let m = align_pools(&cn, &en, &[], 10, 0.0);
    assert_eq!(m["LIMITED_1"].en_pool_id, "LIMITED_EN_1");
    assert_eq!(m["LIMITED_2"].en_pool_id, "LIMITED_EN_2");
    assert!(m.values().all(|p| p.method == AlignMethod::Content));
}

#[test]
fn anchor_pairs_by_open_order_and_is_labelled() {
    let cn = game(
        vec![
            pool("SINGLE_1", "SINGLE", 100 * D, &[]),
            pool("SINGLE_2", "SINGLE", 106 * D, &[]),
            pool("SINGLE_9", "SINGLE", 150 * D, &[]), // no shared anchor
        ],
        vec![act("act1side", 100 * D)],
    );
    let en = game(
        vec![
            pool("SINGLE_EN_1", "SINGLE", 261 * D, &[]),
            pool("SINGLE_EN_2", "SINGLE", 270 * D, &[]),
        ],
        vec![act("act1side", 260 * D)],
    );
    let m = align_pools(&cn, &en, &[], 10, 0.0);
    assert_eq!(m["SINGLE_1"].en_pool_id, "SINGLE_EN_1");
    assert_eq!(m["SINGLE_1"].method, AlignMethod::Anchor);
    assert_eq!(m["SINGLE_2"].en_pool_id, "SINGLE_EN_2");
    assert!(!m.contains_key("SINGLE_9"));
}

#[test]
fn content_candidates_outside_the_lag_envelope_are_rejected() {
    let history: Vec<ActivityPair> = (0..6)
        .map(|i| ActivityPair {
            cn_id: format!("a{i}"),
            name: String::new(),
            activity_type: "TYPE_ACT9D0".into(),
            has_stage: true,
            cn_start: i * 30 * D,
            en_start: (i * 30 + 160) * D,
        })
        .collect();
    assert_eq!(tolerance_days(&history, 10, 0.0), 30.0);
    let cn = game(
        vec![pool("CLASSIC_1", "CLASSIC", 200 * D, &["char_a", "char_b"])],
        vec![],
    );
    let en = game(
        vec![
            pool("CLASSIC_EN_1", "CLASSIC", 210 * D, &["char_a", "char_b"]), // lag 10: EN's own run
            pool("CLASSIC_EN_2", "CLASSIC", 365 * D, &["char_a", "char_b"]), // lag 165: the real one
        ],
        vec![],
    );
    let m = align_pools(&cn, &en, &history, 10, 0.0);
    assert_eq!(m["CLASSIC_1"].en_pool_id, "CLASSIC_EN_2");
    assert_eq!(m["CLASSIC_1"].method, AlignMethod::Content);
}

#[test]
fn independence_is_derived_from_count_mismatch() {
    let mut cn_pools = vec![];
    let mut en_pools = vec![];
    let mut cn_acts = vec![];
    let mut en_acts = vec![];
    for i in 0..12 {
        let id = format!("act{i}side");
        cn_acts.push(act(&id, (100 + i * 40) * D));
        en_acts.push(act(&id, (260 + i * 40) * D));
        cn_pools.push(pool(
            &format!("DOUBLE_{i}"),
            "DOUBLE",
            (100 + i * 40) * D,
            &[],
        ));
        cn_pools.push(pool(
            &format!("SINGLE_{i}"),
            "SINGLE",
            (100 + i * 40) * D,
            &[],
        ));
        en_pools.push(pool(
            &format!("DOUBLE_EN_{i}"),
            "DOUBLE",
            (261 + i * 40) * D,
            &[],
        ));
        en_pools.push(pool(
            &format!("SINGLE_EN_{i}"),
            "SINGLE",
            (261 + i * 40) * D,
            &[],
        ));
        if i < 8 {
            en_pools.push(pool(
                &format!("DOUBLE_EN_{i}b"),
                "DOUBLE",
                (268 + i * 40) * D,
                &[],
            ));
        }
    }
    let cn = game(cn_pools, cn_acts);
    let en = game(en_pools, en_acts);
    let ind = independent_rules(&cn, &en);
    assert!(ind.contains("DOUBLE") && !ind.contains("SINGLE"));
    let m = align_pools(&cn, &en, &[], 10, 0.0);
    assert!(m.keys().all(|k| k.starts_with("SINGLE_")));
    assert_eq!(m.len(), 12);
}

#[test]
fn anchor_window_is_bounded() {
    let cn = game(
        vec![pool("SINGLE_1", "SINGLE", 130 * D, &[])],
        vec![act("act1side", 100 * D)],
    );
    let shared: HashSet<&str> = std::iter::once("act1side").collect();
    assert!(anchors(&cn.gacha.gacha_pool_client, &cn, &shared).is_empty());
}
