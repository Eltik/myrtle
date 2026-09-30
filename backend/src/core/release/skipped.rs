//! CN outfit-shop listings EN has passed without carrying.
//!
//! CN and EN number recommend-panel entries with one shared `TagId` space
//! (EN `tag_867` is CN `tag_867`, and so on for every listing EN confirmed).
//! EN ships entries roughly in tag order, so once EN carries a tag whose CN
//! entry started at or after a CN listing, EN has moved past that listing's
//! slot; if EN still has no entry with that listing's tag, EN did not list
//! it. The 2026-04-23 Fashion Review (`tag_1022`) is the case that forced
//! this: EN ran that edition as blind boxes from 2026-09-16 with nothing in
//! the outfit shop, carries none of `tag_1010`..`tag_1022`, and ships
//! `tag_1023`..`tag_1033`, so the estimate of CN start + median lag
//! (2026-09-28) was a sale that never came.
//!
//! Only a later entry EN ran on its delayed calendar counts. Global
//! releases open on both servers together: of the 992 EN entries CN also
//! lists, 20 opened within 0.7 d of CN (the Ambience Synesthesia gift packs,
//! starter packs, two Test Collections), 9 within 19.2 to 56.3 d, and the
//! other 963 at 103.2 d or later. `tag_1027`, the 2026 concert pack, opened
//! on EN 2026-04-26, 0.7 d after CN; counted, it calls every skipped tag
//! below it passed months before EN reaches their slot. On the 2026-09-05 EN
//! extract that is 13 CN outfit listings from `tag_1007` up, three of which
//! (`tag_1007`..`tag_1009`) EN then listed on 2026-09-10 to 2026-09-23;
//! without it, none. [`REVIEW_LAG_MIN_SECS`] (90 d) sits in the empty
//! interval between 56.3 and 103.2 d.
//!
//! Only the tag says this. The blind-box window lives in an announcement
//! image, and the item table ties no `RandomSkinbox_*` to an edition
//! (`RandomSkinbox_1` and `_2` were on EN before 2026-09-16, and `_2` and
//! `_3` sold in the same window), so an Unlisted row never says how EN ran it.

use std::collections::{HashMap, HashSet};

use crate::core::gamedata::types::{
    GameData,
    shop::{ListingKind, tag_number},
};

use super::{skins::REVIEW_LAG_MIN_SECS, types::Resolution};

/// Kill switch: `RELEASE_TRUST_SKIPPED_TAGS=0` resolves every CN listing as
/// before, so nothing is ever Unlisted.
fn trust_skipped_tags() -> bool {
    std::env::var("RELEASE_TRUST_SKIPPED_TAGS").ok().as_deref() != Some("0")
}

#[derive(Debug, Clone, Default)]
pub struct SkippedTags {
    enabled: bool,
    /// CN start of every CN recommend entry, by tag number.
    cn_start: HashMap<u32, i64>,
    /// Every tag number EN's recommend panel carries.
    en_tags: HashSet<u32>,
    /// The EN tags EN opened at least [`REVIEW_LAG_MIN_SECS`] after CN: the
    /// entries on EN's delayed calendar.
    en_lagged: Vec<u32>,
}

impl SkippedTags {
    pub fn build(cn: &GameData, en: &GameData) -> Self {
        Self::from_tags(
            cn.recommend_tags
                .iter()
                .filter_map(|t| Some((tag_number(&t.tag_id)?, t.start_time))),
            en.recommend_tags
                .iter()
                .filter_map(|t| Some((tag_number(&t.tag_id)?, t.start_time))),
            trust_skipped_tags(),
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

    /// Whether EN has passed the CN listing with this tag, starting on CN at
    /// `cn_start`, without carrying it. A tag that is not `tag_NNNN` never is.
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

    /// CN starts of the Fashion Review editions EN has passed.
    pub fn review_starts(&self, cn: &GameData) -> HashSet<i64> {
        cn.skin_listings
            .iter()
            .filter(|l| l.kind == ListingKind::Review)
            .filter(|l| self.passed(l.tag_id.as_deref(), l.start_time))
            .map(|l| l.start_time)
            .collect()
    }
}

/// Confirmed > Override > Unlisted > Estimated: only an estimate yields to
/// a passed listing.
pub fn unless_passed(resolution: Resolution, passed: bool) -> Resolution {
    if passed && matches!(resolution, Resolution::Estimated { .. }) {
        Resolution::Unlisted
    } else {
        resolution
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const D: i64 = 86_400;

    /// CN tags 10..=14 on day 100, 15 on day 90 (an older pack numbered
    /// later), 16 and 17 on day 120; EN carries 9, 15 and 16, each 160 d
    /// after CN.
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
}
