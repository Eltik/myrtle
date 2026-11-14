use std::collections::{BTreeSet, HashMap, HashSet};

use myrtle_gamedata::model::GameModel;

use myrtle_gamedata_raw::tables::gacha::GachaPoolClient;

use super::{
    estimate::{ActivityPair, backtest, build_lag_model},
    types::AlignMethod,
};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleIndependence {
    pub rule_type: String,
    pub events: usize,
    pub mismatch_rate: f64,
    pub independent: bool,
}

pub const INDEPENDENCE_MIN_EVENTS: usize = 10;
pub const INDEPENDENCE_THRESHOLD: f64 = 0.5;

pub fn rule_independence(cn: &GameModel, en: &GameModel) -> Vec<RuleIndependence> {
    let shared: HashSet<&str> = cn
        .activities
        .keys()
        .filter(|id| en.activities.contains_key(*id))
        .map(String::as_str)
        .collect();
    let cn_anchor = anchors(&cn.gacha.gacha_pool_client, cn, &shared);
    let en_anchor = anchors(&en.gacha.gacha_pool_client, en, &shared);
    let mut counts: HashMap<(String, String), (usize, usize)> = HashMap::new();
    for p in &cn.gacha.gacha_pool_client {
        if let Some(a) = cn_anchor.get(p.gacha_pool_id.as_str()) {
            counts
                .entry((p.gacha_rule_type.clone(), (*a).to_string()))
                .or_default()
                .0 += 1;
        }
    }
    for p in &en.gacha.gacha_pool_client {
        if let Some(a) = en_anchor.get(p.gacha_pool_id.as_str()) {
            counts
                .entry((p.gacha_rule_type.clone(), (*a).to_string()))
                .or_default()
                .1 += 1;
        }
    }
    let mut per_rule: HashMap<String, (usize, usize)> = HashMap::new();
    for ((rule, _), (c, e)) in counts {
        let r = per_rule.entry(rule).or_default();
        r.0 += 1;
        if c != e {
            r.1 += 1;
        }
    }
    let mut out: Vec<RuleIndependence> = per_rule
        .into_iter()
        .map(|(rule_type, (events, mismatched))| {
            let mismatch_rate = mismatched as f64 / events as f64;
            RuleIndependence {
                independent: events >= INDEPENDENCE_MIN_EVENTS
                    && mismatch_rate >= INDEPENDENCE_THRESHOLD,
                rule_type,
                events,
                mismatch_rate,
            }
        })
        .collect();
    out.sort_by(|a, b| a.rule_type.cmp(&b.rule_type));
    out
}

#[must_use]
pub fn independent_rules(cn: &GameModel, en: &GameModel) -> HashSet<String> {
    rule_independence(cn, en)
        .into_iter()
        .filter(|r| r.independent)
        .map(|r| r.rule_type)
        .collect()
}

pub const ANCHOR_WINDOW_SECS: i64 = 21 * 86_400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolPair {
    pub cn_pool_id: String,
    pub en_pool_id: String,
    pub method: AlignMethod,
}

#[must_use]
pub fn anchors<'a>(
    pools: &'a [GachaPoolClient],
    activities: &'a GameModel,
    shared_ids: &HashSet<&str>,
) -> HashMap<&'a str, &'a str> {
    let mut starts: Vec<(i64, &str)> = activities
        .activities
        .values()
        .filter(|a| shared_ids.contains(a.id.as_str()) && a.start_time > 0)
        .map(|a| (a.start_time, a.id.as_str()))
        .collect();
    starts.sort_unstable();
    let mut out = HashMap::new();
    for p in pools {
        let idx = starts.partition_point(|(s, _)| *s <= p.open_time);
        if idx == 0 {
            continue;
        }
        let (start, id) = starts[idx - 1];
        if p.open_time - start <= ANCHOR_WINDOW_SECS {
            out.insert(p.gacha_pool_id.as_str(), id);
        }
    }
    out
}

fn featured_set(p: &GachaPoolClient) -> BTreeSet<&str> {
    p.featured6.iter().map(String::as_str).collect()
}

const MIN_TOLERANCE_DAYS: f64 = 30.0;

#[must_use]
pub fn tolerance_days(pairs: &[ActivityPair], window: usize, half_life: f64) -> f64 {
    backtest(pairs, window, half_life, 0)
        .p90_abs_err_days
        .max(MIN_TOLERANCE_DAYS)
}

