pub mod align;
pub mod art;
pub mod estimate;
pub mod ledger;
pub mod prices;
pub mod skins;
pub mod types;

use std::collections::HashMap;

use crate::database::queries::release::OverrideRow;

pub use types::*;

pub fn override_index(rows: &[OverrideRow]) -> HashMap<(String, String), &OverrideRow> {
    rows.iter()
        .map(|r| ((r.kind.clone(), r.cn_id.clone()), r))
        .collect()
}

pub fn override_name(
    kind: &str,
    cn_id: &str,
    overrides: &HashMap<(String, String), &OverrideRow>,
) -> Option<crate::core::translate::AutoName> {
    let o = overrides.get(&(kind.to_string(), cn_id.to_string()))?;
    let name = o.en_name.as_deref()?.trim();
    (!name.is_empty()).then(|| crate::core::translate::AutoName {
        text: name.to_string(),
        source: crate::core::translate::AutoNameSource::Override,
    })
}

pub fn resolve(
    kind: &str,
    cn_id: &str,
    confirmed: Option<(&str, i64, i64)>,
    overrides: &HashMap<(String, String), &OverrideRow>,
    model: &LagModel,
    cn_start: i64,
) -> Resolution {
    if let Some((en_id, en_start, en_end)) = confirmed {
        return Resolution::Confirmed {
            en_id: en_id.to_string(),
            en_start,
            en_end,
        };
    }
    if let Some(o) = overrides.get(&(kind.to_string(), cn_id.to_string()))
        && let Some(en_start) = o.en_start
    {
        return Resolution::Override {
            en_id: o.en_id.clone(),
            en_start,
            en_end: o.en_end,
            source: o.source.clone(),
            note: o.note.clone(),
        };
    }
    if cn_start <= 0 {
        return Resolution::Unmodelled;
    }
    estimate::estimate(model, cn_start)
}

/// One resolution per CN Fashion Review edition, in `cn` order. An edition
/// the EN data lists is Confirmed; otherwise an override under
/// `KIND_REVIEW`, keyed by the CN start in unix seconds, stands in for the
/// announced date. The listing still wins once the data carries it, so the
/// row turns Confirmed on the next extract with nobody deleting the override.
pub fn resolve_reviews(
    cn: &[(i64, i64)],
    en: &[(i64, i64)],
    overrides: &HashMap<(String, String), &OverrideRow>,
    model: &LagModel,
) -> Vec<Resolution> {
    cn.iter()
        .zip(skins::pair_reviews(cn, en))
        .map(|(&(cn_start, _), en_window)| {
            let en_id = en_window.map(|(en_start, _)| format!("review:{en_start}"));
            let confirmed = en_window
                .zip(en_id.as_deref())
                .map(|((en_start, en_end), id)| (id, en_start, en_end));
            resolve(
                ledger::KIND_REVIEW,
                &cn_start.to_string(),
                confirmed,
                overrides,
                model,
                cn_start,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn model() -> LagModel {
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
            updated_at: Utc::now(),
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
        let r = resolve_reviews(&cn, &before, &none, &m);
        assert!(matches!(r[0], Resolution::Confirmed { en_start, .. } if en_start == 180 * D));
        assert!(
            matches!(r[1], Resolution::Estimated { .. }),
            "no listing, no override"
        );

        let rows = vec![
            ov(ledger::KIND_REVIEW, &key, Some(350 * D)),
            ov(ledger::KIND_SKIN, "0", Some(1)),
        ];
        let idx = override_index(&rows);
        let r = resolve_reviews(&cn, &before, &idx, &m);
        assert!(
            matches!(r[0], Resolution::Confirmed { en_start, .. } if en_start == 180 * D),
            "another kind's override never reaches a review"
        );
        assert!(matches!(r[1], Resolution::Override { en_start, .. } if en_start == 350 * D));

        let after = [(180 * D, 208 * D), (352 * D, 380 * D)];
        let r = resolve_reviews(&cn, &after, &idx, &m);
        assert_eq!(
            r[1],
            Resolution::Confirmed {
                en_id: format!("review:{}", 352 * D),
                en_start: 352 * D,
                en_end: 380 * D,
            },
            "the listing wins over the override left in place"
        );

        let blank = vec![ov(ledger::KIND_REVIEW, &key, None)];
        let r = resolve_reviews(&cn, &before, &override_index(&blank), &m);
        assert!(
            matches!(r[1], Resolution::Estimated { .. }),
            "an override without a start leaves the estimate"
        );
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
}
