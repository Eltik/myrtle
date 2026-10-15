//! The In-Game Music gallery's WIRE TYPES, and nothing that derives them.
//!
//! Every field's doc comment reaches the `OpenAPI` document and the generated TS
//! binding. The numbers quoted are measured on the EN tree; the census that
//! backs them is printed by `tests/music_real_data_test.rs`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What a track is FOR, derived from the banks that name it and the scripts
/// that play it. A track can be several at once: a theme that scores an event
/// menu and also one of its story scenes is `lobby` and `story`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub enum MusicKind {
    /// A `battle.*` bank or a level's `BgmEvent`: plays during a fight.
    Battle,
    /// A `sys.*` bank outside the map signals: home screen, event menus,
    /// Contingency Contract season pages, the base jukebox.
    Lobby,
    /// Played by a `[PlayMusic]` command in a story script.
    Story,
    /// An Integrated Strategies or Reclamation Algorithm map signal
    /// (`ON_ROGUELIKE_MAP*`, `ON_SANDBOX_MAP*` and their kin).
    Map,
}

/// The kind of place a track plays in, which says how to read the place's
/// `name` and `items`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub enum MusicPlaceKind {
    /// Stages whose level file names the track's bank as its `BgmEvent`.
    /// `name` is the zone, event or season the stages sit in; each item is one
    /// stage (`id` is the stage id the stage viewer opens).
    Stage,
    /// Story scenes that play it. `name` is the story group; each item is one
    /// story (`id` is the story id the reader opens) or, for a script the
    /// story table does not list, the script's file name with no `id`.
    Story,
    /// A Contingency Contract season: its page, its permanent map or its
    /// challenge map. `name` is the season.
    Season,
    /// An event's own menu or minigame tracks. `name` is the event.
    Event,
    /// A main story chapter's menu theme. `name` is the chapter.
    Chapter,
    /// The home screen or the base: a home background that plays it, or the
    /// base jukebox. `name` is the background where there is one.
    Lobby,
    /// An Integrated Strategies or Reclamation Algorithm screen. `name` is the
    /// theme.
    Map,
}

/// What the track does at that place, where the kind alone does not say.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub enum MusicRole {
    /// A menu or page theme.
    Menu,
    /// The fight on a season's permanent map.
    Battle,
    /// The fight once a season's risk passes its challenge point.
    Challenge,
    /// One of an event's minigame or mode tracks.
    Minigame,
    /// The menu of an event's permanent rerun.
    Rerun,
    /// The music a home background plays.
    Background,
    /// A track the base jukebox can play.
    Jukebox,
    /// A piece of base furniture that plays music (a music box, a record
    /// player).
    Furniture,
    /// The node map.
    Map,
    /// The map before a boss floor, or an event boss's entrance
    /// (`activity_table` `ConstData.BossEnterBgmKey`).
    Boss,
    /// The shop.
    Shop,
    /// An encounter or event node.
    Encounter,
}

/// One stage, story or named spot inside a place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MusicPlaceItem {
    /// The stage id or story id the site links to. Absent where there is no
    /// page to open.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<String>,
    /// The short code the game shows (`1-7`, `ST-2`), where there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub code: Option<String>,
    /// The stage or story name, game text.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name: Option<String>,
}

/// One place a track plays.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MusicPlace {
    pub kind: MusicPlaceKind,
    /// The event, season, chapter, story group, zone, theme or background,
    /// game text. Absent where the place has no name of its own.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub role: Option<MusicRole>,
    /// The stages or stories, in the game's order. Empty for a menu.
    pub items: Vec<MusicPlaceItem>,
}

/// One piece of music: a loop and, usually, a one-shot intro in front of it.
///
/// A track is keyed by its LOOP clip, because the game pairs one loop with
/// different intros in 7 of its 509 EN banks and the loop is what a listener
/// hears as the piece.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MusicTrack {
    /// The loop clip's file name with `_loop` cut, lowercase
    /// (`m_bat_act27side_1`). Stable across builds.
    pub id: String,
    /// The title the game gives it: a `Musics` row's `Name`, or the base
    /// jukebox's `BgmName` where no `Musics` row names it. Absent for most
    /// tracks: the game titles only the ones a player can pick or collect.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub title: Option<String>,
    /// The chapter, event, season, theme or Reclamation Algorithm run the
    /// track was made for, game text: the chapter whose theme it is, else a
    /// table's claim (an archive jukebox, a run, a season, a theme's map, a
    /// Story Collection set), else the clip's folder (`act27side`) when it
    /// names an event, else the earliest menu, else the earliest stage zone,
    /// else the event or chapter whose scripts play it most. Absent for a
    /// track with none of these.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub origin: Option<String>,
    /// What the track is for, sorted. Never empty.
    pub kinds: Vec<MusicKind>,
    /// The one-shot intro, a path under `/assets`. Absent when the track has
    /// no intro or the intro clip is not on disk.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub intro_url: Option<String>,
    /// The loop, a path under `/assets`. Absent when the track is
    /// UNAVAILABLE: the game names it but no tree we hold has the clip.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub loop_url: Option<String>,
    /// The server whose tree holds the clips, when it is not the server asked
    /// for. Music is the same file on every server, so a clip the asked
    /// server no longer publishes is served from another that still has it.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub server: Option<String>,
    /// The audio banks that name the track (`battle.ON_GAME_READY.indust`),
    /// sorted. Empty for a track only story scripts play.
    pub banks: Vec<String>,
    /// Where it plays, grouped: stages first, then story, seasons, events,
    /// chapters, lobby and maps.
    pub places: Vec<MusicPlace>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MusicTotals {
    pub tracks: u32,
    pub titled: u32,
    /// Tracks with a playable loop, from any tree.
    pub with_audio: u32,
    pub unavailable: u32,
    /// Of `withAudio`, the tracks served from another server's tree.
    pub from_other_server: u32,
}

/// Every piece of music the game names, for the In-Game Music gallery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MusicIndex {
    pub totals: MusicTotals,
    pub tracks: Vec<MusicTrack>,
}
