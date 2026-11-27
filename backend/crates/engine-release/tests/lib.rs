#![allow(clippy::unwrap_used, clippy::expect_used)]
use std::collections::HashSet;

use myrtle_engine_release::*;

const fn model() -> LagModel {
    LagModel {
        window: 10,
        n: 3,
        median_days: 160.0,
        p25_days: 155.0,
        p75_days: 165.0,
        samples: vec![],
    }
}

fn ov(kind: &str, id: &str, start: Option<i64>) -> OverrideRow {
    OverrideRow {
        kind: kind.into(),
        cn_id: id.into(),
        en_id: None,
        en_name: None,
        featured_chars: None,
        en_start: start,
        en_end: None,
        source: "manual".into(),
        note: String::new(),
    }
}

#[test]
fn review_override_stands_in_until_the_en_listing_lands() {
    const D: i64 = 86_400;
    let m = model();
    let cn = [(0, 28 * D), (200 * D, 228 * D)];
    let key = (200 * D).to_string();
    let before = [(180 * D, 208 * D)];

    let none = override_index(&[]);
    let r = resolve_reviews(&cn, &before, &none, &m, &HashSet::new());
    assert!(matches!(r[0], Resolution::Confirmed { en_start, .. } if en_start == 180 * D));
    assert!(
        matches!(r[1], Resolution::Estimated { .. }),
        "no listing, no override"
    );

    let rows = vec![
        ov(kinds::KIND_REVIEW, &key, Some(350 * D)),
        ov(kinds::KIND_SKIN, "0", Some(1)),
    ];
    let idx = override_index(&rows);
    let r = resolve_reviews(&cn, &before, &idx, &m, &HashSet::new());
    assert!(
        matches!(r[0], Resolution::Confirmed { en_start, .. } if en_start == 180 * D),
        "another kind's override never reaches a review"
    );
    assert!(matches!(r[1], Resolution::Override { en_start, .. } if en_start == 350 * D));

    let after = [(180 * D, 208 * D), (352 * D, 380 * D)];
    let r = resolve_reviews(&cn, &after, &idx, &m, &HashSet::new());
    assert_eq!(
        r[1],
        Resolution::Confirmed {
            en_id: format!("review:{}", 352 * D),
            en_start: 352 * D,
            en_end: 380 * D,
        },
        "the listing wins over the override left in place"
    );

    let blank = vec![ov(kinds::KIND_REVIEW, &key, None)];
    let r = resolve_reviews(&cn, &before, &override_index(&blank), &m, &HashSet::new());
    assert!(
        matches!(r[1], Resolution::Estimated { .. }),
        "an override without a start leaves the estimate"
    );
}

#[test]
fn a_passed_edition_is_unlisted_unless_listed_or_announced() {
    const D: i64 = 86_400;
    let m = model();
    let cn = [(0, 28 * D), (200 * D, 228 * D), (300 * D, 328 * D)];
    let en = [(180 * D, 208 * D)];
    let passed: HashSet<i64> = [0, 200 * D].into_iter().collect();
    let none = override_index(&[]);

    let r = resolve_reviews(&cn, &en, &none, &m, &passed);
    assert!(
        matches!(r[0], Resolution::Confirmed { .. }),
        "a listed edition stays Confirmed whatever its tag says"
    );
    assert_eq!(r[1], Resolution::Unlisted);
    assert!(
        matches!(r[2], Resolution::Estimated { .. }),
        "EN has not shipped past it"
    );

    let key = (200 * D).to_string();
    let rows = vec![ov(kinds::KIND_REVIEW, &key, Some(350 * D))];
    let r = resolve_reviews(&cn, &en, &override_index(&rows), &m, &passed);
    assert!(
        matches!(r[1], Resolution::Override { en_start, .. } if en_start == 350 * D),
        "an announcement outranks the tag"
    );

    let before = resolve_reviews(&cn, &en, &none, &m, &HashSet::new());
    let r = resolve_reviews(&cn, &en, &none, &m, &passed);
    for i in [0, 2] {
        assert_eq!(r[i], before[i], "only the passed edition moves");
    }
}

#[test]
fn sighting_beats_override_beats_estimate() {
    let rows = vec![
        ov("activity", "act1", Some(500)),
        ov("activity", "act2", None),
    ];
    let idx = override_index(&rows);
    let m = model();
    assert!(matches!(
        resolve("activity", "act1", Some(("act1", 900, 950)), &idx, &m, 100),
        Resolution::Confirmed { en_start: 900, .. }
    ));
    assert!(matches!(
        resolve("activity", "act1", None, &idx, &m, 100),
        Resolution::Override { en_start: 500, .. }
    ));
    assert!(matches!(
        resolve("activity", "act2", None, &idx, &m, 86_400),
        Resolution::Estimated { .. }
    ));
    assert_eq!(
        resolve("activity", "act3", None, &idx, &m, 0),
        Resolution::Unmodelled
    );
}
