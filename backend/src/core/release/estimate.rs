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

/// Recency half-life of the lag window, in EN releases: the newest pair weighs
/// 1, the one `h` releases older 0.5. Measured 2026-10-24 by walk-forward over
/// the 66 CN-2024+ stage events (the three wiki-announced November pairs
/// included): MAE 7.84 -> 7.22 d at release, 9.58 -> 9.14 d averaged over
/// 0..165 d forecast horizons. Both measures bottom out at 3 on an integer
/// sweep of 1..5 (7.33 / 7.26 / 7.22 / 7.26 / 7.36 and 9.44 / 9.22 / 9.14 /
/// 9.24 / 9.32); `docs/release/CN_EN_RELEASE_MAPPING.md` section 5k has the
/// whole sweep and what it ruled out.
pub const DEFAULT_HALF_LIFE: f64 = 3.0;

/// `RELEASE_LAG_HALF_LIFE`. A MISSING variable is the default. `0`, or any
/// value at or below zero, is the kill switch: every pair weighs the same and
/// the model is the unweighted trailing median, bit for bit. A value that does
/// not parse to a finite number falls back to the default, as
/// `RELEASE_LAG_WINDOW` does.
pub fn half_life_from_env() -> f64 {
    match std::env::var("RELEASE_LAG_HALF_LIFE") {
        Ok(v) => parse_half_life(&v),
        Err(_) => DEFAULT_HALF_LIFE,
    }
}

