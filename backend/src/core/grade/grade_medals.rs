use std::collections::HashSet;

use crate::core::gamedata::types::medal::{MedalData, MedalDefinition};

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
    user_medals: &[(String, Option<i64>, Option<i64>)], // (medal_id, first_ts, reach_ts)
    medal_data: &MedalData,
) -> f64 {
    if medal_data.medals.is_empty() {
        return 0.0;
    }

    let earned: HashSet<&str> = user_medals.iter().map(|(id, _, _)| id.as_str()).collect();

    let permanent_score = score_permanent_pool(&earned, medal_data);
    let event_score = score_event_pool(&earned, medal_data);

    (permanent_score * PERMANENT_POOL_WEIGHT) + (event_score * EVENT_POOL_WEIGHT)
}

fn score_permanent_pool(earned: &HashSet<&str>, medal_data: &MedalData) -> f64 {
    let mut earned_weight = 0.0;
    let mut total_weight = 0.0;

    for medal in medal_data.medals.values() {
        if !is_permanent(medal) {
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

fn is_permanent(medal: &MedalDefinition) -> bool {
    if medal.expire_times.is_empty() {
        return true;
    }

    let latest = medal.expire_times.iter().max_by_key(|e| e.start);

    if let Some(entry) = latest {
        return entry.expire_type == "PERM" && entry.end == -1;
    }

    false
}

fn score_event_pool(earned: &HashSet<&str>, medal_data: &MedalData) -> f64 {
    let now = chrono::Utc::now().timestamp();

    let mut earned_weighted = 0.0;
    let mut cap = 0.0;

    for medal in medal_data.medals.values() {
        let Some(end_ts) = event_end_timestamp(medal) else {
            continue; // not an event medal
        };

        let recency = recency_weight(end_ts, now);
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

fn event_end_timestamp(medal: &MedalDefinition) -> Option<i64> {
    if medal.expire_times.is_empty() {
        return None; // permanent, not event
    }

    let latest = medal.expire_times.iter().max_by_key(|e| e.start)?;

    if latest.expire_type == "PERM" && latest.end == -1 {
        return None; // permanent medals handled elsewhere
    }

    if latest.end > 0 {
        Some(latest.end)
    } else {
        Some(latest.start.max(0)) // ongoing event
    }
}

fn recency_weight(event_end_ts: i64, now_ts: i64) -> f64 {
    let age_seconds = (now_ts - event_end_ts).max(0) as f64;
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

fn rarity_weight(rarity: &str) -> f64 {
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
