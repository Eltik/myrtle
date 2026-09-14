use std::collections::HashMap;

use crate::core::gamedata::types::activity::ActivityBasicInfo;

use crate::core::gamedata::types::GameData;

use super::types::{Backtest, LagModel, LagSample, Resolution};

const SECS_PER_DAY: f64 = 86_400.0;
pub const DEFAULT_WINDOW: usize = 10;

pub fn window_from_env() -> usize {
    match std::env::var("RELEASE_LAG_WINDOW") {
        Ok(v) => v.trim().parse().unwrap_or(DEFAULT_WINDOW),
        Err(_) => DEFAULT_WINDOW,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityPair {
    pub cn_id: String,
    pub name: String,
    pub activity_type: String,
    pub has_stage: bool,
    pub cn_start: i64,
    pub en_start: i64,
}

pub const YEARLY_MIN_DAYS: f64 = 300.0;
pub const YEARLY_MAX_DAYS: f64 = 430.0;

pub fn yearly_types(pairs: &[ActivityPair]) -> Vec<String> {
    let mut by_type: HashMap<&str, Vec<f64>> = HashMap::new();
    for p in pairs {
        by_type
            .entry(p.activity_type.as_str())
            .or_default()
            .push(lag_days(p));
    }
    let mut out: Vec<String> = by_type
        .into_iter()
        .filter(|(_, lags)| {
            lags.len() >= 2
                && lags
                    .iter()
                    .all(|l| (YEARLY_MIN_DAYS..=YEARLY_MAX_DAYS).contains(l))
        })
        .map(|(t, _)| t.to_string())
        .collect();
    out.sort();
    out
}

pub fn split_yearly(
    pairs: &[ActivityPair],
    yearly: &[String],
) -> (Vec<ActivityPair>, Vec<ActivityPair>) {
    let is_yearly = |p: &ActivityPair| yearly.iter().any(|y| y == &p.activity_type);
    let general = pairs
        .iter()
        .filter(|p| p.has_stage && !is_yearly(p))
        .cloned()
        .collect();
    let yearly = pairs.iter().filter(|p| is_yearly(p)).cloned().collect();
    (general, yearly)
}

pub fn build_yearly_model(yearly_pairs: &[ActivityPair]) -> LagModel {
    build_lag_model(yearly_pairs, yearly_pairs.len().max(1))
}

pub fn activity_pairs(cn: &GameData, en: &GameData) -> Vec<ActivityPair> {
    let mut pairs: Vec<ActivityPair> = cn
        .activities
        .values()
        .filter(|a| a.start_time > 0)
        .filter_map(|a| {
            let e = en.activities.get(&a.id)?;
            (e.start_time > 0).then(|| ActivityPair {
                cn_id: a.id.clone(),
                name: a.name.clone(),
                activity_type: a.activity_type.clone(),
                has_stage: a.has_stage,
                cn_start: a.start_time,
                en_start: e.start_time,
            })
        })
        .collect();
    pairs.sort_by(|a, b| a.en_start.cmp(&b.en_start).then(a.cn_id.cmp(&b.cn_id)));
    pairs
}

pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    match sorted.len() {
        0 => 0.0,
        1 => sorted[0],
        n => {
            let rank = p * (n - 1) as f64;
            let lo = rank.floor() as usize;
            let hi = rank.ceil() as usize;
            let frac = rank - lo as f64;
            sorted[lo] + (sorted[hi] - sorted[lo]) * frac
        }
    }
}

fn lag_days(p: &ActivityPair) -> f64 {
    (p.en_start - p.cn_start) as f64 / SECS_PER_DAY
}

pub fn build_lag_model(pairs: &[ActivityPair], window: usize) -> LagModel {
    if window == 0 || pairs.is_empty() {
        return LagModel {
            window,
            n: 0,
            median_days: 0.0,
            p25_days: 0.0,
            p75_days: 0.0,
            samples: Vec::new(),
        };
    }
    let start = pairs.len().saturating_sub(window);
    let tail = &pairs[start..];
    let mut lags: Vec<f64> = tail.iter().map(lag_days).collect();
    lags.sort_by(|a, b| a.partial_cmp(b).expect("finite lag"));
    LagModel {
        window,
        n: tail.len(),
        median_days: percentile(&lags, 0.5),
        p25_days: percentile(&lags, 0.25),
        p75_days: percentile(&lags, 0.75),
        samples: tail
            .iter()
            .map(|p| LagSample {
                cn_id: p.cn_id.clone(),
                name: p.name.clone(),
                cn_start: p.cn_start,
                en_start: p.en_start,
                lag_days: lag_days(p),
            })
            .collect(),
    }
}

pub(super) fn shift(start: i64, days: f64) -> i64 {
    start + (days * SECS_PER_DAY).round() as i64
}

pub fn estimate(model: &LagModel, cn_start: i64) -> Resolution {
    if model.n == 0 {
        return Resolution::Unmodelled;
    }
    Resolution::Estimated {
        en_start: shift(cn_start, model.median_days),
        lo: shift(cn_start, model.p25_days),
        hi: shift(cn_start, model.p75_days),
    }
}

pub fn backtest(pairs: &[ActivityPair], window: usize, since: i64) -> Backtest {
    let mut abs_errs: Vec<f64> = Vec::new();
    let mut in_band = 0usize;
    if window > 0 {
        for (i, target) in pairs.iter().enumerate() {
            if target.cn_start < since {
                continue;
            }
            let prior: Vec<ActivityPair> = pairs[..i]
                .iter()
                .filter(|p| p.en_start < target.en_start)
                .cloned()
                .collect();
            if prior.len() < 3 {
                continue;
            }
            let model = build_lag_model(&prior, window);
            if let Resolution::Estimated { en_start, lo, hi } = estimate(&model, target.cn_start) {
                abs_errs.push((en_start - target.en_start).abs() as f64 / SECS_PER_DAY);
                if target.en_start >= lo && target.en_start <= hi {
                    in_band += 1;
                }
            }
        }
    }
    abs_errs.sort_by(|a, b| a.partial_cmp(b).expect("finite error"));
    let n = abs_errs.len();
    Backtest {
        window,
        n,
        median_abs_err_days: percentile(&abs_errs, 0.5),
        p75_abs_err_days: percentile(&abs_errs, 0.75),
        p90_abs_err_days: percentile(&abs_errs, 0.9),
        max_abs_err_days: abs_errs.last().copied().unwrap_or(0.0),
        band_hit_rate: if n == 0 {
            0.0
        } else {
            in_band as f64 / n as f64
        },
    }
}

pub fn cn_day(ts: i64) -> Option<chrono::NaiveDate> {
    let tz = chrono::FixedOffset::east_opt(8 * 3600)?;
    Some(
        chrono::TimeZone::timestamp_opt(&tz, ts, 0)
            .single()?
            .date_naive(),
    )
}

pub fn stage_starts_by_day(cn: &GameData) -> HashMap<chrono::NaiveDate, Vec<&ActivityBasicInfo>> {
    let mut out: HashMap<chrono::NaiveDate, Vec<&ActivityBasicInfo>> = HashMap::new();
    for a in cn.activities.values() {
        if a.has_stage
            && a.start_time > 0
            && let Some(day) = cn_day(a.start_time)
        {
            out.entry(day).or_default().push(a);
        }
    }
    for v in out.values_mut() {
        v.sort_by(|a, b| a.start_time.cmp(&b.start_time).then(a.id.cmp(&b.id)));
    }
    out
}

pub fn stage_anchor<'a>(
    by_day: &'a HashMap<chrono::NaiveDate, Vec<&'a ActivityBasicInfo>>,
    ts: i64,
) -> Option<&'a ActivityBasicInfo> {
    let day = cn_day(ts)?;
    by_day.get(&day).and_then(|v| v.first().copied())
}

