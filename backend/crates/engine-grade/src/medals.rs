use std::collections::HashSet;

use myrtle_gamedata_raw::tables::medal::{MedalData, MedalDefinition, Obtainability};

pub type UserMedalRow = (String, Option<serde_json::Value>, Option<i64>, Option<i64>);

const PERMANENT_POOL_WEIGHT: f64 = 0.65;
const EVENT_POOL_WEIGHT: f64 = 0.35;

const DECAY_HORIZON_SECONDS: f64 = 5.0 * 365.25 * 86400.0; // 5 years
const RECENY_FLOOR: f64 = 0.30;

const RARITY_T1: f64 = 1.0;
const RARITY_T2: f64 = 4.0;
const RARITY_T2D5: f64 = 10.0;
const RARITY_T3: f64 = 20.0;
const RARITY_T3D5: f64 = 40.0;

const HIDDEN_MULTIPLIER: f64 = 1.5;

#[must_use]
pub fn grade_medals(
    user_medals: &[UserMedalRow],
    medal_data: &MedalData,
    owned_operators: &HashSet<&str>,
    now: i64,
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

    let permanent_score = score_permanent_pool(&earned, medal_data, owned_operators, now);
    let event_score = score_event_pool(&earned, medal_data, owned_operators, now);

    (permanent_score * PERMANENT_POOL_WEIGHT) + (event_score * EVENT_POOL_WEIGHT)
}

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

#[must_use]
pub fn rarity_weight(rarity: &str) -> f64 {
    match rarity {
        "T1" => RARITY_T1,
        "T1D5" => f64::midpoint(RARITY_T1, RARITY_T2),
        "T2" => RARITY_T2,
        "T2D5" => RARITY_T2D5,
        "T3" => RARITY_T3,
        "T3D5" => RARITY_T3D5,
        _ => RARITY_T1,
    }
}

pub fn is_medal_earned(val: &serde_json::Value, fts: i64, rts: i64) -> bool {
    if rts > 0 {
        return true;
    }
    if rts == -1
        && fts > 0
        && let Some(arr) = val.as_array()
    {
        if arr.is_empty() {
            return true;
        }
        return arr.iter().all(is_condition_met);
    }
    false
}

fn is_condition_met(cond: &serde_json::Value) -> bool {
    cond.as_array()
        .and_then(|pair| {
            let a = pair.first()?.as_i64()?;
            let r = pair.get(1)?.as_i64()?;
            Some(a >= r)
        })
        .unwrap_or(false)
}
