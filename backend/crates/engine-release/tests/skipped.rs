#![allow(clippy::unwrap_used, clippy::expect_used)]
use myrtle_engine_release::{skins::REVIEW_LAG_MIN_SECS, skipped::*, types::Resolution};

const D: i64 = 86_400;

fn tags(enabled: bool) -> SkippedTags {
    SkippedTags::from_tags(
        [
            (9, 50 * D),
            (10, 100 * D),
            (11, 100 * D),
            (14, 100 * D),
            (15, 90 * D),
            (16, 120 * D),
            (17, 120 * D),
        ],
        [(9, 210 * D), (15, 250 * D), (16, 280 * D)],
        enabled,
    )
}

#[test]
fn a_skipped_tag_en_has_shipped_past_is_passed() {
    let t = tags(true);
    assert!(t.passed(Some("tag_10"), 100 * D), "tag 16 started day 120");
    assert!(t.passed(Some("tag_14"), 100 * D));
    assert!(!t.passed(Some("tag_9"), 50 * D), "EN carries it");
    assert!(
        !t.passed(Some("tag_17"), 120 * D),
        "no EN tag above 17: EN is not past it yet"
    );
    assert!(
        !t.passed(Some("tag_14"), 130 * D),
        "every later EN tag opened on CN before this listing"
    );
}

#[test]
fn a_later_tag_that_started_earlier_on_cn_does_not_pass() {
    let t = SkippedTags::from_tags([(10, 100 * D), (15, 90 * D)], [(15, 250 * D)], true);
    assert!(
        !t.passed(Some("tag_10"), 100 * D),
        "tag 15 opened before tag 10 on CN"
    );
}

#[test]
fn a_global_release_does_not_move_en_past_anything() {
    let cn = [(10, 100 * D), (16, 101 * D)];
    let together = SkippedTags::from_tags(cn, [(16, 101 * D)], true);
    assert!(
        !together.passed(Some("tag_10"), 100 * D),
        "tag 16 opened on both servers the same day"
    );
    let lagged = SkippedTags::from_tags(cn, [(16, 101 * D + REVIEW_LAG_MIN_SECS)], true);
    assert!(lagged.passed(Some("tag_10"), 100 * D));
}

#[test]
fn an_unparseable_tag_never_passes() {
    let t = tags(true);
    for tag in [None, Some(""), Some("tag_"), Some("tag_1a"), Some("1010")] {
        assert!(!t.passed(tag, 100 * D), "{tag:?}");
    }
}

#[test]
fn the_kill_switch_passes_nothing() {
    let t = tags(false);
    assert!(!t.passed(Some("tag_10"), 100 * D));
    assert!(!t.passed(Some("tag_14"), 100 * D));
}

#[test]
fn only_an_estimate_yields() {
    let est = Resolution::Estimated {
        en_start: 1,
        lo: 0,
        hi: 2,
    };
    let conf = Resolution::Confirmed {
        en_id: "x".into(),
        en_start: 5,
        en_end: 6,
    };
    let ov = Resolution::Override {
        en_id: None,
        en_start: 5,
        en_end: None,
        source: "manual".into(),
        note: String::new(),
    };
    assert_eq!(unless_passed(est.clone(), true), Resolution::Unlisted);
    assert_eq!(unless_passed(est.clone(), false), est);
    assert_eq!(unless_passed(conf.clone(), true), conf);
    assert_eq!(unless_passed(ov.clone(), true), ov);
    assert_eq!(
        unless_passed(Resolution::Unmodelled, true),
        Resolution::Unmodelled
    );
}
