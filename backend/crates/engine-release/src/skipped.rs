use std::collections::{HashMap, HashSet};

use myrtle_gamedata::model::GameModel;

use myrtle_gamedata_raw::tables::item::{ListingKind, tag_number};

use super::{Switches, skins::REVIEW_LAG_MIN_SECS, types::Resolution};

#[derive(Debug, Clone, Default)]
pub struct SkippedTags {
    enabled: bool,
    /// CN start of every entry sorted by tag number.
    cn_start: HashMap<u32, i64>,
    /// Every tag number for EN
    en_tags: HashSet<u32>,
    /// EN tags opened at least [`REVIEW_LAG_MIN_SECS`] after CN
    en_lagged: Vec<u32>,
}

impl SkippedTags {
    #[must_use]
    pub fn build(cn: &GameModel, en: &GameModel, switches: Switches) -> Self {
        Self::from_tags(
            cn.recommend_tags
                .iter()
                .filter_map(|t| Some((tag_number(&t.tag_id)?, t.start_time))),
            en.recommend_tags
                .iter()
                .filter_map(|t| Some((tag_number(&t.tag_id)?, t.start_time))),
            switches.trust_skipped_tags,
        )
    }

    pub fn from_tags(
        cn: impl IntoIterator<Item = (u32, i64)>,
        en: impl IntoIterator<Item = (u32, i64)>,
        enabled: bool,
    ) -> Self {
        let mut cn_start: HashMap<u32, i64> = HashMap::new();
        for (tag, start) in cn {
            cn_start
                .entry(tag)
                .and_modify(|s| *s = (*s).min(start))
                .or_insert(start);
        }
        let mut en_tags = HashSet::new();
        let mut en_lagged = Vec::new();
        for (tag, en_start) in en {
            en_tags.insert(tag);
            if cn_start
                .get(&tag)
                .is_some_and(|&cn| en_start - cn >= REVIEW_LAG_MIN_SECS)
            {
                en_lagged.push(tag);
            }
        }
        Self {
            enabled,
            cn_start,
            en_tags,
            en_lagged,
        }
    }

    /// Whether EN has passed the CN listing with this tag
    pub fn passed(&self, tag_id: Option<&str>, cn_start: i64) -> bool {
        let Some(tag) = tag_id.and_then(tag_number) else {
            return false;
        };
        self.enabled
            && !self.en_tags.contains(&tag)
            && self.en_lagged.iter().any(|&later| {
                later > tag
                    && self
                        .cn_start
                        .get(&later)
                        .is_some_and(|&start| start >= cn_start)
            })
    }

    #[must_use]
    pub fn review_starts(&self, cn: &GameModel) -> HashSet<i64> {
        cn.skin_listings
            .iter()
            .filter(|l| l.kind == ListingKind::Review)
            .filter(|l| self.passed(l.tag_id.as_deref(), l.start_time))
            .map(|l| l.start_time)
            .collect()
    }
}

/// Confirmed > Override > Unlisted > Estimated
#[must_use]
pub fn unless_passed(resolution: Resolution, passed: bool) -> Resolution {
    if passed && matches!(resolution, Resolution::Estimated { .. }) {
        Resolution::Unlisted
    } else {
        resolution
    }
}
