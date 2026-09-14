pub mod align;
pub mod art;
pub mod estimate;
pub mod ledger;
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
