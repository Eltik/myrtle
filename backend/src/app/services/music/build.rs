//! The music index build: every clip the game names, keyed by its loop, with
//! every place a table or a script says it plays.
//!
//! The chain, one source per place kind:
//!
//! - TRACKS come from `audio_data.BgmBanks` (a loop and usually an intro per
//!   bank) and from every `[PlayMusic(intro=, key=)]` in a story script,
//!   resolved through `story_variables`. The `empty` clip the game uses as
//!   silence is not a track.
//! - TITLES are `audio_data.Musics[].Name` through the row's bank, then the
//!   base jukebox's `BgmName` (`building_data.MusicData`).
//! - STAGES are the stage index's level files: each level's `BgmEvent` is a
//!   bank suffix, `battle.ON_GAME_READY.{event}`, followed through
//!   `BankAlias`. A level the asked server no longer ships is read from the
//!   next tree that has it.
//! - STORY scenes are the scripts `story_review_table` lists, plus the scripts
//!   on disk it does not list.
//! - SEASONS are `crisis_table.SeasonInfo` (`PermBgm`, `PermBgmHard`) and
//!   `crisis_v2_table.SeasonInfoDataMap` (`SeasonBgm`, `SeasonBgmChallenge`),
//!   plus the season page banks.
//! - CHAPTERS are `zone_table.MainlineAdditionInfo[].GameMusicId` (then the
//!   `MAINLINE` sets of `stage_table.StorylineStorySets`), named by the zone,
//!   so the side-story zones after Episode 14 (`act2mainss_zone1`) are named.
//! - EVENTS, MAPS and the LOBBY are the `sys.*` banks, by the entity the bank
//!   name ends in (an activity, a rerun, an Integrated Strategies theme, a
//!   Reclamation Algorithm run by `sandbox_perm_table.BasicInfo[].TopicName`),
//!   and the home backgrounds (`display_meta_table.HomeBackgroundData`). The
//!   Integrated Strategies floors are `roguelike_topic_table`'s zone
//!   `BgmSignal`s, each place item a floor. The Reclamation Algorithm fights
//!   are `sandbox_perm_table`'s `BattleBgm*`. Furniture is
//!   `building_data.CustomData.Furnitures[].MusicId`.
//! - ORIGINS are a chapter's theme, else a table's claim (archive jukebox,
//!   run, season, theme, Story Collection set), else the clip folder, else
//!   the earliest menu bank, else the earliest stage zone, else the story
//!   group that plays it most; see the comment where they meet.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;
use serde::Deserialize;

use super::dto::{
    MusicIndex, MusicKind, MusicPlace, MusicPlaceItem, MusicPlaceKind, MusicRole, MusicTotals,
    MusicTrack,
};
use crate::core::gamedata::types::GameData;
use crate::core::story::variables::StoryVariables;

/// What the build reads beyond `GameData`, owned so a `cpu::run` closure can
/// move it.
#[derive(Debug, Clone)]
pub struct MusicSources {
    /// The asked server's text tree: `gamedata/story` and `gamedata/excel`.
    pub assets_dir: PathBuf,
    /// `gamedata/levels` of the asked server first, then the fallbacks.
    pub level_dirs: Vec<PathBuf>,
    /// Art trees in lookup order: `None` for the asked server's own, else the
    /// server code a clip from that tree is served under.
    pub clip_trees: Vec<(Option<String>, PathBuf)>,
}

/// The game's silence placeholder (`Music/obt/empty`, `Battle/b_ui/empty`),
/// which banks use as an intro or loop that plays nothing.
const SILENCE: &str = "empty";

static PLAY_MUSIC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\[\s*playmusic\s*\(([^)\]]*)\)").expect("static regex"));
static MUSIC_ARG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?i)\b(intro|key)\s*=\s*"([^"]*)""#).expect("static regex"));

/// A logical clip path as one key: lowercase, forward slashes, no `audio/`.
fn norm(path: &str) -> String {
    let p = path.trim().replace('\\', "/").to_ascii_lowercase();
    let p = p.trim_start_matches('/');
    p.strip_prefix("audio/").unwrap_or(p).to_owned()
}

fn stem(path: &str) -> &str {
    let last = path.rsplit('/').next().unwrap_or(path);
    last.strip_suffix(".ogg").unwrap_or(last)
}

/// The track id a loop clip makes: its stem with `_loop` cut.
fn loop_key(path: &str) -> String {
    let s = stem(path);
    s.strip_suffix("_loop").unwrap_or(s).to_owned()
}

fn intro_base(path: &str) -> &str {
    let s = stem(path);
    s.strip_suffix("_intro").unwrap_or(s)
}

/// The folder under `music/` a clip sits in (`act27side`, `rogue_3`).
fn clip_folder(path: &str) -> Option<&str> {
    let mut parts = path.split('/');
    parts.by_ref().find(|p| *p == "music")?;
    let folder = parts.next()?;
    // A clip directly under `music/` has no folder of its own.
    parts.next().map(|_| folder)
}

fn trimmed(s: &str) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_owned())
}

#[derive(Debug, Default)]
struct PlaceAcc {
    kind: Option<MusicPlaceKind>,
    name: Option<String>,
    role: Option<MusicRole>,
    items: Vec<MusicPlaceItem>,
    seen: HashSet<(Option<String>, Option<String>, Option<String>)>,
}

#[derive(Debug)]
struct TrackAcc {
    loop_path: String,
    intros: Vec<String>,
    banks: BTreeSet<String>,
    kinds: BTreeSet<MusicKind>,
    title: Option<String>,
    places: Vec<PlaceAcc>,
    place_at: HashMap<(MusicPlaceKind, Option<String>, Option<MusicRole>), usize>,
    /// The main zones whose chapter theme this is.
    chapters: BTreeSet<String>,
    /// The stage index zones (a main zone or an activity id) of its stages.
    stage_zones: BTreeSet<String>,
    /// `[PlayMusic]` calls per story group, the groups the Archives list as
    /// an event or a chapter (`ActType` not `NONE`, so no operator records).
    story_plays: HashMap<String, usize>,
}

impl TrackAcc {
    fn new(loop_path: String) -> Self {
        Self {
            loop_path,
            intros: Vec::new(),
            banks: BTreeSet::new(),
            kinds: BTreeSet::new(),
            title: None,
            places: Vec::new(),
            place_at: HashMap::new(),
            chapters: BTreeSet::new(),
            stage_zones: BTreeSet::new(),
            story_plays: HashMap::new(),
        }
    }

    fn place(
        &mut self,
        kind: MusicPlaceKind,
        name: Option<String>,
        role: Option<MusicRole>,
    ) -> &mut PlaceAcc {
        self.kinds.insert(place_kind_music(kind, role));
        let key = (kind, name.clone(), role);
        let at = *self.place_at.entry(key).or_insert_with(|| {
            self.places.push(PlaceAcc {
                kind: Some(kind),
                name,
                role,
                ..PlaceAcc::default()
            });
            self.places.len() - 1
        });
        &mut self.places[at]
    }
}

impl PlaceAcc {
    fn item(&mut self, id: Option<String>, code: Option<String>, name: Option<String>) {
        if self.seen.insert((id.clone(), code.clone(), name.clone())) {
            self.items.push(MusicPlaceItem { id, code, name });
        }
    }
}

/// What a place says the track is for.
const fn place_kind_music(kind: MusicPlaceKind, role: Option<MusicRole>) -> MusicKind {
    match kind {
        MusicPlaceKind::Stage => MusicKind::Battle,
        MusicPlaceKind::Story => MusicKind::Story,
        MusicPlaceKind::Season => match role {
            Some(MusicRole::Battle | MusicRole::Challenge) => MusicKind::Battle,
            _ => MusicKind::Lobby,
        },
        MusicPlaceKind::Map => match role {
            Some(MusicRole::Map | MusicRole::Boss) => MusicKind::Map,
            Some(MusicRole::Battle) => MusicKind::Battle,
            _ => MusicKind::Lobby,
        },
        MusicPlaceKind::Event if matches!(role, Some(MusicRole::Boss)) => MusicKind::Battle,
        MusicPlaceKind::Event | MusicPlaceKind::Chapter | MusicPlaceKind::Lobby => MusicKind::Lobby,
    }
}

/// What a bank's name says the track is for, with no place attached.
/// `map_banks` are the Integrated Strategies map signals the topic table
/// names (`roguelike_topic_table.Details[].Zones[].BgmSignal` and kin).
fn bank_kind(name: &str, map_banks: &HashSet<String>) -> Option<MusicKind> {
    let mut parts = name.split('.');
    let scope = parts.next()?;
    let event = parts.next().unwrap_or("");
    match scope {
        "battle" | "ui" => Some(MusicKind::Battle),
        "sys" if map_banks.contains(name) || is_sandbox_map_signal(event) => Some(MusicKind::Map),
        "sys" => Some(MusicKind::Lobby),
        _ => None,
    }
}

