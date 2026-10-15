//! The Monster Siren Records catalogue's WIRE TYPES.
//!
//! Field names follow the public Monster Siren API (`cid`, `name`, `intro`,
//! `coverUrl`, `coverDeUrl`, `artists`) where a field is passed through, so a
//! reader who knows the label's own site recognises them. Covers are the
//! label's CDN URLs (`web.hycdn.cn/siren/pic/...`), which the CDN serves with
//! `Access-Control-Allow-Origin: *` (measured 2026-10-11), so a page uses them
//! directly. Audio and lyrics go through this backend: audio because the
//! label signs every stream URL for about three hours, lyrics so the file is
//! fetched from the label once and kept.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// One release (album, EP or single) as the catalogue lists it.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SirenAlbum {
    /// The label's catalogue id, the key of every other call.
    pub cid: String,
    pub name: String,
    /// Square cover on the label's CDN.
    pub cover_url: String,
    /// The release's credited artists, whitespace trimmed. The house name
    /// `塞壬唱片-MSR` is a normal entry here; a page that wants guests only
    /// drops the artist that credits the most tracks.
    pub artists: Vec<String>,
    /// Position in the label's list, 0 = newest release.
    pub list_order: i32,
    pub track_count: i32,
    /// Tracks with a lyrics file.
    pub lyrics_count: i32,
    /// Tracks whose title names them an instrumental.
    pub instrumental_count: i32,
    /// The day the cover was uploaded, read from the cover URL's `/pic/YYYYMMDD/`
    /// segment: the label publishes no release date, and this is the closest
    /// thing the data holds. `None` when the URL carries no date.
    pub added_on: Option<NaiveDate>,
}

/// One track as the catalogue lists it. The stream and the lyrics are
/// separate calls.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SirenSong {
    pub cid: String,
    /// The title as published.
    pub name: String,
    /// The title to show: `name` without a trailing `(Instrumental)` or
    /// `（Instrumental）` marker (any case), which `instrumental` already says.
    /// `name` itself when the marker is the whole title or not at the end.
    pub title: String,
    pub album_cid: String,
    pub artists: Vec<String>,
    /// The label publishes an LRC file for this track.
    pub has_lyrics: bool,
    /// The title names the track an instrumental (contains `instrumental`,
    /// any case). Derived from the title: the API carries no flag for it.
    pub instrumental: bool,
    /// The label publishes a music video for it.
    pub has_mv: bool,
    /// Position in the album, 0-based, in the label's album order.
    pub track_order: i32,
}

/// The whole catalogue in one response: every release and every track.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SirenCatalog {
    /// Newest first (by `list_order`).
    pub albums: Vec<SirenAlbum>,
    /// Album order, then track order.
    pub songs: Vec<SirenSong>,
    /// When the sync last read the label's lists. `None` when the catalogue
    /// was loaded from the database and never synced in this process.
    pub synced_at: Option<DateTime<Utc>>,
    /// The upload day shared by the original bulk import of the back
    /// catalogue, when one day accounts for more than half the releases: a
    /// page shows no "on Monster Siren since" for those, because the day says
    /// when the site launched, not when the music came out. `None` when no
    /// single day dominates.
    pub bulk_import_day: Option<NaiveDate>,
}

/// One release in full: the listing fields, the label's blurb, the wide banner
/// and the tracks in album order.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SirenAlbumDetail {
    pub cid: String,
    pub name: String,
    /// The label's blurb, newlines kept. Empty when the label wrote none.
    pub intro: String,
    /// The franchise the label files it under (`arknights` for every release
    /// as of 2026-10-11).
    pub belong: String,
    pub cover_url: String,
    /// The wide (16:10) promotional art, when the label published one.
    pub cover_de_url: Option<String>,
    pub artists: Vec<String>,
    pub list_order: i32,
    pub added_on: Option<NaiveDate>,
    pub songs: Vec<SirenSong>,
}
