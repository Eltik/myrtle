#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::float_cmp)]
use myrtle_engine_release::switches::parse_half_life;
use myrtle_engine_release::{estimate::*, types::Resolution};

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
