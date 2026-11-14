pub mod align;
pub mod estimate;
pub mod kinds;
pub mod operator_changes;
pub mod prices;
pub mod skins;
pub mod skipped;
pub mod switches;
pub mod translate;
pub mod types;

use std::collections::{HashMap, HashSet};

pub use switches::Switches;
pub use types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverrideRow {
    pub kind: String,
    pub cn_id: String,
    pub en_id: Option<String>,
    pub en_name: Option<String>,
    pub en_start: Option<i64>,
    pub en_end: Option<i64>,
    pub featured_chars: Option<Vec<String>>,
    pub source: String,
    pub note: String,
}

#[must_use]
pub fn override_index(rows: &[OverrideRow]) -> HashMap<(String, String), &OverrideRow> {
    rows.iter()
        .map(|r| ((r.kind.clone(), r.cn_id.clone()), r))
        .collect()
}

#[must_use]
pub fn override_name(
    kind: &str,
    cn_id: &str,
    overrides: &HashMap<(String, String), &OverrideRow>,
) -> Option<crate::translate::AutoName> {
    let o = overrides.get(&(kind.to_string(), cn_id.to_string()))?;
    let name = o.en_name.as_deref()?.trim();
    (!name.is_empty()).then(|| crate::translate::AutoName {
        text: name.to_string(),
        source: crate::translate::AutoNameSource::Override,
    })
}

#[must_use]
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

#[must_use]
pub fn resolve_reviews(
    cn: &[(i64, i64)],
    en: &[(i64, i64)],
    overrides: &HashMap<(String, String), &OverrideRow>,
    model: &LagModel,
    passed: &HashSet<i64>,
) -> Vec<Resolution> {
    cn.iter()
        .zip(skins::pair_reviews(cn, en))
        .map(|(&(cn_start, _), en_window)| {
            let en_id = en_window.map(|(en_start, _)| format!("review:{en_start}"));
            let confirmed = en_window
                .zip(en_id.as_deref())
                .map(|((en_start, en_end), id)| (id, en_start, en_end));
            skipped::unless_passed(
                resolve(
                    kinds::KIND_REVIEW,
                    &cn_start.to_string(),
                    confirmed,
                    overrides,
                    model,
                    cn_start,
                ),
                passed.contains(&cn_start),
            )
        })
        .collect()
}
