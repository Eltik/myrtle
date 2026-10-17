//! The spoiler horizon (contract section 1): the era scale, the reader's
//! horizon position and its era bucket.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// An era boundary from the active manifest: `position` is a gatePos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "TrevorEra")]
#[schema(as = TrevorEra)]
pub struct Era {
    pub id: String,
    pub name: String,
    pub position: i32,
}

/// The horizon gatePos: the profile's, lowered (never raised) by the story
/// being read.
pub fn horizon_pos(profile_pos: i32, story_pos: Option<i32>) -> i32 {
    story_pos.map_or(profile_pos, |s| s.min(profile_pos)).max(0)
}

/// The era boundary at or before `pos`, if any.
pub fn era_at(eras: &[Era], pos: i32) -> Option<&Era> {
    eras.iter()
        .filter(|e| e.position > 0 && e.position <= pos)
        .max_by_key(|e| e.position)
}

/// "Spoiler-safe up to" in a response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "TrevorSpoilerSafe")]
#[schema(as = TrevorSpoilerSafe)]
pub struct SpoilerSafe {
    pub era_id: String,
    pub name: String,
    pub position: i32,
}

impl From<&Era> for SpoilerSafe {
    fn from(e: &Era) -> Self {
        Self {
            era_id: e.id.clone(),
            name: e.name.clone(),
            position: e.position,
        }
    }
}

/// A resolved horizon: the gatePos a job runs under (and its answer is
/// cached by), and the label the reader sees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Horizon {
    pub era_pos: i32,
    pub spoiler_safe: Option<SpoilerSafe>,
}

/// Bucket `pos` to its era boundary. With `buckets` off
/// (`TREVOR_ERA_BUCKETS=0`) the job and the cache use `pos` exactly, the
/// pre-era behaviour; the label still names the era at or before it.
pub fn bucket_horizon(eras: &[Era], pos: i32, buckets: bool) -> Horizon {
    let era = era_at(eras, pos);
    Horizon {
        era_pos: if buckets {
            era.map_or(0, |e| e.position)
        } else {
            pos
        },
        spoiler_safe: era.map(SpoilerSafe::from),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eras() -> Vec<Era> {
        [("main_0", 1), ("main_1", 21), ("main_2", 40)]
            .into_iter()
            .map(|(id, position)| Era {
                id: id.into(),
                name: id.to_uppercase(),
                position,
            })
            .collect()
    }

    #[test]
    fn the_request_story_lowers_the_horizon_and_never_raises_it() {
        assert_eq!(horizon_pos(30, None), 30);
        assert_eq!(horizon_pos(30, Some(12)), 12);
        assert_eq!(horizon_pos(30, Some(500)), 30);
        assert_eq!(horizon_pos(0, Some(5)), 0);
    }

    #[test]
    fn the_horizon_buckets_to_the_era_at_or_before_it() {
        let e = eras();
        assert_eq!(bucket_horizon(&e, 0, true).era_pos, 0);
        assert_eq!(bucket_horizon(&e, 0, true).spoiler_safe, None);
        assert_eq!(bucket_horizon(&e, 1, true).era_pos, 1);
        assert_eq!(bucket_horizon(&e, 20, true).era_pos, 1);
        let at_21 = bucket_horizon(&e, 21, true);
        assert_eq!(at_21.era_pos, 21);
        assert_eq!(at_21.spoiler_safe.unwrap().era_id, "main_1");
        assert_eq!(bucket_horizon(&e, 39, true).era_pos, 21);
        assert_eq!(bucket_horizon(&e, 1000, true).era_pos, 40);
    }

    #[test]
    fn era_buckets_off_keys_the_exact_position() {
        let e = eras();
        let h = bucket_horizon(&e, 39, false);
        assert_eq!(h.era_pos, 39);
        assert_eq!(h.spoiler_safe.unwrap().position, 21);
    }
}
