use std::collections::{BTreeSet, HashMap, HashSet};

use crate::core::gamedata::types::{GameData, gacha::GachaPoolClient};

use super::{
    estimate::{ActivityPair, backtest, build_lag_model},
    types::AlignMethod,
};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RuleIndependence {
    pub rule_type: String,
    pub events: usize,
    pub mismatch_rate: f64,
    pub independent: bool,
}

pub const INDEPENDENCE_MIN_EVENTS: usize = 10;
pub const INDEPENDENCE_THRESHOLD: f64 = 0.5;

pub fn rule_independence(cn: &GameData, en: &GameData) -> Vec<RuleIndependence> {
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

pub fn independent_rules(cn: &GameData, en: &GameData) -> HashSet<String> {
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

pub fn anchors<'a>(
    pools: &'a [GachaPoolClient],
    activities: &'a GameData,
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

pub fn tolerance_days(pairs: &[ActivityPair], window: usize) -> f64 {
    backtest(pairs, window, 0)
        .p90_abs_err_days
        .max(MIN_TOLERANCE_DAYS)
}

fn model_at(pairs: &[ActivityPair], window: usize, en_time: i64) -> Option<f64> {
    let idx = pairs.partition_point(|p| p.en_start < en_time);
    if idx == 0 || window == 0 {
        return None;
    }
    Some(build_lag_model(&pairs[..idx], window).median_days)
}

pub fn align_pools(
    cn: &GameData,
    en: &GameData,
    pairs: &[ActivityPair],
    window: usize,
) -> HashMap<String, PoolPair> {
    let tolerance = tolerance_days(pairs, window);
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
            model_at(pairs, window, e.open_time)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gamedata::types::activity::ActivityBasicInfo;

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

    fn game(pools: Vec<GachaPoolClient>, acts: Vec<ActivityBasicInfo>) -> GameData {
        let mut g = GameData::new();
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
        let m = align_pools(&cn, &en, &[], 10);
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
        let m = align_pools(&cn, &en, &[], 10);
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
        assert_eq!(tolerance_days(&history, 10), 30.0);
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
        let m = align_pools(&cn, &en, &history, 10);
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
        let m = align_pools(&cn, &en, &[], 10);
        assert!(m.keys().all(|k| k.starts_with("SINGLE_")));
        assert_eq!(m.len(), 12);
    }

    #[test]
    fn anchor_window_is_bounded() {
        let cn = game(
            vec![pool("SINGLE_1", "SINGLE", 130 * D, &[])],
            vec![act("act1side", 100 * D)],
        );
        let shared: HashSet<&str> = ["act1side"].into_iter().collect();
        assert!(anchors(&cn.gacha.gacha_pool_client, &cn, &shared).is_empty());
    }
}
