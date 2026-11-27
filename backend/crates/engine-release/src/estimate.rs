use std::collections::HashMap;

use myrtle_gamedata_raw::tables::activity::ActivityBasicInfo;

use myrtle_gamedata::model::GameModel;

use super::types::{Backtest, LagModel, LagSample, Resolution};

const SECS_PER_DAY: f64 = 86_400.0;
pub use crate::switches::{DEFAULT_HALF_LIFE, DEFAULT_WINDOW};

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

#[must_use]
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

#[must_use]
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

#[must_use]
pub fn build_yearly_model(yearly_pairs: &[ActivityPair]) -> LagModel {
    build_lag_model(yearly_pairs, yearly_pairs.len().max(1), 0.0)
}

#[must_use]
pub fn activity_pairs(cn: &GameModel, en: &GameModel) -> Vec<ActivityPair> {
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

#[must_use]
pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    match sorted.len() {
        0 => 0.0,
        1 => sorted[0],
        n => {
            let rank = p * (n - 1) as f64;
            let lo = rank.floor() as usize;
            let hi = rank.ceil() as usize;
            let frac = rank - lo as f64;
            (sorted[hi] - sorted[lo]).mul_add(frac, sorted[lo])
        }
    }
}

#[must_use]
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
            let x = p.mul_add(mid[n - 1] - mid[0], mid[0]);
            let hi = mid.partition_point(|&m| m <= x).clamp(1, n - 1);
            let lo = hi - 1;
            let span = mid[hi] - mid[lo];
            let frac = if span > 0.0 {
                (x - mid[lo]) / span
            } else {
                0.0
            };
            (sorted[hi].0 - sorted[lo].0).mul_add(frac, sorted[lo].0)
        }
    }
}

fn recency_weights(n: usize, half_life: f64) -> Vec<f64> {
    (0..n)
        .map(|i| 0.5_f64.powf((n - 1 - i) as f64 / half_life))
        .collect()
}

fn lag_days(p: &ActivityPair) -> f64 {
    (p.en_start - p.cn_start) as f64 / SECS_PER_DAY
}

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

#[must_use]
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

#[must_use]
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

#[must_use]
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
    #[must_use]
    pub fn build(cn: &'a GameModel) -> Self {
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

    #[must_use]
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

#[must_use]
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

#[must_use]
pub fn en_activity_index(en: &GameModel) -> HashMap<&str, (i64, i64, &str)> {
    en.activities
        .values()
        .map(|a| (a.id.as_str(), (a.start_time, a.end_time, a.name.as_str())))
        .collect()
}
