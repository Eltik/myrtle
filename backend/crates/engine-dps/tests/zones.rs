#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::float_cmp)] // the bit-for-bit claims are what these tests check
use myrtle_dps_formulas::operator_unit::EnemyStats;
use myrtle_engine_dps::shreds::apply_shreds;
use myrtle_engine_dps::zones::*;

fn legacy_pair(pct: i32, flat: i32) -> (f64, f64) {
    (1.0 - (f64::from(pct) / 100.0), f64::from(flat))
}

#[test]
fn legacy_shred_reproduces_apply_shreds_bit_for_bit() {
    // The full legacy range the request validator admits, on a grid.
    let none = StatReduction::default();
    let mut checked = 0;
    for def_pct in (-100..=100).step_by(5) {
        for def_flat in [-500, -1, 0, 1, 100, 450, 999, 5000] {
            for res_pct in (-100..=100).step_by(10) {
                for res_flat in [-20, 0, 7, 15, 60] {
                    let (dm, df) = legacy_pair(def_pct, def_flat);
                    let (rm, rf) = legacy_pair(res_pct, res_flat);
                    let d = ResolvedReduction::new(&none, dm, df);
                    let r = ResolvedReduction::new(&none, rm, rf);
                    for (def, res) in [(0.0, 0.0), (300.0, 20.0), (1000.0, 50.0), (2500.0, 100.0)] {
                        let enemy = EnemyStats { defense: def, res };
                        let old = apply_shreds(&enemy, &[dm, df, rm, rf]);
                        // `apply`, not `resolve_enemy`: the RES cap is a zones
                        // rule and is the one difference (see below).
                        assert_eq!(
                            old.defense.to_bits(),
                            d.apply(def).to_bits(),
                            "{def_pct} {def_flat} {def}"
                        );
                        assert_eq!(
                            old.res.to_bits(),
                            r.apply(res).to_bits(),
                            "{res_pct} {res_flat} {res}"
                        );
                        if old.res <= 100.0 {
                            let new = resolve_enemy(&enemy, &d, &r);
                            assert_eq!(old.res.to_bits(), new.res.to_bits());
                        }
                        checked += 1;
                    }
                }
            }
        }
    }
    assert!(checked > 10_000);
}

#[test]
fn zone_percent_adds_to_the_legacy_percent() {
    let zone = StatReduction {
        percent: Some(0.2),
        ..Default::default()
    };
    let (m, f) = legacy_pair(30, 0);
    let r = ResolvedReduction::new(&zone, m, f);
    assert!((r.ratio - 0.5).abs() < 1e-15);
    assert!((r.apply(1000.0) - 500.0).abs() < 1e-9);
    // Floored: 30% + 90% removes everything, never inverts.
    let zone = StatReduction {
        percent: Some(0.9),
        ..Default::default()
    };
    assert_eq!(ResolvedReduction::new(&zone, m, f).apply(1000.0), 0.0);
}

#[test]
fn order_is_flat_then_percent_then_ignore_flat_then_ignore_percent() {
    let zone = StatReduction {
        flat: Some(100.0),
        percent: Some(0.5),
        ignore_flat: Some(50.0),
        ignore_percent: Some(0.4),
        steal: vec![],
    };
    let r = ResolvedReduction::new(&zone, 1.0, 0.0);
    // ((1000 - 100) * 0.5 - 50) * 0.6 = 240
    assert!((r.apply(1000.0) - 240.0).abs() < 1e-9);
    // Each step floors at 0.
    assert_eq!(r.apply(120.0), 0.0);
}

#[test]
fn steal_takes_the_max_and_adds_to_flat() {
    let zone = StatReduction {
        flat: Some(10.0),
        steal: vec![50.0, 80.0, 30.0],
        ..Default::default()
    };
    let r = ResolvedReduction::new(&zone, 1.0, 5.0);
    assert_eq!(r.flat, 95.0);
}

#[test]
fn res_is_capped_at_100() {
    let none = ResolvedReduction::new(&StatReduction::default(), 1.0, 0.0);
    let out = resolve_enemy(
        &EnemyStats {
            defense: 0.0,
            res: 250.0,
        },
        &none,
        &none,
    );
    assert_eq!(out.res, 100.0);
}

#[test]
fn max_groups_take_the_largest_source() {
    assert_eq!(max_source(&[]), None);
    assert_eq!(max_source(&[0.1, 0.3, 0.2]), Some(0.3));
    assert_eq!(max_source(&[-0.2, -0.1]), Some(-0.1));
    assert_eq!(product(&[]), 1.0);
    assert!((product(&[1.2, 1.5]) - 1.8).abs() < 1e-15);
}

#[test]
fn fragile_groups_max_within_and_multiply_across() {
    let zones = BuffZones {
        fragile: vec![0.2, 0.3],
        physical_fragile: vec![0.1, 0.25],
        arts_fragile: vec![0.5],
        damage_reduction: vec![0.2, 0.5],
        ..Default::default()
    };
    // Legacy fragile 0.25 loses the MAX to 0.3.
    let phys = post_multiplier(&zones, Some(0.25), Some(DamageType::Physical));
    assert!((phys - 1.3 * 1.25 * 0.8 * 0.5).abs() < 1e-12);
    let arts = post_multiplier(&zones, None, Some(DamageType::Arts));
    assert!((arts - 1.3 * 1.5 * 0.8 * 0.5).abs() < 1e-12);
    // Legacy fragile wins when it is the largest generic source.
    let legacy = post_multiplier(&zones, Some(0.4), None);
    assert!((legacy - 1.4 * 0.8 * 0.5).abs() < 1e-12);
}

#[test]
fn empty_zones_leave_the_legacy_fragile_multiplier_exact() {
    let zones = BuffZones::default();
    for f in [0.0, 0.1, 0.3, -0.25, 1.7] {
        assert_eq!(
            post_multiplier(&zones, Some(f), None).to_bits(),
            (1.0 + f).to_bits()
        );
    }
    assert_eq!(post_multiplier(&zones, None, Some(DamageType::Arts)), 1.0);
}
