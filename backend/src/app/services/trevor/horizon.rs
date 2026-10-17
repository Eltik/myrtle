//! The spoiler horizon (contract section 1): the era scale, the reader's
//! horizon position and its era bucket.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::app::error::ApiError;

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

/// Spoiler mode (contract v1.1 section 8): `safe` follows the reader's
/// progress; `all` answers from every released story.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "lowercase")]
#[ts(export, rename = "TrevorSpoilerMode")]
#[schema(as = TrevorSpoilerMode)]
pub enum SpoilerMode {
    #[default]
    Safe,
    All,
}

impl SpoilerMode {
    /// The request's `spoilers`, defaulted to `safe`; any other value is a 400.
    /// With spoiler mode switched off (`TREVOR_SPOILER_MODE=0`) the field is
    /// ignored, as it was before spoiler mode: every ask is `safe`.
    pub fn parse_request(value: Option<&str>, enabled: bool) -> Result<Self, ApiError> {
        if !enabled {
            return Ok(Self::Safe);
        }
        match value.map(str::trim) {
            None | Some("safe") => Ok(Self::Safe),
            Some("all") => Ok(Self::All),
            Some(_) => Err(ApiError::BadRequest(
                "spoilers must be \"safe\" or \"all\"".into(),
            )),
        }
    }

    /// The response's `spoilers`: present only for `all`, so a spoiler-safe
    /// response keeps its bytes.
    pub fn reported(self) -> Option<Self> {
        (self == Self::All).then_some(Self::All)
    }
}

/// The horizon of a spoiler-mode ask: the no-limit bucket, which sorts above
/// every era (so no spoiler-safe lookup at or before a reader's era reaches
/// it) and which the worker receives as `horizonPos: null`.
pub fn no_limit_horizon() -> Horizon {
    Horizon {
        era_pos: super::NO_LIMIT_ERA,
        spoiler_safe: None,
    }
}

/// The `horizonPos` a worker receives for a job's stored horizon: the
/// no-limit bucket is sent as null, every other value as it is.
pub fn worker_horizon(horizon_pos: Option<i32>) -> Option<i32> {
    horizon_pos.filter(|p| *p != super::NO_LIMIT_ERA)
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

    #[test]
    fn spoilers_parse_to_safe_by_default_and_refuse_unknown_values() {
        assert_eq!(
            SpoilerMode::parse_request(None, true).unwrap(),
            SpoilerMode::Safe
        );
        assert_eq!(
            SpoilerMode::parse_request(Some("safe"), true).unwrap(),
            SpoilerMode::Safe
        );
        assert_eq!(
            SpoilerMode::parse_request(Some("all"), true).unwrap(),
            SpoilerMode::All
        );
        assert!(matches!(
            SpoilerMode::parse_request(Some("ALL"), true),
            Err(ApiError::BadRequest(_))
        ));
        assert!(SpoilerMode::parse_request(Some("some"), true).is_err());
    }

    #[test]
    fn spoiler_mode_off_ignores_the_field_as_before() {
        assert_eq!(
            SpoilerMode::parse_request(Some("all"), false).unwrap(),
            SpoilerMode::Safe
        );
        assert_eq!(
            SpoilerMode::parse_request(Some("junk"), false).unwrap(),
            SpoilerMode::Safe
        );
    }

    #[test]
    fn the_no_limit_bucket_is_above_every_era_and_reaches_the_worker_as_null() {
        let h = no_limit_horizon();
        assert_ne!(h.era_pos, 0);
        assert!(eras().iter().all(|e| e.position < h.era_pos));
        assert_eq!(h.spoiler_safe, None);
        assert_eq!(worker_horizon(Some(h.era_pos)), None);
        assert_eq!(worker_horizon(Some(21)), Some(21));
        assert_eq!(worker_horizon(Some(0)), Some(0));
        assert_eq!(worker_horizon(None), None);
        assert_eq!(SpoilerMode::All.reported(), Some(SpoilerMode::All));
        assert_eq!(SpoilerMode::Safe.reported(), None);
    }
}
