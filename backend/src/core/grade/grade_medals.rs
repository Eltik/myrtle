use std::collections::HashSet;

use crate::app::services::roster::is_medal_earned;
use crate::core::gamedata::types::medal::{MedalData, MedalDefinition, Obtainability};

/// A medal row as returned from `user_medals`: `(medal_id, val, first_ts, reach_ts)`.
pub type UserMedalRow = (String, Option<serde_json::Value>, Option<i64>, Option<i64>);

const PERMANENT_POOL_WEIGHT: f64 = 0.65;
const EVENT_POOL_WEIGHT: f64 = 0.35;

const DECAY_HORIZON_SECONDS: f64 = 5.0 * 365.25 * 86400.0; // 5 years; may need tweaking
const RECENY_FLOOR: f64 = 0.30;

const RARITY_T1: f64 = 1.0;
const RARITY_T2: f64 = 4.0;
const RARITY_T2D5: f64 = 10.0;
const RARITY_T3: f64 = 20.0;
const RARITY_T3D5: f64 = 40.0; // Not in data but defined in schema

const HIDDEN_MULTIPLIER: f64 = 1.5;

pub fn grade_medals(
    user_medals: &[UserMedalRow],
    medal_data: &MedalData,
    owned_operators: &HashSet<&str>,
) -> f64 {
    if medal_data.medals.is_empty() {
        return 0.0;
    }

    let earned: HashSet<&str> = user_medals
        .iter()
        .filter(|(_, val, fts, rts)| {
            let v = val.as_ref().unwrap_or(&serde_json::Value::Null);
            is_medal_earned(v, fts.unwrap_or(0), rts.unwrap_or(0))
        })
        .map(|(id, _, _, _)| id.as_str())
        .collect();

    let now = chrono::Utc::now().timestamp();
    let permanent_score = score_permanent_pool(&earned, medal_data, owned_operators, now);
    let event_score = score_event_pool(&earned, medal_data, owned_operators, now);

    (permanent_score * PERMANENT_POOL_WEIGHT) + (event_score * EVENT_POOL_WEIGHT)
}

/// Gated on a collab operator the user doesn't own. Dropped from the pool (like
/// one-time competitive stages) so nobody is penalized for an unearnable medal.
fn is_unobtainable_for(medal_id: &str, medal_data: &MedalData, owned: &HashSet<&str>) -> bool {
    medal_data
        .operator_lock(medal_id)
        .is_some_and(|lock| !owned.contains(lock.operator_id.as_str()))
}

fn score_permanent_pool(
    earned: &HashSet<&str>,
    medal_data: &MedalData,
    owned: &HashSet<&str>,
    now: i64,
) -> f64 {
    let mut earned_weight = 0.0;
    let mut total_weight = 0.0;

    for medal in medal_data.medals.values() {
        if is_unobtainable_for(&medal.medal_id, medal_data, owned) {
            continue;
        }
        if !matches!(
            medal_data.obtainability(&medal.medal_id, now),
            Obtainability::Permanent
        ) {
            continue;
        }

        let weight = medal_weight(medal);
        total_weight += weight;

        if earned.contains(medal.medal_id.as_str()) {
            earned_weight += weight;
        }
    }

    if total_weight <= 0.0 {
        return 0.0;
    }

    (earned_weight / total_weight).min(1.0)
}

fn score_event_pool(
    earned: &HashSet<&str>,
    medal_data: &MedalData,
    owned: &HashSet<&str>,
    now: i64,
) -> f64 {
    let mut earned_weighted = 0.0;
    let mut cap = 0.0;

    for medal in medal_data.medals.values() {
        if is_unobtainable_for(&medal.medal_id, medal_data, owned) {
            continue;
        }
        let Obtainability::Event { proxy_close_ts } =
            medal_data.obtainability(&medal.medal_id, now)
        else {
            continue;
        };

        let recency = recency_weight(proxy_close_ts, now);
        let weight = medal_weight(medal) * recency;

        cap += weight;

        if earned.contains(medal.medal_id.as_str()) {
            earned_weighted += weight;
        }
    }

    if cap <= 0.0 {
        return 0.0;
    }

    (earned_weighted / cap).min(1.0)
}

fn recency_weight(event_close_ts: i64, now_ts: i64) -> f64 {
    if event_close_ts <= 0 {
        return 1.0;
    }
    let age_seconds = (now_ts - event_close_ts).max(0) as f64;
    let age_ratio = age_seconds / DECAY_HORIZON_SECONDS;
    (1.0 - age_ratio).max(RECENY_FLOOR)
}

fn medal_weight(medal: &MedalDefinition) -> f64 {
    let base = rarity_weight(&medal.rarity);
    if medal.is_hidden {
        base * HIDDEN_MULTIPLIER
    } else {
        base
    }
}

/// Also used by the improvements service to sort medal gaps by value.
pub fn rarity_weight(rarity: &str) -> f64 {
    match rarity {
        "T1" => RARITY_T1,
        "T1D5" => (RARITY_T1 + RARITY_T2) / 2.0, // Not in data, interpolate
        "T2" => RARITY_T2,
        "T2D5" => RARITY_T2D5,
        "T3" => RARITY_T3,
        "T3D5" => RARITY_T3D5,
        _ => RARITY_T1, // Unknown rarity, assume cheapest
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gamedata::types::medal::OperatorLock;

    const NOW: i64 = 1_800_000_000;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    fn medal(
        id: &str,
        rarity: &str,
        hidden: bool,
        expire_times: serde_json::Value,
    ) -> MedalDefinition {
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
        // CURRENT BEHAVIOR, suspected bug: `MedalData::obtainability` returns
        // `proxy_close_ts: 0` for an unknown medal with the comment "End-ts in the
        // deep past forces full decay", but a close of 0 here means full weight.
        // Unreachable from `grade_medals` today (it only walks known medals), and an
        // open-ended active TEMP window also uses 0 to mean "no decay".
        assert!(close(recency_weight(0, NOW), 1.0));
        assert!(close(recency_weight(-5, NOW), 1.0));
    }

    #[test]
    fn no_medal_catalogue_grades_zero() {
        let owned = HashSet::new();
        assert!(close(
            grade_medals(&[earned("m1")], &MedalData::default(), &owned),
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
        let only_t3 = grade_medals(&[earned("m_t3")], &d, &owned);
        assert!(close(only_t3, 20.0 / 21.0 * PERMANENT_POOL_WEIGHT));
        let both = grade_medals(&[earned("m_t1"), earned("m_t3")], &d, &owned);
        assert!(close(both, PERMANENT_POOL_WEIGHT));
        // Unearned rows (no reach time, no condition list) count for nothing.
        let unearned = grade_medals(&[("m_t3".into(), None, Some(5), None)], &d, &owned);
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
        let g = grade_medals(&[earned("m_event")], &d, &owned);
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
            grade_medals(&rows, &d, &not_owned),
            PERMANENT_POOL_WEIGHT
        ));
        let owned: HashSet<&str> = std::iter::once("char_collab").collect();
        assert!(close(
            grade_medals(&rows, &d, &owned),
            PERMANENT_POOL_WEIGHT / 21.0
        ));
    }
}