pub fn en_activity_index(en: &GameData) -> HashMap<&str, (i64, i64, &str)> {
    en.activities
        .values()
        .map(|a| (a.id.as_str(), (a.start_time, a.end_time, a.name.as_str())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(id: &str, cn_days: i64, lag_days: i64) -> ActivityPair {
        ActivityPair {
            cn_id: id.into(),
            name: id.into(),
            activity_type: "TYPE_ACT9D0".into(),
            has_stage: true,
            cn_start: cn_days * 86_400,
            en_start: (cn_days + lag_days) * 86_400,
        }
    }

    #[test]
    fn yearly_types_are_named_and_derived() {
        let mut pairs: Vec<ActivityPair> = (0..4)
            .map(|i| pair(&format!("a{i}"), i * 30, 160))
            .collect();
        for (i, t) in ["FOO", "FOO", "BAR"].iter().enumerate() {
            let mut p = pair(&format!("y{i}"), i as i64 * 400, 365);
            p.activity_type = (*t).into();
            pairs.push(p);
        }
        let y = yearly_types(&pairs);
        assert_eq!(y, vec!["FOO".to_string()]);
        let (general, yearly) = split_yearly(&pairs, &y);
        assert_eq!(general.len(), 5);
        assert_eq!(yearly.len(), 2);
        assert_eq!(build_yearly_model(&yearly).median_days, 365.0);
        assert_eq!(build_yearly_model(&[]).n, 0);
    }

    #[test]
    fn percentile_interpolates() {
        let v = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(percentile(&v, 0.5), 2.5);
        assert_eq!(percentile(&v, 0.25), 1.75);
        assert_eq!(percentile(&v, 1.0), 4.0);
        assert_eq!(percentile(&[], 0.5), 0.0);
        assert_eq!(percentile(&[7.0], 0.9), 7.0);
    }

    #[test]
    fn model_uses_only_the_trailing_window() {
        let pairs: Vec<ActivityPair> = (0..6)
            .map(|i| pair(&format!("a{i}"), i * 30, 100 + i * 10))
            .collect();
        let m = build_lag_model(&pairs, 3);
        assert_eq!(m.n, 3);
        assert_eq!(m.median_days, 140.0);
        assert_eq!(m.p25_days, 135.0);
        assert_eq!(m.p75_days, 145.0);
        assert_eq!(
            m.samples
                .iter()
                .map(|s| s.cn_id.as_str())
                .collect::<Vec<_>>(),
            ["a3", "a4", "a5"]
        );
    }

    #[test]
    fn window_zero_is_the_kill_switch() {
        let pairs = vec![pair("a", 0, 100)];
        let m = build_lag_model(&pairs, 0);
        assert_eq!(m.n, 0);
        assert_eq!(estimate(&m, 5 * 86_400), Resolution::Unmodelled);
    }

    #[test]
    fn estimate_shifts_by_median_and_bands_by_quartiles() {
        let pairs: Vec<ActivityPair> = [150, 160, 170]
            .iter()
            .enumerate()
            .map(|(i, l)| pair(&format!("a{i}"), i as i64 * 30, *l))
            .collect();
        let m = build_lag_model(&pairs, 10);
        assert_eq!(
            estimate(&m, 1_000 * 86_400),
            Resolution::Estimated {
                en_start: 1_160 * 86_400,
                lo: 1_155 * 86_400,
                hi: 1_165 * 86_400
            }
        );
    }

    #[test]
    fn backtest_only_sees_earlier_en_releases() {
        let pairs: Vec<ActivityPair> = (0..8)
            .map(|i| pair(&format!("a{i}"), i * 30, 160))
            .collect();
        let b = backtest(&pairs, 5, 0);
        assert_eq!(b.n, 5);
        assert_eq!(b.median_abs_err_days, 0.0);
        assert_eq!(b.band_hit_rate, 1.0);
        let b2 = backtest(&pairs, 5, 6 * 30 * 86_400);
        assert_eq!(b2.n, 2);
    }
}
