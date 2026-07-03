use std::collections::HashSet;

use crate::app::services::roster::is_medal_earned;
use crate::core::gamedata::types::medal::{MedalData, MedalDefinition, Obtainability};

/// A medal row as returned from `user_medals`: `(medal_id, val, first_ts, reach_ts)`.
pub type UserMedalRow = (String, Option<serde_json::Value>, Option<i64>, Option<i64>);

const PERMANENT_POOL_WEIGHT: f64 = 0.65;
const EVENT_POOL_WEIGHT: f64 = 0.35;

const DECAY_HORIZON_SECONDS: f64 = 5.0 * 365.25 * 86400.0; // 5 years, might need tweaking as game progresses
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

/// True when a medal can't be earned by *this* user: it's gated on a collab
/// operator the user doesn't own. Such medals are dropped from the pool entirely
/// (like one-time competitive stages in the stage universe) so players aren't
/// penalized for medals they have no way to earn. If the user owns the operator,
/// the medal is achievable and scored normally.
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

/// Scoring weight of a medal rarity tier. Also used by the improvements
/// service to sort medal gaps by value.
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