/// A Reclamation Algorithm map bank (`ON_SANDBOX_MAP`, `_MAPRUSH`,
/// `_DUNGEONMAP`, `_CHALLENGEMAP`), read off the event name: no excel table
/// names the sandbox `sys` signals (a search of every EN table for
/// `ON_SANDBOX` finds only `audio_data`), so the name is the only source.
fn is_sandbox_map_signal(event: &str) -> bool {
    event
        .strip_prefix("ON_SANDBOX_")
        .is_some_and(|rest| rest.contains("MAP"))
}

/// FALLBACK: the role of an Integrated Strategies bank the topic table does
/// not name (`ON_ROGUELIKE_SHOP`, `_EVENT`, `_LOADED`, `_VARIATIONHIGH`),
/// read off the event name. The table-named map signals never reach it.
fn roguelike_role(event: &str) -> MusicRole {
    let rest = event.strip_prefix("ON_ROGUELIKE_").unwrap_or(event);
    if rest.contains("SHOP") {
        MusicRole::Shop
    } else if rest.contains("BOSS") {
        MusicRole::Boss
    } else if rest.contains("EVENT") {
        MusicRole::Encounter
    } else if rest.contains("MAP") || rest.starts_with("VARIATION") {
        MusicRole::Map
    } else {
        MusicRole::Menu
    }
}

#[derive(Default)]
struct Registry {
    tracks: HashMap<String, TrackAcc>,
    /// Bank name -> track id, banks only (aliases resolve through [`Self::bank`]).
    bank_track: HashMap<String, String>,
    alias: HashMap<String, String>,
}

impl Registry {
    /// Register a loop (and its intro) and answer the track id, `None` for
    /// the silence placeholder.
    fn pair(&mut self, loop_path: &str, intro: Option<&str>) -> Option<String> {
        let loop_path = norm(loop_path);
        if stem(&loop_path) == SILENCE || stem(&loop_path).is_empty() {
            return None;
        }
        let key = loop_key(&loop_path);
        let acc = self
            .tracks
            .entry(key.clone())
            .or_insert_with(|| TrackAcc::new(loop_path.clone()));
        if let Some(intro) = intro.map(norm)
            && stem(&intro) != SILENCE
            && intro != acc.loop_path
            && !acc.intros.contains(&intro)
        {
            acc.intros.push(intro);
        }
        Some(key)
    }

    /// [`Self::pair`], answering the track itself.
    fn pair_mut(&mut self, loop_path: &str, intro: Option<&str>) -> Option<&mut TrackAcc> {
        let key = self.pair(loop_path, intro)?;
        self.tracks.get_mut(&key)
    }

    fn bank(&self, name: &str) -> Option<&String> {
        self.own_bank(name)
            .or_else(|| self.bank_track.get(self.alias.get(name)?))
    }

    /// The track of a bank of that name, never through an alias.
    fn own_bank(&self, name: &str) -> Option<&String> {
        self.bank_track.get(name)
    }

    fn track_mut(&mut self, name: &str) -> Option<&mut TrackAcc> {
        let key = self.bank(name)?.clone();
        self.tracks.get_mut(&key)
    }
}

/// The track ids already claimed, by the first table that names an origin.
#[derive(Default)]
struct Origins(HashMap<String, String>);

impl Origins {
    /// The first claim on a track wins.
    fn claim(&mut self, key: Option<&String>, name: Option<String>) {
        if let (Some(key), Some(name)) = (key, name) {
            self.0.entry(key.clone()).or_insert(name);
        }
    }
}

/// `audio_data.Musics` rows by id, as the bank each plays through.
struct MusicRows<'a>(HashMap<&'a str, &'a str>);

impl MusicRows<'_> {
    /// The track a `Musics` row id plays.
    fn track(&self, reg: &Registry, id: &str) -> Option<String> {
        reg.bank(self.0.get(id)?).cloned()
    }
}

/// The battle bank a level or a table names by its suffix.
fn battle_bank(event: &str) -> String {
    format!("battle.ON_GAME_READY.{event}")
}

/// A `sys.*` bank as its event and, when it has one, its entity.
fn sys_parts(bank: &str) -> Option<(&str, Option<&str>)> {
    let mut parts = bank.split('.');
    if parts.next()? != "sys" {
        return None;
    }
    let event = parts.next()?;
    Some((event, parts.next()))
}

/// Every `.ogg` under a tree's music and anonymous-bundle audio, by lowercase
/// stem, as a path under `/assets`.
fn clip_index(art_dir: &Path) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for root in ["audio/audio/sound_beta_2", "audio/anon"] {
        let dir = art_dir.join(root);
        for entry in walkdir::WalkDir::new(&dir).into_iter().flatten() {
            let path = entry.path();
            if !entry.file_type().is_file()
                || path.extension().and_then(|e| e.to_str()) != Some("ogg")
            {
                continue;
            }
            let Ok(rel) = path.strip_prefix(art_dir) else {
                continue;
            };
            let Some(s) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let url = format!("/{}", rel.to_string_lossy().replace('\\', "/"));
            out.entry(s.to_ascii_lowercase()).or_insert(url);
        }
    }
    out
}

