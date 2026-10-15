//! The music library's rows and the account-side wire types: likes, listening
//! history and playlists. The catalogue's own wire types live in
//! `services::music::siren::dto`.
//!
//! A track is named by a PREFIXED id in every one of these: `siren:<cid>` for
//! a Monster Siren release, `game:<track id>` for an in-game track, so one
//! like, one play and one playlist row serves both collections.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// The longest playlist name stored, in characters (`music_playlists.name
/// varchar(60)`).
pub const PLAYLIST_NAME_MAX_CHARS: usize = 60;
/// Playlists one account keeps.
pub const PLAYLIST_LIMIT: i64 = 100;
/// Tracks one playlist holds.
pub const PLAYLIST_TRACKS_MAX: usize = 1000;
/// Plays the library response carries, newest first. Older plays still count
/// in the totals.
pub const HISTORY_LIMIT: i64 = 200;

// ---------------------------------------------------------------------------
// Catalogue rows
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SirenAlbumRow {
    pub cid: String,
    pub name: String,
    pub intro: String,
    pub belong: String,
    pub cover_url: String,
    pub cover_de_url: Option<String>,
    pub artists: Vec<String>,
    pub list_order: i32,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SirenSongRow {
    pub cid: String,
    pub album_cid: String,
    pub name: String,
    pub artists: Vec<String>,
    pub lyric_url: Option<String>,
    pub mv_url: Option<String>,
    pub mv_cover_url: Option<String>,
    pub track_order: i32,
    /// The LRC file is already in the row (`lyric_text IS NOT NULL`).
    pub has_lyric_text: bool,
}

// ---------------------------------------------------------------------------
// Account rows
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MusicLikeRow {
    pub track_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MusicPlayRow {
    pub id: i64,
    pub track_id: String,
    pub listened_seconds: f32,
    pub played_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MusicPlayTotalRow {
    pub track_id: String,
    pub plays: i64,
    pub seconds: f64,
    pub last_played_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MusicPlaylistRow {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MusicPlaylistTrackRow {
    pub playlist_id: uuid::Uuid,
    pub track_id: String,
    pub position: i32,
}

// ---------------------------------------------------------------------------
// Wire types
// ---------------------------------------------------------------------------

/// A liked track.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicLike {
    /// `siren:<cid>` or `game:<track id>`.
    pub track_id: String,
    pub liked_at: DateTime<Utc>,
}

/// One listen. A play is logged once the listener has actually heard enough
/// of the track (the page's rule, not the server's) and its listened time is
/// raised as they go on.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicPlay {
    #[ts(type = "number")]
    pub id: i64,
    pub track_id: String,
    pub listened_seconds: f32,
    pub played_at: DateTime<Utc>,
}

/// `POST /music/history`.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicPlayInput {
    pub track_id: String,
    /// Seconds actually heard so far, 0 or more.
    pub listened_seconds: f32,
}

/// `PUT /music/history/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicPlayUpdate {
    /// Seconds actually heard so far. Only ever raises the stored value.
    pub listened_seconds: f32,
}

/// Plays per track over the account's whole history, for the stats view. The
/// page joins these with the catalogue for titles, artists and albums.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicPlayTotal {
    pub track_id: String,
    #[ts(type = "number")]
    pub plays: i64,
    pub seconds: f64,
    pub last_played_at: Option<DateTime<Utc>>,
}

/// A playlist with its tracks in order.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicPlaylist {
    pub id: uuid::Uuid,
    pub name: String,
    pub track_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// `POST /music/playlists`.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicPlaylistCreate {
    pub name: String,
    /// Tracks to start with, in order. Optional.
    #[serde(default)]
    pub track_ids: Vec<String>,
}

/// `PATCH /music/playlists/{id}`: a rename, a reorder, or both. A field left
/// out is unchanged. `track_ids` is the FULL new order; a track left out is
/// removed.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicPlaylistPatch {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub track_ids: Option<Vec<String>>,
}

/// `POST /music/playlists/{id}/tracks`: append these, skipping any already in
/// the playlist.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicPlaylistAdd {
    pub track_ids: Vec<String>,
}

/// Everything the account keeps about music, in one read.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicLibrary {
    /// Newest like first.
    pub likes: Vec<MusicLike>,
    /// The most recent plays, newest first, at most `history_limit`.
    pub history: Vec<MusicPlay>,
    /// How many plays `history` can carry.
    #[ts(type = "number")]
    pub history_limit: i64,
    /// Per-track totals over the WHOLE history, most played first.
    pub totals: Vec<MusicPlayTotal>,
    /// The first play ever logged, for "listening since".
    pub first_played_at: Option<DateTime<Utc>>,
    /// Oldest playlist first.
    pub playlists: Vec<MusicPlaylist>,
    #[ts(type = "number")]
    pub playlist_limit: i64,
    #[ts(type = "number")]
    pub playlist_tracks_max: i64,
}

impl MusicLikeRow {
    pub fn into_like(self) -> MusicLike {
        MusicLike {
            track_id: self.track_id,
            liked_at: self.created_at,
        }
    }
}

impl MusicPlayRow {
    pub fn into_play(self) -> MusicPlay {
        MusicPlay {
            id: self.id,
            track_id: self.track_id,
            listened_seconds: self.listened_seconds,
            played_at: self.played_at,
        }
    }
}

impl MusicPlayTotalRow {
    pub fn into_total(self) -> MusicPlayTotal {
        MusicPlayTotal {
            track_id: self.track_id,
            plays: self.plays,
            seconds: self.seconds,
            last_played_at: self.last_played_at,
        }
    }
}

impl MusicPlaylistRow {
    pub fn into_playlist(self, track_ids: Vec<String>) -> MusicPlaylist {
        MusicPlaylist {
            id: self.id,
            name: self.name,
            track_ids,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
