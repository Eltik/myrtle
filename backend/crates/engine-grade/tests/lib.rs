#![allow(clippy::unwrap_used, clippy::expect_used)]
use myrtle_engine_grade::weighted_average;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-12
}

#[test]
fn averages_by_weight() {
    assert!(close(weighted_average(&[(30.0, 1.0), (10.0, 0.0)]), 0.75));
    assert!(close(weighted_average(&[(1.0, 0.2), (1.0, 0.4)]), 0.3));
}

#[test]
fn a_single_dimension_is_its_own_score() {
    assert!(close(weighted_average(&[(25.0, 0.6)]), 0.6));
}

#[test]
fn non_positive_total_weight_reads_zero() {
    assert!(close(weighted_average(&[]), 0.0));
    assert!(close(weighted_average(&[(0.0, 1.0)]), 0.0));
    assert!(close(weighted_average(&[(-2.0, 1.0), (1.0, 1.0)]), 0.0));
}