/// A level's `BgmEvent`, read as text: the key is a top-level string and
/// the files run to megabytes, so a scan beats a parse.
fn read_bgm_event(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let at = text.find("\"BgmEvent\"")?;
    let rest = &text[at + "\"BgmEvent\"".len()..];
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    trimmed(&rest[..end])
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct CrisisTable {
    #[serde(default)]
    season_info: Vec<CrisisSeason>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct CrisisSeason {
    #[serde(default)]
    season_id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    perm_bgm: Option<String>,
    #[serde(default)]
    perm_bgm_hard: Option<String>,
    #[serde(default)]
    start_ts: i64,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct CrisisV2Table {
    #[serde(default)]
    season_info_data_map: KeyedOrMap<CrisisV2Season>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(rename_all = "PascalCase")]
struct CrisisV2Season {
    #[serde(default)]
    season_id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    season_bgm: Option<String>,
    #[serde(default)]
    season_bgm_challenge: Option<String>,
    #[serde(default)]
    start_ts: i64,
}

/// The tables write a map as `[{key, value}]`; tolerate a plain object too.
#[derive(Deserialize)]
#[serde(untagged)]
enum KeyedOrMap<T> {
    Keyed(Vec<Keyed<T>>),
    Map(HashMap<String, T>),
}

impl<T> Default for KeyedOrMap<T> {
    fn default() -> Self {
        Self::Keyed(Vec::new())
    }
}

impl<T> KeyedOrMap<T> {
    /// The values in table order (a plain object's in map order).
    fn values(&self) -> Vec<&T> {
        match self {
            Self::Keyed(v) => v.iter().map(|k| &k.value).collect(),
            Self::Map(m) => m.values().collect(),
        }
    }

    fn into_values(self) -> Vec<T> {
        match self {
            Self::Keyed(v) => v.into_iter().map(|k| k.value).collect(),
            Self::Map(m) => m.into_values().collect(),
        }
    }

    /// `(key, value)` in table order (a plain object has none, so sorted).
    fn into_pairs(self) -> Vec<(String, T)> {
        match self {
            Self::Keyed(v) => v.into_iter().map(|k| (k.key, k.value)).collect(),
            Self::Map(m) => {
                let mut v: Vec<_> = m.into_iter().collect();
                v.sort_by(|a, b| a.0.cmp(&b.0));
                v
            }
        }
    }
}

/// `activity_table.Activity`: activity type -> `{key: activity id, value}`.
/// Only the fields that name music are read (`ActivityMusic`).
#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct ActivityTable {
    #[serde(default)]
    activity: HashMap<String, Vec<Keyed<ActivityMusic>>>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct ActivityMusic {
    #[serde(default, deserialize_with = "lenient")]
    const_data: Option<ActivityMusicConst>,
    /// A minigame's tracks (`act29side`): `Musics[].Id`s.
    #[serde(default, deserialize_with = "lenient")]
    music_data_map: Vec<ActivityMusicRow>,
    #[serde(default, deserialize_with = "lenient")]
    product_group_data_map: Vec<Keyed<ActivityProductGroup>>,
}

/// A field other activity types shape differently (`ConstData` is a
/// different object per type) reads as empty instead of failing the table.
fn lenient<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    let v = serde_json::Value::deserialize(d)?;
    Ok(T::deserialize(v).unwrap_or_default())
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct ActivityMusicConst {
    /// The event menu's `Musics[].Id` (`act45side`).
    #[serde(default)]
    entry_music_id: Option<String>,
    /// The entity of the boss-entrance bank (`act1vhalfidle_boss_enter` in
    /// `battle.ON_CUSTOM_TRIGGER.act1vhalfidle_boss_enter`).
    #[serde(default)]
    boss_enter_bgm_key: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct ActivityMusicRow {
    #[serde(default)]
    music_id: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct ActivityProductGroup {
    /// A minigame group's default `Musics[].Id`.
    #[serde(default)]
    default_bgm_signal: Option<String>,
}

#[derive(Deserialize)]
struct Keyed<T> {
    #[serde(default)]
    key: String,
    value: T,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct DisplayMeta {
    #[serde(default)]
    home_background_data: HomeBackgroundData,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct HomeBackgroundData {
    #[serde(default)]
    home_bg_data_list: Vec<HomeBg>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct HomeBg {
    #[serde(default)]
    bg_name: String,
    #[serde(default)]
    multi_form_list: Vec<HomeBgForm>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct HomeBgForm {
    #[serde(default)]
    bg_music_id: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct BuildingMusic {
    #[serde(default)]
    music_data: BuildingMusicData,
    #[serde(default)]
    custom_data: BuildingCustom,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct BuildingCustom {
    #[serde(default)]
    furnitures: KeyedOrMap<FurnitureMusic>,
}

/// A piece of furniture; the ones that play music name a `Musics` row.
#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct FurnitureMusic {
    #[serde(default)]
    name: String,
    #[serde(default)]
    music_id: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct ZoneMusicTable {
    #[serde(default)]
    mainline_addition_info: KeyedOrMap<MainlineAddition>,
    /// The main zones in story order; most carry no `ZoneOpenTime`.
    #[serde(default)]
    mainline_zone_id_list: Vec<String>,
}

/// A main story zone's theme: `zone_table.MainlineAdditionInfo[].GameMusicId`.
#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct MainlineAddition {
    #[serde(default)]
    zone_id: String,
    #[serde(default)]
    game_music_id: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct StorySetTable {
    #[serde(default)]
    storyline_story_sets: KeyedOrMap<StorySetMusic>,
}

/// A Story Collection set's theme: a main chapter (`MAINLINE`, through its
/// zone) or an event (`SS`, `COLLECT`, through `RelevantActivityId`).
#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct StorySetMusic {
    #[serde(default)]
    game_music_id: String,
    #[serde(default)]
    relevant_activity_id: Option<String>,
    #[serde(default)]
    mainline_data: Option<StorySetZone>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct StorySetZone {
    #[serde(default)]
    zone_id: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct RoguelikeTopics {
    #[serde(default)]
    details: KeyedOrMap<RoguelikeDetail>,
    #[serde(default)]
    topics: KeyedOrMap<RoguelikeTopic>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct RoguelikeTopic {
    #[serde(default)]
    start_time: i64,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct RoguelikeDetail {
    #[serde(default)]
    zones: KeyedOrMap<RoguelikeZone>,
    #[serde(default)]
    game_const: RoguelikeConst,
    #[serde(default)]
    variation_data: KeyedOrMap<RoguelikeVariation>,
}

/// A floor's map music: `sys.{BgmSignal}.{theme}`, and the low-Sanity
/// variant where the theme has one.
#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct RoguelikeZone {
    #[serde(default)]
    name: String,
    #[serde(default)]
    bgm_signal: Option<String>,
    #[serde(default)]
    bgm_signal_with_low_san: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct RoguelikeConst {
    #[serde(default)]
    sp_zone_shop_bgm_signal: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct RoguelikeVariation {
    #[serde(default)]
    outer_name: String,
    #[serde(default)]
    sound: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct BuildingMusicData {
    #[serde(default)]
    music_datas: KeyedOrMap<JukeboxEntry>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(rename_all = "PascalCase")]
struct JukeboxEntry {
    #[serde(default)]
    bgm_name: String,
    #[serde(default)]
    bgm_sort_id: i64,
    #[serde(default)]
    game_music_id: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct SandboxPerm {
    #[serde(default)]
    basic_info: KeyedOrMap<SandboxTopic>,
    #[serde(default)]
    detail: SandboxDetail,
}

/// A Reclamation Algorithm run's own name (`sandbox_1` is "Tales Within the
/// Sand"), which is NOT the 2023 event `act1sandbox`.
#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct SandboxTopic {
    #[serde(default)]
    topic_id: String,
    #[serde(default)]
    topic_name: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct SandboxMusic {
    #[serde(default)]
    music_id: String,
}

#[derive(Deserialize, Default)]
struct SandboxDetail {
    #[serde(rename = "SANDBOX_V2", default)]
    sandbox_v2: Vec<SandboxKeyed>,
    /// `sandbox_2` "Relaunch Anchor" on CN (1 run), empty on EN until the
    /// 2026-10-28 update. Its bank names sit in `GameConst`, not `BasicConst`.
    #[serde(rename = "SANDBOX_V3", default)]
    sandbox_v3: Vec<SandboxKeyed>,
}

#[derive(Deserialize, Default)]
struct SandboxKeyed {
    #[serde(default)]
    key: String,
    #[serde(default)]
    value: SandboxRun,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct SandboxRun {
    /// `BattleBgmHunt`, `ConstructModeBgmHome` and their kin, each a bank
    /// suffix under `battle.ON_GAME_READY.`.
    #[serde(default)]
    basic_const: HashMap<String, serde_json::Value>,
    /// A V3 run's constants: `BuildModeBgmHome` (`sandbox_2_homebuildmode` on
    /// CN) is its one bank name. V2 runs carry a `GameConst` too, which is not
    /// read for them, so V2 placement does not move.
    #[serde(default)]
    game_const: HashMap<String, serde_json::Value>,
    /// The run's archive jukebox, as `Musics` row ids.
    #[serde(default)]
    archive_music_unlock_data: KeyedOrMap<SandboxMusic>,
}

fn read_table<T: for<'de> Deserialize<'de> + Default>(excel: &Path, name: &str) -> T {
    let path = excel.join(format!("{name}.json"));
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|e| {
            tracing::warn!(table = name, error = %e, "music index: table did not parse");
            T::default()
        }),
        Err(_) => T::default(),
    }
}

/// The names the bank entities resolve to.
struct Names<'a> {
    gd: &'a GameData,
    seasons: HashMap<String, String>,
    /// `sandbox_perm_table.BasicInfo[].TopicName` by topic id.
    sandboxes: HashMap<String, String>,
}

impl Names<'_> {
    /// An activity's name, or the rerun that links it.
    fn activity(&self, id: &str) -> Option<String> {
        if let Some(a) = self.gd.activities.get(id) {
            return trimmed(&a.name);
        }
        let mut reruns: Vec<_> = self
            .gd
            .retro_acts
            .values()
            .filter(|r| r.linked_act_id.iter().any(|l| l == id))
            .collect();
        reruns.sort_by_key(|r| r.index);
        reruns.first().and_then(|r| trimmed(&r.name))
    }

    fn rerun(&self, id: &str) -> Option<String> {
        self.gd.retro_acts.get(id).and_then(|r| trimmed(&r.name))
    }

    fn theme(&self, id: &str) -> Option<String> {
        self.gd
            .roguelike
            .themes
            .get(id)
            .and_then(|t| trimmed(&t.theme_name))
    }

    /// A Reclamation Algorithm run by its topic id, through the topic table.
    fn sandbox(&self, id: &str) -> Option<String> {
        self.sandboxes.get(id).cloned()
    }

    /// The entity a `sys.ON_SANDBOX_*` bank ends in: a run (`sandbox_1`), one
    /// of a run's own modes (`sandbox_1_homebuildmode`, the longest topic id
    /// it extends), or the 2023 event `act1sandbox`, an activity.
    fn sandbox_entity(&self, entity: &str) -> Option<String> {
        self.sandbox(entity)
            .or_else(|| {
                self.sandboxes
                    .iter()
                    .filter(|(id, _)| {
                        entity
                            .strip_prefix(id.as_str())
                            .is_some_and(|r| r.starts_with('_'))
                    })
                    .max_by_key(|(id, _)| id.len())
                    .map(|(_, name)| name.clone())
            })
            .or_else(|| self.activity(entity))
    }

    /// An archive's owner (`story_review_meta_table.ActArchiveData`): an
    /// event, an Integrated Strategies theme or a Reclamation Algorithm run.
    fn archive_owner(&self, id: &str) -> Option<String> {
        self.activity(id)
            .or_else(|| self.theme(id))
            .or_else(|| self.sandbox(id))
    }

    /// FALLBACK, last resort: the event a clip's folder (`act27side`) names,
    /// matched against the activity ids, used only when no chapter or table
    /// claims the track. Names 238 of 569 EN tracks (measured 2026-10-11; 242
    /// before the chapter step went first, 386 as the first source; CN 266 of
    /// 637 and 429 then, not re-measured).
    /// Dropped guesses: `sandbox_N` as event `actNsandbox` (wrong: `sandbox_1`
    /// is "Tales Within the Sand", `act1sandbox` a separate 2023 event),
    /// `act1crisisv2_N` as season `crisis_v2_season_N_1`, `rogue_N` as a
    /// theme. The tables now name all 79 EN tracks in those folders (75 by a
    /// claim, 4 through their season page bank).
    fn folder(&self, folder: &str) -> Option<String> {
        if !folder.starts_with("act") {
            return None;
        }
        // `act18d5d0` is `act18d5`, `act14d0d0` is `act14d0`: the trailing
        // `d0` is a bundle revision the activity id does not carry.
        let mut candidate = folder;
        loop {
            if let Some(name) = self.activity(candidate) {
                return Some(name);
            }
            candidate = candidate.strip_suffix("d0")?;
        }
    }
}

/// The time a place with no date sorts at: after every dated place.
const UNDATED: i64 = i64::MAX;

/// When each place a track plays in opened, so the earliest can be named.
struct Chronology {
    /// A rerun's activity id -> its original: the first activity of the
    /// `retro_table.RetroActList[].LinkedActId` that lists the rerun.
    original: HashMap<String, String>,
    /// A season's `StartTs` by season id, both crisis tables.
    seasons: HashMap<String, i64>,
    /// A theme's `roguelike_topic_table.Topics[].StartTime` by theme id.
    themes: HashMap<String, i64>,
    /// A main zone's open time: its `ZoneOpenTime` where it has one (EN: 5
    /// of 17), else its place in `MainlineZoneIdList`, ahead of every dated
    /// place.
    mainline: HashMap<String, i64>,
}

impl Chronology {
    /// An activity's start and name, a rerun read as its original.
    fn activity(&self, names: &Names<'_>, id: &str) -> Option<(i64, String)> {
        let id = self.original.get(id).map_or(id, String::as_str);
        let start = names.gd.activities.get(id)?.start_time;
        Some((start, names.activity(id)?))
    }

    /// The start and name of the event, season, theme or run a `sys.*` menu
    /// bank belongs to.
    fn menu(&self, names: &Names<'_>, bank: &str) -> Option<(i64, String)> {
        let (event, entity) = sys_parts(bank)?;
        let entity = entity?;
        match event {
            "ON_ACTIVITY_LOADED" | "ON_ACT_CAMP_SELECT" | "ACTIVITY_ACTIVE_TRACKS" => {
                self.activity(names, entity)
            }
            "ON_RETRO_LOADED" => {
                let original = names.gd.retro_acts.get(entity)?.linked_act_id.first()?;
                self.activity(names, original)
            }
            e if e.starts_with("ON_CRISIS_") => Some((
                *self.seasons.get(entity)?,
                names.seasons.get(entity)?.clone(),
            )),
            e if e.starts_with("ON_ROGUELIKE_") => {
                Some((*self.themes.get(entity)?, names.theme(entity)?))
            }
            // Every run's banks are claimed by `sandbox_perm_table` before
            // this is read; the run carries no start of its own here.
            e if e.starts_with("ON_SANDBOX_") => Some((UNDATED, names.sandbox_entity(entity)?)),
            _ => None,
        }
    }
}

/// The place a `sys.*` bank names, when its entity resolves to one.
/// The topic table's map signals reach their banks in their own pass, which
/// names the floor; `map_banks` keeps this one from adding a second place.
fn sys_place(
    names: &Names<'_>,
    bank: &str,
    map_banks: &HashSet<String>,
) -> Option<(MusicPlaceKind, Option<String>, Option<MusicRole>)> {
    let (event, entity) = sys_parts(bank)?;
    let place = match event {
        // The home backgrounds, the jukebox and the furniture reach
        // `ON_MUSIC` banks through their `Musics` rows.
        "ON_MUSIC" => return None,
        _ if map_banks.contains(bank) => return None,
        "ON_ACTIVITY_LOADED" | "ON_ACT_CAMP_SELECT" => (
            MusicPlaceKind::Event,
            entity.and_then(|e| names.activity(e)),
            Some(MusicRole::Menu),
        ),
        "ACTIVITY_ACTIVE_TRACKS" => (
            MusicPlaceKind::Event,
            entity.and_then(|e| names.activity(e)),
            Some(MusicRole::Minigame),
        ),
        "ON_RETRO_LOADED" => (
            MusicPlaceKind::Event,
            entity.and_then(|e| names.rerun(e)),
            Some(MusicRole::Rerun),
        ),
        "ON_CRISIS_SEASON_LOADED" | "ON_CRISIS_V2_SEASON_LOADED" | "ON_CRISIS_NO_SEASON_LOADED" => {
            (
                MusicPlaceKind::Season,
                entity.and_then(|e| names.seasons.get(e).cloned()),
                Some(MusicRole::Menu),
            )
        }
        e if e.starts_with("ON_ROGUELIKE_") => (
            MusicPlaceKind::Map,
            entity.and_then(|t| names.theme(t)),
            Some(roguelike_role(e)),
        ),
        e if e.starts_with("ON_SANDBOX_") => (
            MusicPlaceKind::Map,
            entity.and_then(|t| names.sandbox_entity(t)),
            Some(if is_sandbox_map_signal(e) {
                MusicRole::Map
            } else {
                MusicRole::Menu
            }),
        ),
        _ => (MusicPlaceKind::Lobby, None, Some(MusicRole::Menu)),
    };
    // A place with neither a name nor stages says nothing the kind does not.
    (place.1.is_some() || place.0 == MusicPlaceKind::Lobby).then_some(place)
}

/// Every `[PlayMusic]` in a script as `(loop, intro)` logical paths.
fn script_music<'a>(text: &'a str, vars: &'a StoryVariables) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    for cap in PLAY_MUSIC.captures_iter(text) {
        let mut intro = None;
        let mut key = None;
        for arg in MUSIC_ARG.captures_iter(&cap[1]) {
            let value = arg[2].trim();
            if value.is_empty() {
                continue;
            }
            let resolved = vars.resolve(value).map(str::to_owned);
            if arg[1].eq_ignore_ascii_case("intro") {
                intro = resolved;
            } else {
                key = resolved;
            }
        }
        // A bare `$name` the table does not hold resolves to nothing; a bare
        // logical path resolves to itself.
        if let Some(key) = key.filter(|k| k.contains('/')) {
            out.push((key, intro.filter(|i| i.contains('/'))));
        }
    }
    out
}

/// A Contingency Contract season from either crisis table, with the banks it
/// names as its own.
struct SeasonRow {
    id: String,
    name: String,
    start: i64,
    main: Option<String>,
    challenge: Option<String>,
}

/// Every season: `crisis_table` in table order, then `crisis_v2_table` by
/// season id.
fn read_seasons(excel: &Path) -> Vec<SeasonRow> {
    let crisis: CrisisTable = read_table(excel, "crisis_table");
    let crisis_v2: CrisisV2Table = read_table(excel, "crisis_v2_table");
    let mut v2_seasons = crisis_v2.season_info_data_map.into_values();
    v2_seasons.sort_by(|a, b| a.season_id.cmp(&b.season_id));
    let v1 = crisis.season_info.into_iter().map(|s| SeasonRow {
        id: s.season_id,
        name: s.name,
        start: s.start_ts,
        main: s.perm_bgm,
        challenge: s.perm_bgm_hard,
    });
    let v2 = v2_seasons.into_iter().map(|s| SeasonRow {
        id: s.season_id,
        name: s.name,
        start: s.start_ts,
        main: s.season_bgm,
        challenge: s.season_bgm_challenge,
    });
    v1.chain(v2).collect()
}

/// Every map signal the topic table names, as the bank it plays through.
fn rogue_map_banks(rogue: &[(String, RoguelikeDetail)]) -> HashSet<String> {
    rogue
        .iter()
        .flat_map(|(theme, d)| {
            d.zones
                .values()
                .into_iter()
                .flat_map(|z| [z.bgm_signal.clone(), z.bgm_signal_with_low_san.clone()])
                .chain(
                    d.variation_data
                        .values()
                        .into_iter()
                        .map(|v| v.sound.clone()),
                )
                .flatten()
                .map(move |sig| format!("sys.{sig}.{theme}"))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// A rerun's activity id -> its original: the first activity of the
/// `retro_table.RetroActList[].LinkedActId` that lists the rerun.
fn rerun_originals(gd: &GameData) -> HashMap<String, String> {
    let mut reruns: Vec<_> = gd.retro_acts.values().collect();
    reruns.sort_by_key(|r| r.index);
    let mut original = HashMap::new();
    for r in reruns {
        if let Some((first, rest)) = r.linked_act_id.split_first() {
            for rerun in rest {
                original
                    .entry(rerun.clone())
                    .or_insert_with(|| first.clone());
            }
        }
    }
    original
}

/// A zone's name: the stage index's, else the zone table's.
fn zone_name(gd: &GameData, zone: &str) -> Option<String> {
    gd.stage_index
        .iter()
        .find(|e| e.zone_id == zone)
        .and_then(|e| e.zone_name.clone())
        .or_else(|| {
            let z = gd.zones.get(zone)?;
            z.zone_name_second
                .as_deref()
                .and_then(trimmed)
                .or_else(|| z.zone_name_first.as_deref().and_then(trimmed))
        })
}

/// Build the index for one server.
#[must_use]
pub fn build_music_index(gd: &GameData, src: &MusicSources) -> MusicIndex {
    let excel = src.assets_dir.join("gamedata/excel");
    let vars = StoryVariables::load(&src.assets_dir);

    let seasons = read_seasons(&excel);
    let mut season_names = HashMap::new();
    let mut season_starts = HashMap::new();
    for s in &seasons {
        season_starts.insert(s.id.clone(), s.start);
        if let Some(n) = trimmed(&s.name) {
            season_names.insert(s.id.clone(), n);
        }
    }
    let sandbox: SandboxPerm = read_table(&excel, "sandbox_perm_table");
    let sandboxes = sandbox
        .basic_info
        .into_values()
        .into_iter()
        .filter_map(|t| Some((t.topic_id, trimmed(&t.topic_name)?)))
        .collect();
    let names = Names {
        gd,
        seasons: season_names,
        sandboxes,
    };
    let RoguelikeTopics {
        details: rogue,
        topics,
    } = read_table(&excel, "roguelike_topic_table");
    let rogue = rogue.into_pairs();
    let theme_starts: HashMap<String, i64> = topics
        .into_pairs()
        .into_iter()
        .map(|(id, t)| (id, t.start_time))
        .collect();
    let map_banks = rogue_map_banks(&rogue);

    // 1. Banks.
    let (mut reg, named_banks) = register_banks(gd);

    // What each track was MADE for, as the tables say it. The first claim
    // wins, in this order: an event's archive jukebox
    // (`story_review_meta_table.ActArchiveData`), a Reclamation Algorithm run
    // (`sandbox_perm_table`), a Contingency Contract season's own bank, an
    // Integrated Strategies theme's map signal (`roguelike_topic_table`), a
    // Story Collection event set (`stage_table.StorylineStorySets`).
    // Ruled out by measurement: an original event's own menu bank
    // (`sys.ON_ACTIVITY_LOADED.{act}`) as the first claim moved 10 more EN
    // origins than the order above (and put 2 back): 5 to the right event
    // ("Heart of Surging Flame" for its rerun's name), 5 to a later event
    // that reuses the theme as its menu (`m_sys_w` "Preluding Lights" for
    // "Darknights Memoir").
    let mut origins = Origins::default();
    place_banks(&mut reg, &names, &named_banks, &map_banks);

    let rows = MusicRows(
        gd.music
            .musics()
            .map(|m| (m.id.as_str(), m.bank.as_str()))
            .collect(),
    );

    // 2. Titles, chapter themes and the lobby, all through `Musics` rows.
    let ZoneMusicTable {
        mainline_addition_info,
        mainline_zone_id_list,
    } = read_table(&excel, "zone_table");
    let mut chron = Chronology {
        original: rerun_originals(gd),
        seasons: season_starts,
        themes: theme_starts,
        mainline: mainline_zone_id_list
            .into_iter()
            .zip(0_i64..)
            .map(|(zone, at)| {
                let open = gd.zone_open_times.get(&zone).copied();
                (zone, open.unwrap_or(i64::MIN / 2 + at))
            })
            .collect(),
    };
    let story_sets: StorySetTable = read_table(&excel, "stage_table");
    let story_sets = story_sets.storyline_story_sets.into_pairs();
    let chapter_zone = chapter_zones(gd, mainline_addition_info, &story_sets, &mut chron.mainline);
    place_titles_and_chapters(gd, &mut reg, &chapter_zone);
    place_lobby(&excel, &mut reg, &rows, &named_banks);
    claim_archives(gd, &reg, &names, &rows, &mut origins);

    // 3. Reclamation Algorithm, seasons, Integrated Strategies, the Story
    // Collection and the activity table, claiming in that order.
    place_sandbox_runs(&mut reg, &mut origins, &names, &rows, &sandbox.detail);
    place_seasons(&mut reg, &mut origins, &names, seasons);
    place_rogue(&mut reg, &mut origins, &names, rogue);
    claim_story_sets(&reg, &mut origins, &names, &rows, &story_sets);
    place_activity_music(&excel, &mut reg, &mut origins, &names, &rows, &named_banks);

    // 4. Stages, through each level's `BgmEvent`.
    place_stages(gd, &src.level_dirs, &mut reg);

    // 5. Story scenes: every listed story, then the scripts the table skips.
    let listed = place_listed_stories(gd, &src.assets_dir, &vars, &mut reg);
    place_unlisted_stories(
        &src.assets_dir.join("gamedata/story"),
        &listed,
        &vars,
        &mut reg,
        &names,
    );

    // 6. Clips, origin and the wire shape.
    assemble(reg, origins, &names, &chron, &src.clip_trees)
}

/// Every bank's loop and intro as a track, sorted so the first intro a loop
/// meets is the same every build, and every bank or alias name that reaches
/// one, sorted.
fn register_banks(gd: &GameData) -> (Registry, Vec<String>) {
    let mut reg = Registry {
        alias: gd
            .music
            .aliases()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect(),
        ..Registry::default()
    };
    let mut banks: Vec<_> = gd.music.banks().collect();
    banks.sort_by(|a, b| a.name.cmp(&b.name));
    for bank in &banks {
        let Some(loop_path) = bank.loop_.as_deref() else {
            continue;
        };
        if let Some(key) = reg.pair(loop_path, bank.intro.as_deref()) {
            reg.bank_track.insert(bank.name.clone(), key);
        }
    }
    let mut named_banks: Vec<String> = reg.bank_track.keys().cloned().collect();
    named_banks.extend(reg.alias.keys().filter(|a| reg.bank(a).is_some()).cloned());
    named_banks.sort();
    (reg, named_banks)
}

/// Each bank on its track, with the kind its name says and the place its
/// `sys.*` entity names.
fn place_banks(
    reg: &mut Registry,
    names: &Names<'_>,
    named_banks: &[String],
    map_banks: &HashSet<String>,
) {
    for bank in named_banks {
        let kind = bank_kind(bank, map_banks);
        let place = sys_place(names, bank, map_banks);
        if let Some(acc) = reg.track_mut(bank) {
            acc.banks.insert(bank.clone());
            if let Some(k) = kind {
                acc.kinds.insert(k);
            }
            if let Some((kind, name, role)) = place {
                acc.place(kind, name, role);
            }
        }
    }
}

/// Music id -> main story zone: the zone table, then the Story Collection.
/// Each zone the zone table names is dated into `mainline` on the way.
fn chapter_zones(
    gd: &GameData,
    additions: KeyedOrMap<MainlineAddition>,
    story_sets: &[(String, StorySetMusic)],
    mainline: &mut HashMap<String, i64>,
) -> HashMap<String, String> {
    let mut chapter_zone: HashMap<String, String> = HashMap::new();
    for info in additions.into_values() {
        // A main zone outside `MainlineZoneIdList` (`act2mainss_zone1`, the
        // side chapters after Episode 14) dates by its `ZoneOpenTime` or not
        // at all.
        if !info.zone_id.is_empty() {
            let open = gd.zone_open_times.get(&info.zone_id).copied();
            mainline
                .entry(info.zone_id.clone())
                .or_insert_with(|| open.unwrap_or(UNDATED));
        }
        if !info.game_music_id.is_empty() && !info.zone_id.is_empty() {
            chapter_zone
                .entry(info.game_music_id)
                .or_insert(info.zone_id);
        }
    }
    for (_, set) in story_sets {
        if let Some(zone) = set.mainline_data.as_ref().and_then(|m| m.zone_id.clone())
            && !set.game_music_id.is_empty()
        {
            chapter_zone
                .entry(set.game_music_id.clone())
                .or_insert(zone);
        }
    }
    chapter_zone
}

/// Each `Musics` row's title onto its track, and the chapter whose theme it
/// is.
fn place_titles_and_chapters(
    gd: &GameData,
    reg: &mut Registry,
    chapter_zone: &HashMap<String, String>,
) {
    let mut musics: Vec<_> = gd.music.musics().collect();
    musics.sort_by(|a, b| a.id.cmp(&b.id));
    for m in &musics {
        let Some(key) = reg.bank(&m.bank).cloned() else {
            continue;
        };
        let Some(acc) = reg.tracks.get_mut(&key) else {
            continue;
        };
        if acc.title.is_none() {
            acc.title = trimmed(&m.name);
        }
        if let Some(zone) = chapter_zone.get(&m.id) {
            acc.chapters.insert(zone.clone());
            acc.place(
                MusicPlaceKind::Chapter,
                zone_name(gd, zone),
                Some(MusicRole::Menu),
            );
        }
    }
}

/// The lobby: the home backgrounds, the base jukebox (a title where the
/// `Musics` row has none) and the furniture that plays music.
fn place_lobby(excel: &Path, reg: &mut Registry, rows: &MusicRows<'_>, named_banks: &[String]) {
    let display: DisplayMeta = read_table(excel, "display_meta_table");
    for bg in &display.home_background_data.home_bg_data_list {
        for form in &bg.multi_form_list {
            let Some(key) = rows.track(reg, &form.bg_music_id) else {
                continue;
            };
            if let Some(acc) = reg.tracks.get_mut(&key) {
                acc.place(
                    MusicPlaceKind::Lobby,
                    trimmed(&bg.bg_name),
                    Some(MusicRole::Background),
                );
            }
        }
    }
    let building: BuildingMusic = read_table(excel, "building_data");
    let mut jukebox = building.music_data.music_datas.into_values();
    jukebox.sort_by_key(|j| j.bgm_sort_id);
    for entry in &jukebox {
        let Some(key) = rows.track(reg, &entry.game_music_id) else {
            continue;
        };
        if let Some(acc) = reg.tracks.get_mut(&key) {
            if acc.title.is_none() {
                acc.title = trimmed(&entry.bgm_name);
            }
            acc.place(MusicPlaceKind::Lobby, None, Some(MusicRole::Jukebox));
        }
    }
    // Furniture that plays music, named by `building_data.CustomData`.
    for (_, furniture) in building.custom_data.furnitures.into_pairs() {
        let Some(key) = furniture
            .music_id
            .as_deref()
            .and_then(|id| rows.track(reg, id))
        else {
            continue;
        };
        if let Some(acc) = reg.tracks.get_mut(&key) {
            acc.place(
                MusicPlaceKind::Lobby,
                trimmed(&furniture.name),
                Some(MusicRole::Furniture),
            );
        }
    }
    // FALLBACK: a `sys.ON_MUSIC.furniture_*` bank no furniture row names
    // (EN: `furniture_audioPlayer`, 1 track) is still furniture, read off the
    // bank name, with no name of its own.
    for bank in named_banks {
        let is_furniture = bank
            .strip_prefix("sys.ON_MUSIC.")
            .is_some_and(|e| e.starts_with("furniture_"));
        if let Some(acc) = reg.track_mut(bank).filter(|_| is_furniture)
            && !acc
                .places
                .iter()
                .any(|p| p.role == Some(MusicRole::Furniture))
        {
            acc.place(MusicPlaceKind::Lobby, None, Some(MusicRole::Furniture));
        }
    }
}

/// The event archives' jukeboxes. An original run claims before its rerun
/// (`act13side` before `act13sre`, which lists the same tracks).
fn claim_archives(
    gd: &GameData,
    reg: &Registry,
    names: &Names<'_>,
    rows: &MusicRows<'_>,
    origins: &mut Origins,
) {
    let mut archives: Vec<_> = gd
        .story_archives
        .act_archive_data
        .components
        .iter()
        .collect();
    archives.sort_by_key(|(id, _)| {
        let act = gd.activities.get(id.as_str());
        (
            act.is_some_and(|a| a.is_replicate),
            act.map_or(i64::MAX, |a| a.start_time),
            (*id).clone(),
        )
    });
    for (id, component) in archives {
        let owner = names.archive_owner(id);
        for item in component.music.iter().flat_map(|m| &m.items) {
            origins.claim(rows.track(reg, &item.value.id).as_ref(), owner.clone());
        }
    }
}

/// Reclamation Algorithm: the run's archive jukebox, then its fight and
/// build-mode banks.
fn place_sandbox_runs(
    reg: &mut Registry,
    origins: &mut Origins,
    names: &Names<'_>,
    rows: &MusicRows<'_>,
    detail: &SandboxDetail,
) {
    let v2 = detail
        .sandbox_v2
        .iter()
        .map(|run| (run, &run.value.basic_const));
    let v3 = detail
        .sandbox_v3
        .iter()
        .map(|run| (run, &run.value.game_const));
    for (run, run_consts) in v2.chain(v3) {
        let name = names.sandbox(&run.key);
        for music in run.value.archive_music_unlock_data.values() {
            origins.claim(rows.track(reg, &music.music_id).as_ref(), name.clone());
        }
        let mut consts: Vec<_> = run_consts.iter().collect();
        consts.sort_by(|a, b| a.0.cmp(b.0));
        for (field, value) in consts {
            let Some(bgm) = value.as_str().and_then(trimmed) else {
                continue;
            };
            let role = if field.starts_with("BattleBgm") {
                MusicRole::Battle
            } else if field.contains("Bgm") {
                MusicRole::Menu
            } else {
                continue;
            };
            let bank = battle_bank(&bgm);
            origins.claim(reg.own_bank(&bank), name.clone());
            if let Some(acc) = reg.track_mut(&bank) {
                acc.place(MusicPlaceKind::Map, name.clone(), Some(role));
            }
        }
    }
}

/// Contingency Contract seasons. A season claims a track only through a bank
/// of its own: `rune_season_3_1_main` is an alias of `m_bat_katzdale`, which
/// the season borrows.
fn place_seasons(
    reg: &mut Registry,
    origins: &mut Origins,
    names: &Names<'_>,
    seasons: Vec<SeasonRow>,
) {
    for s in seasons {
        let name = names.seasons.get(&s.id).cloned();
        let main = s.main.and_then(|m| trimmed(&m));
        let challenge = s
            .challenge
            .and_then(|c| trimmed(&c))
            .filter(|c| Some(c) != main.as_ref());
        for (bgm, role) in [(main, MusicRole::Battle), (challenge, MusicRole::Challenge)] {
            let Some(bgm) = bgm else { continue };
            let bank = battle_bank(&bgm);
            origins.claim(reg.own_bank(&bank), name.clone());
            if let Some(acc) = reg.track_mut(&bank) {
                acc.place(MusicPlaceKind::Season, name.clone(), Some(role));
            }
        }
    }
}

/// Integrated Strategies: each floor's map music, the floor named; each
/// variation's; the special-zone shop's.
fn place_rogue(
    reg: &mut Registry,
    origins: &mut Origins,
    names: &Names<'_>,
    rogue: Vec<(String, RoguelikeDetail)>,
) {
    for (theme, detail) in rogue {
        let name = names.theme(&theme);
        let mut spots: Vec<(String, MusicRole, Option<String>)> = Vec::new();
        for zone in detail.zones.into_values() {
            for sig in [&zone.bgm_signal, &zone.bgm_signal_with_low_san]
                .into_iter()
                .flatten()
            {
                spots.push((sig.clone(), MusicRole::Map, trimmed(&zone.name)));
            }
        }
        for variation in detail.variation_data.into_values() {
            if let Some(sig) = variation.sound {
                spots.push((sig, MusicRole::Map, trimmed(&variation.outer_name)));
            }
        }
        if let Some(sig) = detail.game_const.sp_zone_shop_bgm_signal {
            spots.push((sig, MusicRole::Shop, None));
        }
        for (sig, role, spot) in spots {
            let bank = format!("sys.{sig}.{theme}");
            origins.claim(reg.bank(&bank), name.clone());
            if let Some(acc) = reg.track_mut(&bank) {
                let place = acc.place(MusicPlaceKind::Map, name.clone(), Some(role));
                if spot.is_some() {
                    place.item(None, None, spot);
                }
            }
        }
    }
}

/// The Story Collection's event sets name the event's theme.
fn claim_story_sets(
    reg: &Registry,
    origins: &mut Origins,
    names: &Names<'_>,
    rows: &MusicRows<'_>,
    story_sets: &[(String, StorySetMusic)],
) {
    for (_, set) in story_sets {
        let Some(act) = set.relevant_activity_id.as_deref() else {
            continue;
        };
        origins.claim(
            rows.track(reg, &set.game_music_id).as_ref(),
            names.activity(act),
        );
    }
}

/// The activity table's own music fields, the last claims: a minigame's
/// tracks (`MusicDataMap`, `ProductGroupDataMap[].DefaultBgmSignal`), the
/// event menu (`ConstData.EntryMusicId`) and a boss's entrance
/// (`ConstData.BossEnterBgmKey`, the entity of its bank). A rerun is
/// skipped: it repeats its original's rows (`act29sre` lists the same 17
/// `MusicDataMap` tracks as `act29side`), and naming it would add a second
/// event place to tracks the original already names. EN 2026-10-11: one
/// track gains a place (`m_boss_act1vhalfidle`, "Rebuilding Mandate",
/// Boss), Event places 121 -> 122 tracks, tracks with no place 32 -> 31;
/// the act29side and act45side tracks already had these places through
/// their `sys.*` banks, and no origin moves (554 named before and after).
fn place_activity_music(
    excel: &Path,
    reg: &mut Registry,
    origins: &mut Origins,
    names: &Names<'_>,
    rows: &MusicRows<'_>,
    named_banks: &[String],
) {
    let gd = names.gd;
    let activity_music: ActivityTable = read_table(excel, "activity_table");
    let mut act_rows: Vec<&Keyed<ActivityMusic>> = activity_music
        .activity
        .values()
        .flatten()
        .filter(|row| {
            !gd.activities
                .get(row.key.as_str())
                .is_some_and(|a| a.is_replicate)
        })
        .collect();
    act_rows.sort_by_key(|row| {
        let act = gd.activities.get(row.key.as_str());
        (act.map_or(i64::MAX, |a| a.start_time), row.key.clone())
    });
    for row in act_rows {
        let name = names.activity(&row.key);
        let mut spots: Vec<(Option<String>, MusicRole)> = Vec::new();
        for m in &row.value.music_data_map {
            spots.push((rows.track(reg, &m.music_id), MusicRole::Minigame));
        }
        for group in &row.value.product_group_data_map {
            if let Some(id) = group.value.default_bgm_signal.as_deref() {
                spots.push((rows.track(reg, id), MusicRole::Minigame));
            }
        }
        if let Some(c) = &row.value.const_data {
            if let Some(id) = c.entry_music_id.as_deref() {
                spots.push((rows.track(reg, id), MusicRole::Menu));
            }
            if let Some(entity) = c.boss_enter_bgm_key.as_deref().and_then(trimmed) {
                let bank = named_banks
                    .iter()
                    .find(|b| b.split('.').nth(2) == Some(entity.as_str()));
                spots.push((bank.and_then(|b| reg.bank(b)).cloned(), MusicRole::Boss));
            }
        }
        for (key, role) in spots {
            let Some(key) = key else { continue };
            origins.claim(Some(&key), name.clone());
            if let Some(acc) = reg.tracks.get_mut(&key) {
                acc.place(MusicPlaceKind::Event, name.clone(), Some(role));
            }
        }
    }
}

/// Each stage on the track its level's `BgmEvent` names, the level read from
/// the first of `level_dirs` that has it.
fn place_stages(gd: &GameData, level_dirs: &[PathBuf], reg: &mut Registry) {
    let mut events: HashMap<String, Option<String>> = HashMap::new();
    for entry in &gd.stage_index {
        let Some(level) = entry.level_id.as_deref() else {
            continue;
        };
        let level = level.trim_start_matches('/').to_ascii_lowercase();
        let event = events
            .entry(level.clone())
            .or_insert_with(|| {
                level_dirs
                    .iter()
                    .find_map(|dir| read_bgm_event(&dir.join(format!("{level}.json"))))
            })
            .clone();
        let Some(event) = event else { continue };
        let bank = battle_bank(&event);
        let Some(key) = reg.bank(&bank).or_else(|| reg.bank(&event)).cloned() else {
            continue;
        };
        if let Some(acc) = reg.tracks.get_mut(&key) {
            acc.stage_zones.insert(entry.zone_id.clone());
            let zone = entry.zone_name.clone().or_else(|| trimmed(&entry.zone_id));
            acc.place(MusicPlaceKind::Stage, zone, None).item(
                Some(entry.stage_id.clone()),
                trimmed(&entry.code),
                entry.name.as_deref().and_then(trimmed),
            );
        }
    }
}

/// Every story `story_review_table` lists, in group start order. Answers the
/// scripts it read, lowercase and without the extension, so the unlisted pass
/// skips them.
fn place_listed_stories(
    gd: &GameData,
    assets_dir: &Path,
    vars: &StoryVariables,
    reg: &mut Registry,
) -> HashSet<String> {
    let mut groups: Vec<_> = gd.story_reviews.values().collect();
    groups.sort_by(|a, b| {
        a.start_time
            .cmp(&b.start_time)
            .then_with(|| a.id.cmp(&b.id))
    });
    let mut listed: HashSet<String> = HashSet::new();
    for group in groups {
        let mut stories: Vec<_> = group.info_unlock_datas.iter().collect();
        stories.sort_by_key(|s| s.story_sort);
        for story in stories {
            if story.story_txt.trim().is_empty() {
                continue;
            }
            listed.insert(story.story_txt.trim().to_ascii_lowercase());
            let Ok(text) = crate::core::story::load_script(assets_dir, &story.story_txt) else {
                continue;
            };
            for (loop_path, intro) in script_music(&text, vars) {
                let Some(acc) = reg.pair_mut(&loop_path, intro.as_deref()) else {
                    continue;
                };
                if group.act_type != "NONE" {
                    *acc.story_plays.entry(group.id.clone()).or_default() += 1;
                }
                acc.place(MusicPlaceKind::Story, trimmed(&group.name), None)
                    .item(
                        Some(story.story_id.clone()),
                        story.story_code.as_deref().and_then(trimmed),
                        trimmed(&story.story_name),
                    );
            }
        }
    }
    listed
}

/// The scripts on disk the table does not list, each named by the theme or
/// run its path names and itemised by file name.
fn place_unlisted_stories(
    story_dir: &Path,
    listed: &HashSet<String>,
    vars: &StoryVariables,
    reg: &mut Registry,
    names: &Names<'_>,
) {
    let mut unlisted: Vec<PathBuf> = walkdir::WalkDir::new(story_dir)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|p| p.to_string_lossy().ends_with(".txt.txt"))
        .collect();
    unlisted.sort();
    for path in unlisted {
        let Ok(rel) = path.strip_prefix(story_dir) else {
            continue;
        };
        let rel = rel
            .to_string_lossy()
            .replace('\\', "/")
            .to_ascii_lowercase();
        if rel.starts_with("[uc]info/") {
            continue;
        }
        let txt = rel.trim_end_matches(".txt.txt");
        if listed.contains(txt) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let found = script_music(&text, vars);
        if found.is_empty() {
            continue;
        }
        let group = txt.split('/').find_map(|seg| {
            if seg.starts_with("rogue_") {
                names.theme(seg)
            } else if seg.starts_with("sandbox_") {
                names.sandbox(seg)
            } else {
                None
            }
        });
        let file = txt.rsplit('/').next().map(str::to_owned);
        for (loop_path, intro) in found {
            if let Some(acc) = reg.pair_mut(&loop_path, intro.as_deref()) {
                acc.place(MusicPlaceKind::Story, group.clone(), None).item(
                    None,
                    None,
                    file.clone(),
                );
            }
        }
    }
}

/// The wire shape: each track by id with its clips from the first tree that
/// holds its loop, its origin, its places by kind, and the totals.
fn assemble(
    mut reg: Registry,
    mut origins: Origins,
    names: &Names<'_>,
    chron: &Chronology,
    clip_trees: &[(Option<String>, PathBuf)],
) -> MusicIndex {
    let trees: Vec<(Option<String>, HashMap<String, String>)> = clip_trees
        .iter()
        .map(|(server, dir)| (server.clone(), clip_index(dir)))
        .collect();
    let mut keys: Vec<String> = reg.tracks.keys().cloned().collect();
    keys.sort();
    let mut tracks = Vec::with_capacity(keys.len());
    let mut totals = MusicTotals {
        tracks: 0,
        titled: 0,
        with_audio: 0,
        unavailable: 0,
        from_other_server: 0,
    };
    for key in keys {
        let Some(mut acc) = reg.tracks.remove(&key) else {
            continue;
        };
        // The intro that shares the loop's stem is the loop's own; another
        // bank's lead-in is the fallback.
        let intro = acc
            .intros
            .iter()
            .find(|i| intro_base(i) == key)
            .or_else(|| acc.intros.first())
            .cloned();
        let loop_stem = stem(&acc.loop_path).to_owned();
        let tree = trees
            .iter()
            .find(|(_, clips)| clips.contains_key(&loop_stem));
        let (server, loop_url, intro_url) = match tree {
            Some((server, clips)) => (
                server.clone(),
                clips.get(&loop_stem).cloned(),
                intro.as_deref().and_then(|i| clips.get(stem(i)).cloned()),
            ),
            None => (None, None, None),
        };
        let origin = origin_of(&acc, origins.0.remove(&key), names, chron);
        let mut places: Vec<PlaceAcc> = std::mem::take(&mut acc.places);
        places.sort_by_key(|p| p.kind);
        totals.tracks += 1;
        if acc.title.is_some() {
            totals.titled += 1;
        }
        if loop_url.is_some() {
            totals.with_audio += 1;
            if server.is_some() {
                totals.from_other_server += 1;
            }
        } else {
            totals.unavailable += 1;
        }
        tracks.push(MusicTrack {
            id: key,
            title: acc.title,
            origin,
            kinds: acc.kinds.into_iter().collect(),
            intro_url,
            loop_url,
            server,
            banks: acc.banks.into_iter().collect(),
            places: places
                .into_iter()
                .filter_map(|p| {
                    Some(MusicPlace {
                        kind: p.kind?,
                        name: p.name,
                        role: p.role,
                        items: p.items,
                    })
                })
                .collect(),
        });
    }
    MusicIndex { totals, tracks }
}

/// Where a track comes from, the first of these that names one:
///
/// 1. CHAPTER: the main zone whose theme it is
///    (`zone_table.MainlineAdditionInfo[].GameMusicId`), the earliest.
/// 2. CLAIM: a table's claim, in the order the claims were made.
/// 3. FOLDER: the clip's folder, when it names an event.
/// 4. MENU: the earliest event, season, theme or run whose `sys.*` bank plays
///    it, a rerun read as its original.
/// 5. STAGE: the earliest zone, a main zone or an event, whose stages play it.
/// 6. STORY: the event or chapter whose scripts play it MOST (`[PlayMusic]`
///    calls), the earliest on a tie.
///
/// Scored against an outside index of the EN tracks (2026-10-11; 574 there,
/// 569 here, 564 joined by loop clip): 529 agree, from 471 with a claim, the
/// folder and then the first named place, and 469 with the folder first.
/// The story step buys 21 over the earliest story group (508), which names
/// the old chapter a later event's track was reused in (`m_avg_calm`: 8 calls
/// in "The Masses' Travels", 1 in "Return To Mist"); counting stories rather
/// than calls is 522. The chapter step buys 6 (523 without it), the menu step
/// 25 (504 without it, 17 of them Contingency Contract season pages), a
/// rerun read as its original 4 (525). A story tie goes to the earlier group:
/// the later is 527. Ruled out by measurement: the claims ahead of the
/// chapter (528: CC#2 claims `m_bat_longmenoffice`, the Episode 05 theme);
/// the folder ahead of the claims (527); a theme's own `sys.ON_ROGUELIKE_*`
/// banks ahead of the claims (527: right for `m_sys_rglk1shop`, wrong for 3
/// Ceobe's Fungimist tracks the later themes reuse); the earliest of every
/// source at once (489); the story step ahead of the stage step (509); the
/// stage zone used MOST (530 or 527 by tie-break, so a tie decides it, not
/// the rule); the first named place as a last step (527: it names a tower
/// or a training stage, which the outside index leaves without an origin).
fn origin_of(
    acc: &TrackAcc,
    claimed: Option<String>,
    names: &Names<'_>,
    chron: &Chronology,
) -> Option<String> {
    let dated_zone = |zone: &str| -> Option<(i64, String)> {
        match chron.mainline.get(zone) {
            Some(&at) => Some((at, zone_name(names.gd, zone)?)),
            None => chron.activity(names, zone),
        }
    };
    let earliest = |dated: Vec<(i64, String)>| dated.into_iter().min().map(|(_, name)| name);
    let chapter = || earliest(acc.chapters.iter().filter_map(|z| dated_zone(z)).collect());
    let folder = || clip_folder(&acc.loop_path).and_then(|f| names.folder(f));
    let menu = || {
        earliest(
            acc.banks
                .iter()
                .filter_map(|b| chron.menu(names, b))
                .collect(),
        )
    };
    let stage = || {
        earliest(
            acc.stage_zones
                .iter()
                .filter_map(|z| dated_zone(z))
                .collect(),
        )
    };
    let story = || {
        acc.story_plays
            .iter()
            .filter_map(|(group, &plays)| {
                let review = names.gd.story_reviews.get(group)?;
                let start =
                    chron
                        .mainline
                        .get(group)
                        .copied()
                        .unwrap_or(if review.start_time > 0 {
                            review.start_time
                        } else {
                            UNDATED
                        });
                let name = names
                    .activity(group)
                    .or_else(|| zone_name(names.gd, group))
                    .or_else(|| trimmed(&review.name))?;
                Some((std::cmp::Reverse(plays), start, name))
            })
            .min()
            .map(|(_, _, name)| name)
    };
    chapter()
        .or(claimed)
        .or_else(folder)
        .or_else(menu)
        .or_else(stage)
        .or_else(story)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_loop_keys_the_track_and_silence_is_not_one() {
        assert_eq!(
            loop_key(&norm(
                "Audio/Sound_Beta_2/Music/act27side/m_bat_act27side_1_loop"
            )),
            "m_bat_act27side_1"
        );
        assert_eq!(
            loop_key(&norm("Sound_Beta_2/Music/act27side/m_avg_SiestaCity")),
            "m_avg_siestacity"
        );
        let mut reg = Registry::default();
        assert!(
            reg.pair("Audio/Sound_Beta_2/Music/obt/empty", None)
                .is_none()
        );
        let key = reg
            .pair(
                "Audio/Sound_Beta_2/Music/obt/m_sys_ddd_climax",
                Some("Audio/Sound_Beta_2/Music/obt/empty"),
            )
            .unwrap();
        assert!(reg.tracks[&key].intros.is_empty());
    }

    #[test]
    fn the_folder_drops_bundle_revisions() {
        assert_eq!(
            clip_folder("sound_beta_2/music/act18d5d0/m_sys/m_sys_ccs5_loop"),
            Some("act18d5d0")
        );
        assert_eq!(clip_folder("sound_beta_2/music/m_loose_loop"), None);
    }

    #[test]
    fn play_music_reads_both_halves_through_the_variables() {
        let dir = std::env::temp_dir().join(format!("music_vars_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("gamedata/story")).unwrap();
        std::fs::write(
            dir.join("gamedata/story/story_variables.json.json"),
            r#"{"escape_intro": "Sound_Beta_2/Music/AVG/m_avg_escape_intro", "escape_loop": "Sound_Beta_2/Music/AVG/m_avg_escape_loop"}"#,
        )
        .unwrap();
        let vars = StoryVariables::load(&dir);
        let text = "[PlayMusic(intro=\"$escape_intro\", key=\"$escape_loop\", volume=0.8)]\n[playmusic(key=\"$missing\")]\n[PlayMusic(key=\"Sound_Beta_2/Music/act15side/m_sys_bitw_loop\")]";
        let found = script_music(text, &vars);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].0, "Sound_Beta_2/Music/AVG/m_avg_escape_loop");
        assert_eq!(
            found[0].1.as_deref(),
            Some("Sound_Beta_2/Music/AVG/m_avg_escape_intro")
        );
        assert_eq!(found[1].0, "Sound_Beta_2/Music/act15side/m_sys_bitw_loop");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn map_signals_and_roles() {
        let map_banks: HashSet<String> = ["sys.ON_ROGUELIKE_MAPBOSS.rogue_4".to_owned()].into();
        // A table-named signal is a map, the boss floor's included.
        assert_eq!(
            bank_kind("sys.ON_ROGUELIKE_MAPBOSS.rogue_4", &map_banks),
            Some(MusicKind::Map)
        );
        assert_eq!(
            bank_kind("sys.ON_ROGUELIKE_SHOP.rogue_1", &map_banks),
            Some(MusicKind::Lobby)
        );
        assert_eq!(
            bank_kind("sys.ON_SANDBOX_DUNGEONMAP.sandbox_1", &map_banks),
            Some(MusicKind::Map)
        );
        assert_eq!(
            bank_kind("battle.ON_CUSTOM_TRIGGER.rl5boss4_2", &map_banks),
            Some(MusicKind::Battle)
        );
        assert_eq!(roguelike_role("ON_ROGUELIKE_SHOP"), MusicRole::Shop);
        assert_eq!(
            roguelike_role("ON_ROGUELIKE_SECRET_EVENT"),
            MusicRole::Encounter
        );
    }

    #[test]
    fn a_level_event_is_read_without_a_parse() {
        let path = std::env::temp_dir().join(format!("music_level_{}.json", std::process::id()));
        std::fs::write(&path, "{\"BgmEvent\": \"calamity\", \"Enemies\": []}").unwrap();
        assert_eq!(read_bgm_event(&path).as_deref(), Some("calamity"));
        let _ = std::fs::remove_file(&path);
    }
}
