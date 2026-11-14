#![allow(clippy::unwrap_used, clippy::expect_used)]
use myrtle_engine_grade::total::*;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-12
}

#[test]
fn grade_ladder_boundaries_are_inclusive() {
    let cases = [
        (1.0, "S+"),
        (0.90, "S+"),
        (0.899_999, "S"),
        (0.75, "S"),
        (0.749_999, "A"),
        (0.60, "A"),
        (0.45, "B"),
        (0.30, "C"),
        (0.15, "D"),
        (0.149_999, "F"),
        (0.0, "F"),
    ];
    for (score, grade) in cases {
        assert_eq!(score_to_grade(score), grade, "score {score}");
    }
}

#[test]
fn out_of_range_scores_land_on_the_ends() {
    assert_eq!(score_to_grade(1.5), "S+");
    assert_eq!(score_to_grade(-0.2), "F");
    // NaN fails every `>=` guard and falls through to the bottom rung.
    assert_eq!(score_to_grade(f64::NAN), "F");
}

#[test]
fn section_weights_give_the_documented_shares() {
    assert!(close(SECTION_WEIGHT_TOTAL, 2.5));
    let share = |w: f64| w / SECTION_WEIGHT_TOTAL;
    assert!(close(share(SECTION_WEIGHT_OPERATOR), 0.34));
    assert!(close(share(SECTION_WEIGHT_STAGE), 0.24));
    assert!(close(share(SECTION_WEIGHT_BASE), 0.14));
    assert!(close(share(SECTION_WEIGHT_ROGUELIKE), 0.12));
    assert!(close(share(SECTION_WEIGHT_MEDAL), 0.08));
    assert!(close(share(SECTION_WEIGHT_SANDBOX), 0.08));
}
