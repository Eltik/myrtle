//! Community music stats: the rows and wire types behind `/music/stats`.
//!
//! Tracks are named by the same prefixed id as the library
//! (`database::models::music`): `siren:<cid>` or `game:<track id>`. Every
//! count here is of COUNTED events, after the per-client window
//! (`services::music::stats`), so replaying one track in a loop counts once
//! per window, not once per loop.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// What an event records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicStatKind {
    Listen,
    Download,
}

impl MusicStatKind {
    /// The `music_stat_events.kind` / `music_stat_dedupe.kind` value.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Listen => "listen",
            Self::Download => "download",
        }
    }
}

/// The span a ranking covers, ending now. `week` is the last 7 days and
/// `month` the last 30, both rolling; `all` is every event kept.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, TS, ToSchema,
)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum MusicStatWindow {
    #[default]
    Week,
    Month,
    All,
}

impl MusicStatWindow {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Week => "week",
            Self::Month => "month",
            Self::All => "all",
        }
    }

    /// Days the window reaches back, `None` for all time.
    pub const fn days(self) -> Option<i64> {
        match self {
            Self::Week => Some(7),
            Self::Month => Some(30),
            Self::All => None,
        }
    }
}

/// What a listener ranking orders by: counted listens, or seconds heard as
/// the account's play history records them. Each row carries both.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, TS, ToSchema,
)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum MusicListenerSort {
    #[default]
    Listens,
    Time,
}

impl MusicListenerSort {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Listens => "listens",
            Self::Time => "time",
        }
    }
}

// ---------------------------------------------------------------------------
// Ingest
// ---------------------------------------------------------------------------

/// `POST /music/stats/listen` and `POST /music/stats/download`.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicStatInput {
    /// `siren:<cid>` or `game:<track id>`, a track the catalogue lists.
    pub track_id: String,
}

/// Whether an ingest call counted.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicStatRecorded {
    /// False when the same caller already counted for this track inside the
    /// window (30 minutes for a listen, 24 hours for a download). A client
    /// may call on every play or download; the server decides.
    pub counted: bool,
}

// ---------------------------------------------------------------------------
// Counters
// ---------------------------------------------------------------------------

/// One track's all-time community counts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicTrackCounts {
    #[ts(type = "number")]
    pub listens: i64,
    #[ts(type = "number")]
    pub downloads: i64,
}

/// `GET /music/stats`: every track anyone has listened to or downloaded.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicStatsCatalog {
    /// Track id to its counts. A track absent here has none of either.
    pub tracks: BTreeMap<String, MusicTrackCounts>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MusicTrackCountsRow {
    pub track_id: String,
    pub listens: i64,
    pub downloads: i64,
}

// ---------------------------------------------------------------------------
// Rankings
// ---------------------------------------------------------------------------

/// One row of the top tracks.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicTopTrack {
    /// `rank()` over listens: ties share a rank.
    #[ts(type = "number")]
    pub rank: i64,
    pub track_id: String,
    /// Counted listens inside the window, signed in or not.
    #[ts(type = "number")]
    pub listens: i64,
}

/// `GET /music/stats/top-tracks`.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicTopTracks {
    pub window: MusicStatWindow,
    pub entries: Vec<MusicTopTrack>,
}

/// One ranked listener. Identity fields mirror the score and inventory
/// leaderboards' rows.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicListenerEntry {
    /// `rank()` over the sort: ties share a rank.
    #[ts(type = "number")]
    pub rank: i64,
    pub uid: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub level: Option<i16>,
    pub avatar_id: Option<String>,
    pub server: String,
    /// Counted listens inside the window.
    #[ts(type = "number")]
    pub listens: i64,
    /// Seconds heard inside the window, summed from the account's plays.
    pub listened_seconds: f64,
}

/// `GET /music/stats/top-listeners`, and one track's listeners from
/// `GET /music/stats/tracks/{track_id}/listeners` (`trackId` set).
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicListenerPage {
    /// The track ranked over, absent for the site-wide ranking.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub track_id: Option<String>,
    pub window: MusicStatWindow,
    pub sort: MusicListenerSort,
    pub entries: Vec<MusicListenerEntry>,
    /// Visible listeners in the window: the population the page is cut from.
    #[ts(type = "number")]
    pub total: i64,
}