fn parse_half_life(v: &str) -> f64 {
    match v.trim().parse::<f64>() {
        Ok(h) if h.is_finite() => {
            if h > 0.0 {
                h
            } else {
                0.0
            }
        }
        _ => DEFAULT_HALF_LIFE,
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

/// Unweighted on purpose: the yearly pairs (four April Fools on the live
/// extract, sd 0.4 d) were never part of the recency sweep.
pub fn build_yearly_model(yearly_pairs: &[ActivityPair]) -> LagModel {
    build_lag_model(yearly_pairs, yearly_pairs.len().max(1), 0.0)
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

/// `percentile` over `(value, weight)` samples sorted by value. Each sample
/// sits at the midpoint of its weight mass; `p` runs from the first midpoint to
/// the last and interpolates linearly between neighbours, so equal weights give
/// the same quartiles as `percentile` (to rounding; the kill switch does not
/// rely on that and takes `percentile` itself).
pub fn weighted_percentile(sorted: &[(f64, f64)], p: f64) -> f64 {
    match sorted.len() {
        0 => 0.0,
        1 => sorted[0].0,
        n => {
            let mut mid = Vec::with_capacity(n);
            let mut acc = 0.0;
            for &(_, w) in sorted {
                mid.push(acc + w / 2.0);
                acc += w;
            }
            let x = mid[0] + p * (mid[n - 1] - mid[0]);
            let hi = mid.partition_point(|&m| m <= x).clamp(1, n - 1);
            let lo = hi - 1;
            let span = mid[hi] - mid[lo];
            let frac = if span > 0.0 {
                (x - mid[lo]) / span
            } else {
                0.0
            };
            sorted[lo].0 + (sorted[hi].0 - sorted[lo].0) * frac
        }
    }
}

/// Weight of each of `n` pairs in EN order, the newest last at 1.
fn recency_weights(n: usize, half_life: f64) -> Vec<f64> {
    (0..n)
        .map(|i| 0.5_f64.powf((n - 1 - i) as f64 / half_life))
        .collect()
}

fn lag_days(p: &ActivityPair) -> f64 {
    (p.en_start - p.cn_start) as f64 / SECS_PER_DAY
}

/// The last `window` pairs by EN start; median / p25 / p75 of their lags,
/// weighted by recency when `half_life > 0` and unweighted at `0`.
pub fn build_lag_model(pairs: &[ActivityPair], window: usize, half_life: f64) -> LagModel {
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
    let (median_days, p25_days, p75_days) = if half_life > 0.0 {
        let mut lags: Vec<(f64, f64)> = tail
            .iter()
            .map(lag_days)
            .zip(recency_weights(tail.len(), half_life))
            .collect();
        lags.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("finite lag"));
        (
            weighted_percentile(&lags, 0.5),
            weighted_percentile(&lags, 0.25),
            weighted_percentile(&lags, 0.75),
        )
    } else {
        let mut lags: Vec<f64> = tail.iter().map(lag_days).collect();
        lags.sort_by(|a, b| a.partial_cmp(b).expect("finite lag"));
        (
            percentile(&lags, 0.5),
            percentile(&lags, 0.25),
            percentile(&lags, 0.75),
        )
    };
    LagModel {
        window,
        n: tail.len(),
        median_days,
        p25_days,
        p75_days,
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

pub fn backtest(pairs: &[ActivityPair], window: usize, half_life: f64, since: i64) -> Backtest {
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
            let model = build_lag_model(&prior, window, half_life);
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

pub struct StageRuns<'a> {
    by_day: HashMap<chrono::NaiveDate, Vec<&'a ActivityBasicInfo>>,
    runs: Vec<&'a ActivityBasicInfo>,
}

pub struct Anchored<'a> {
    pub activity: &'a ActivityBasicInfo,
    pub offset_secs: i64,
}

impl<'a> StageRuns<'a> {
    pub fn build(cn: &'a GameData) -> Self {
        let mut by_day: HashMap<chrono::NaiveDate, Vec<&ActivityBasicInfo>> = HashMap::new();
        let mut runs: Vec<&ActivityBasicInfo> = Vec::new();
        for a in cn.activities.values() {
            if !a.has_stage || a.start_time <= 0 {
                continue;
            }
            if let Some(day) = cn_day(a.start_time) {
                by_day.entry(day).or_default().push(a);
            }
            runs.push(a);
        }
        for v in by_day.values_mut() {
            v.sort_by(|a, b| a.start_time.cmp(&b.start_time).then(a.id.cmp(&b.id)));
        }
        runs.sort_by(|a, b| a.start_time.cmp(&b.start_time).then(a.id.cmp(&b.id)));
        Self { by_day, runs }
    }

    pub fn anchor(&self, ts: i64) -> Option<Anchored<'a>> {
        if let Some(a) = cn_day(ts)
            .and_then(|day| self.by_day.get(&day))
            .and_then(|v| v.first().copied())
        {
            return Some(Anchored {
                activity: a,
                offset_secs: 0,
            });
        }
        self.runs
            .iter()
            .rev()
            .find(|a| {
                a.start_time < ts
                    && ts <= a.end_time
                    && ts - a.start_time <= super::align::ANCHOR_WINDOW_SECS
            })
            .map(|a| Anchored {
                activity: a,
                offset_secs: ts - a.start_time,
            })
    }
}

pub fn shift_resolution(r: Resolution, secs: i64) -> Resolution {
    match r {
        Resolution::Confirmed {
            en_id,
            en_start,
            en_end,
        } => Resolution::Confirmed {
            en_id,
            en_start: en_start + secs,
            en_end: en_end + secs,
        },
        Resolution::Override {
            en_id,
            en_start,
            en_end,
            source,
            note,
        } => Resolution::Override {
            en_id,
            en_start: en_start + secs,
            en_end: en_end.map(|e| e + secs),
            source,
            note,
        },
        Resolution::Estimated { en_start, lo, hi } => Resolution::Estimated {
            en_start: en_start + secs,
            lo: lo + secs,
            hi: hi + secs,
        },
        other => other,
    }
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
        let m = build_lag_model(&pairs, 3, 0.0);
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
        let m = build_lag_model(&pairs, 0, DEFAULT_HALF_LIFE);
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
        let m = build_lag_model(&pairs, 10, 0.0);
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
        let b = backtest(&pairs, 5, 0.0, 0);
        assert_eq!(b.n, 5);
        assert_eq!(b.median_abs_err_days, 0.0);
        assert_eq!(b.band_hit_rate, 1.0);
        let b2 = backtest(&pairs, 5, 0.0, 6 * 30 * 86_400);
        assert_eq!(b2.n, 2);
    }

    #[test]
    fn half_life_parses_missing_and_bad_values_to_the_default_and_zero_to_off() {
        assert_eq!(parse_half_life("2.5"), 2.5);
        assert_eq!(parse_half_life(" 3 "), 3.0);
        assert_eq!(parse_half_life("0"), 0.0);
        assert_eq!(parse_half_life("-1"), 0.0);
        assert_eq!(parse_half_life("-0"), 0.0);
        assert_eq!(parse_half_life(""), DEFAULT_HALF_LIFE);
        assert_eq!(parse_half_life("off"), DEFAULT_HALF_LIFE);
        assert_eq!(parse_half_life("inf"), DEFAULT_HALF_LIFE);
        assert_eq!(parse_half_life("NaN"), DEFAULT_HALF_LIFE);
    }

    #[test]
    fn equal_weights_reproduce_percentile() {
        let mut v: Vec<f64> = vec![160.0, 151.06, 166.54, 153.12, 162.67, 159.29, 156.54];
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for n in 1..=v.len() {
            let s = &v[..n];
            let w: Vec<(f64, f64)> = s.iter().map(|&x| (x, 1.0)).collect();
            for p in [0.0, 0.25, 0.5, 0.75, 1.0] {
                assert!((weighted_percentile(&w, p) - percentile(s, p)).abs() < 1e-9);
            }
        }
        assert_eq!(weighted_percentile(&[], 0.5), 0.0);
    }

    #[test]
    fn half_life_zero_is_the_unweighted_trailing_median() {
        let pairs: Vec<ActivityPair> = (0..6)
            .map(|i| pair(&format!("a{i}"), i * 30, 100 + i * 10))
            .collect();
        let off = build_lag_model(&pairs, 6, 0.0);
        assert_eq!(off.median_days, 125.0);
        assert_eq!(off.p25_days, 112.5);
        assert_eq!(off.p75_days, 137.5);
        let on = build_lag_model(&pairs, 6, 1.0);
        assert!(on.median_days > off.median_days, "recent lags weigh more");
        assert!(on.p25_days > off.p25_days && on.p75_days > off.p75_days);
        assert_eq!(on.samples, off.samples, "the window itself does not move");
        let flat = build_lag_model(&pairs, 6, 1e12);
        assert!((flat.median_days - off.median_days).abs() < 1e-6);
    }

    #[test]
    fn backtest_takes_the_half_life() {
        let lags = [150, 150, 150, 170, 170, 170, 170, 170];
        let pairs: Vec<ActivityPair> = lags
            .iter()
            .enumerate()
            .map(|(i, l)| pair(&format!("a{i}"), i as i64 * 30, *l))
            .collect();
        let off = backtest(&pairs, 8, 0.0, 0);
        let on = backtest(&pairs, 8, 1.0, 0);
        assert_eq!(off.n, on.n);
        assert!(on.median_abs_err_days < off.median_abs_err_days);
    }
}