fn model_at(pairs: &[ActivityPair], window: usize, half_life: f64, en_time: i64) -> Option<f64> {
    let idx = pairs.partition_point(|p| p.en_start < en_time);
    if idx == 0 || window == 0 {
        return None;
    }
    Some(build_lag_model(&pairs[..idx], window, half_life).median_days)
}

pub fn align_pools(
    cn: &GameModel,
    en: &GameModel,
    pairs: &[ActivityPair],
    window: usize,
    half_life: f64,
) -> HashMap<String, PoolPair> {
    let tolerance = tolerance_days(pairs, window, half_life);
    let mut cn_pools: Vec<&GachaPoolClient> = cn.gacha.gacha_pool_client.iter().collect();
    let mut en_pools: Vec<&GachaPoolClient> = en.gacha.gacha_pool_client.iter().collect();
    cn_pools.sort_by_key(|p| (p.open_time, p.gacha_pool_id.clone()));
    en_pools.sort_by_key(|p| (p.open_time, p.gacha_pool_id.clone()));

    let mut out: HashMap<String, PoolPair> = HashMap::new();
    let mut used_en: HashSet<&str> = HashSet::new();
    let independent = independent_rules(cn, en);

    let mut en_by_rule: HashMap<&str, Vec<(&GachaPoolClient, BTreeSet<&str>)>> = HashMap::new();
    for p in &en_pools {
        let set = featured_set(p);
        if !set.is_empty() {
            en_by_rule
                .entry(p.gacha_rule_type.as_str())
                .or_default()
                .push((p, set));
        }
    }
    for p in &cn_pools {
        let want = featured_set(p);
        if want.is_empty() {
            continue;
        }
        let Some(cands) = en_by_rule.get(p.gacha_rule_type.as_str()) else {
            continue;
        };
        if let Some((e, _)) = cands.iter().find(|(e, have)| {
            if e.open_time <= p.open_time
                || used_en.contains(e.gacha_pool_id.as_str())
                || !want.is_subset(have)
            {
                return false;
            }
            let lag = (e.open_time - p.open_time) as f64 / 86_400.0;
            model_at(pairs, window, half_life, e.open_time)
                .is_none_or(|median| (lag - median).abs() <= tolerance)
        }) {
            used_en.insert(e.gacha_pool_id.as_str());
            out.insert(
                p.gacha_pool_id.clone(),
                PoolPair {
                    cn_pool_id: p.gacha_pool_id.clone(),
                    en_pool_id: e.gacha_pool_id.clone(),
                    method: AlignMethod::Content,
                },
            );
        }
    }

    let shared: HashSet<&str> = cn
        .activities
        .keys()
        .filter(|id| en.activities.contains_key(*id))
        .map(String::as_str)
        .collect();
    let cn_anchor = anchors(&cn.gacha.gacha_pool_client, cn, &shared);
    let en_anchor = anchors(&en.gacha.gacha_pool_client, en, &shared);

    let mut en_by_anchor: HashMap<(&str, &str), Vec<&GachaPoolClient>> = HashMap::new();
    for p in &en_pools {
        if used_en.contains(p.gacha_pool_id.as_str()) {
            continue;
        }
        if let Some(a) = en_anchor.get(p.gacha_pool_id.as_str()) {
            en_by_anchor
                .entry((a, p.gacha_rule_type.as_str()))
                .or_default()
                .push(p);
        }
    }
    let mut cn_by_anchor: HashMap<(&str, &str), Vec<&GachaPoolClient>> = HashMap::new();
    for p in &cn_pools {
        if out.contains_key(p.gacha_pool_id.as_str()) || independent.contains(&p.gacha_rule_type) {
            continue;
        }
        if let Some(a) = cn_anchor.get(p.gacha_pool_id.as_str()) {
            cn_by_anchor
                .entry((a, p.gacha_rule_type.as_str()))
                .or_default()
                .push(p);
        }
    }
    for (key, cns) in cn_by_anchor {
        let Some(ens) = en_by_anchor.get(&key) else {
            continue;
        };
        for (c, e) in cns.iter().zip(ens.iter()) {
            out.insert(
                c.gacha_pool_id.clone(),
                PoolPair {
                    cn_pool_id: c.gacha_pool_id.clone(),
                    en_pool_id: e.gacha_pool_id.clone(),
                    method: AlignMethod::Anchor,
                },
            );
        }
    }

    out
}
