use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MusicKind {
    Battle,
    Lobby,
    Story,
    Map,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MusicPlaceKind {
    Stage,
    Story,
    Season,
    Event,
    Chapter,
    Lobby,
    Map,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MusicRole {
    Menu,
    Battle,
    Challenge,
    Minigame,
    Rerun,
    Background,
    Jukebox,
    Furniture,
    Map,
    Boss,
    Shop,
    Encounter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicPlaceItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicPlace {
    pub kind: MusicPlaceKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<MusicRole>,
    pub items: Vec<MusicPlaceItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicTrack {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    pub kinds: Vec<MusicKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intro_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loop_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    pub banks: Vec<String>,
    pub places: Vec<MusicPlace>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicTotals {
    pub tracks: u32,
    pub titled: u32,
    pub with_audio: u32,
    pub unavailable: u32,
    pub from_other_server: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicIndex {
    pub totals: MusicTotals,
    pub tracks: Vec<MusicTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SirenAlbum {
    pub cid: String,
    pub name: String,
    pub cover_url: String,
    pub artists: Vec<String>,
    pub list_order: i32,
    pub track_count: i32,
    pub lyrics_count: i32,
    pub instrumental_count: i32,
    pub added_on: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SirenSong {
    pub cid: String,
    pub name: String,
    pub title: String,
    pub album_cid: String,
    pub artists: Vec<String>,
    pub has_lyrics: bool,
    pub instrumental: bool,
    pub has_mv: bool,
    pub track_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SirenCatalog {
    pub albums: Vec<SirenAlbum>,
    pub songs: Vec<SirenSong>,
    pub synced_at: Option<DateTime<Utc>>,
    pub bulk_import_day: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SirenAlbumDetail {
    pub cid: String,
    pub name: String,
    pub intro: String,
    pub belong: String,
    pub cover_url: String,
    pub cover_de_url: Option<String>,
    pub artists: Vec<String>,
    pub list_order: i32,
    pub added_on: Option<NaiveDate>,
    pub songs: Vec<SirenSong>,
}
