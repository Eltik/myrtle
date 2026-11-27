#![allow(clippy::unwrap_used, clippy::expect_used)]
use std::collections::HashSet;

use myrtle_engine_grade::medals::*;
use myrtle_gamedata_raw::tables::medal::{MedalData, MedalDefinition, Obtainability, OperatorLock};

const NOW: i64 = 1_800_000_000;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-12
}

fn medal(id: &str, rarity: &str, hidden: bool, expire_times: serde_json::Value) -> MedalDefinition {
    serde_json::from_value(serde_json::json!({
        "MedalId": id,
        "MedalName": id,
        "MedalType": "playerMedal",
        "SlotId": 0,
        "Rarity": rarity,
        "IsHidden": hidden,
        "ExpireTimes": expire_times,
    }))
    .expect("medal")
}

fn data(medals: Vec<MedalDefinition>) -> MedalData {
    MedalData {
        medals: medals
            .into_iter()
            .map(|m| (m.medal_id.clone(), m))
            .collect(),
        ..MedalData::default()
    }
}

fn earned(id: &str) -> UserMedalRow {
    (id.to_string(), None, None, Some(1))
}

#[test]
fn rarity_weights_follow_the_tier_table() {
    let cases = [
        ("T1", 1.0),
        ("T1D5", 2.5),
        ("T2", 4.0),
        ("T2D5", 10.0),
        ("T3", 20.0),
        ("T3D5", 40.0),
        ("T9", 1.0),
        ("", 1.0),
    ];
    for (rarity, weight) in cases {
        assert!(close(rarity_weight(rarity), weight), "{rarity}");
    }
}

#[test]
fn hidden_medals_weigh_half_again() {
    let plain = medal("m_plain", "T3", false, serde_json::json!([]));
    let hidden = medal("m_hidden", "T3", true, serde_json::json!([]));
    assert!(close(medal_weight(&plain), 20.0));
    assert!(close(medal_weight(&hidden), 30.0));
}

#[test]
fn recency_decays_linearly_to_a_floor() {
    let horizon = DECAY_HORIZON_SECONDS as i64;
    assert!(close(recency_weight(NOW, NOW), 1.0));
    // A close in the future has not aged.
    assert!(close(recency_weight(NOW + 86_400, NOW), 1.0));
    assert!(close(recency_weight(NOW - horizon / 2, NOW), 0.5));
    assert!(close(recency_weight(NOW - horizon, NOW), RECENY_FLOOR));
    assert!(close(recency_weight(NOW - 10 * horizon, NOW), RECENY_FLOOR));
}

#[test]
fn a_missing_close_time_is_not_decayed() {
    // 0 means an ongoing event with no scheduled close (`Obtainability::Event`
    // docs), which an open-ended active TEMP window relies on.
    assert!(close(recency_weight(0, NOW), 1.0));
    assert!(close(recency_weight(-5, NOW), 1.0));
}

#[test]
fn an_unknown_medal_is_a_closed_event_at_the_recency_floor() {
    // Unreachable from `grade_medals` today (it only walks known medals); this
    // pins what `obtainability` promises for an id the catalogue lacks.
    let unknown = MedalData::default().obtainability("medal_not_in_table", NOW);
    assert_eq!(unknown, Obtainability::Event { proxy_close_ts: 1 });
    let Obtainability::Event { proxy_close_ts } = unknown else {
        unreachable!()
    };
    assert!(close(recency_weight(proxy_close_ts, NOW), RECENY_FLOOR));
}

#[test]
fn no_medal_catalogue_grades_zero() {
    let owned = HashSet::new();
    assert!(close(
        grade_medals(&[earned("m1")], &MedalData::default(), &owned, NOW),
        0.0
    ));
}

#[test]
fn permanent_pool_is_rarity_weighted() {
    let d = data(vec![
        medal("m_t1", "T1", false, serde_json::json!([])),
        medal("m_t3", "T3", false, serde_json::json!([])),
    ]);
    let owned = HashSet::new();
    let only_t3 = grade_medals(&[earned("m_t3")], &d, &owned, NOW);
    assert!(close(only_t3, 20.0 / 21.0 * PERMANENT_POOL_WEIGHT));
    let both = grade_medals(&[earned("m_t1"), earned("m_t3")], &d, &owned, NOW);
    assert!(close(both, PERMANENT_POOL_WEIGHT));
    // Unearned rows (no reach time, no condition list) count for nothing.
    let unearned = grade_medals(&[("m_t3".into(), None, Some(5), None)], &d, &owned, NOW);
    assert!(close(unearned, 0.0));
}

#[test]
fn active_event_medals_fill_the_event_pool() {
    let d = data(vec![
        medal("m_perm", "T1", false, serde_json::json!([])),
        medal(
            "m_event",
            "T2",
            false,
            serde_json::json!([{ "Start": 0, "End": -1, "Type_": "TEMP" }]),
        ),
    ]);
    let owned = HashSet::new();
    let g = grade_medals(&[earned("m_event")], &d, &owned, NOW);
    assert!(close(g, EVENT_POOL_WEIGHT));
}

#[test]
fn collab_locked_medals_drop_out_unless_the_operator_is_owned() {
    let mut d = data(vec![
        medal("m_free", "T1", false, serde_json::json!([])),
        medal("m_collab", "T3", false, serde_json::json!([])),
    ]);
    d.operator_locked.insert(
        "m_collab".into(),
        OperatorLock {
            operator_id: "char_collab".into(),
            operator_name: "Collab".into(),
        },
    );
    let rows = [earned("m_free")];
    let not_owned = HashSet::new();
    assert!(close(
        grade_medals(&rows, &d, &not_owned, NOW),
        PERMANENT_POOL_WEIGHT
    ));
    let owned: HashSet<&str> = std::iter::once("char_collab").collect();
    assert!(close(
        grade_medals(&rows, &d, &owned, NOW),
        PERMANENT_POOL_WEIGHT / 21.0
    ));
}
