use std::collections::{BTreeSet, HashMap, HashSet};

use myrtle_gamedata::model::GameModel;

use crate::dto::{
    MusicIndex, MusicKind, MusicPlace, MusicPlaceItem, MusicPlaceKind, MusicRole, MusicTotals,
    MusicTrack,
};
use crate::sources::{MusicSources, ScriptPlay, StoryScript, clip_index, level_key};
use crate::tables::{
    KeyedOrMap, MainlineAddition, MusicTables, RoguelikeDetail, SandboxDetail, StorySetMusic,
};

const SILENCE: &str = "empty";
const UNDATED: i64 = i64::MAX;

fn norm(path: &str) -> String {
    let p = path.trim().replace('\\', "/").to_ascii_lowercase();
    let p = p.trim_start_matches('/');
    p.strip_prefix("audio/").unwrap_or(p).to_owned()
}

fn stem(path: &str) -> &str {
    let last = path.rsplit('/').next().unwrap_or(path);
    last.strip_suffix(".ogg").unwrap_or(last)
}

fn loop_key(path: &str) -> String {
    let s = stem(path);
    s.strip_suffix("_loop").unwrap_or(s).to_owned()
}

fn intro_base(path: &str) -> &str {
    let s = stem(path);
    s.strip_suffix("_intro").unwrap_or(s)
}

fn clip_folder(path: &str) -> Option<&str> {
    let mut parts = path.split('/');
    parts.by_ref().find(|p| *p == "music")?;
    let folder = parts.next()?;
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

impl PlaceAcc {
    fn item(&mut self, id: Option<String>, code: Option<String>, name: Option<String>) {
        if self.seen.insert((id.clone(), code.clone(), name.clone())) {
            self.items.push(MusicPlaceItem { id, code, name });
        }
    }
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
    chapters: BTreeSet<String>,
    stage_zones: BTreeSet<String>,
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

fn is_sandbox_map_signal(event: &str) -> bool {
    event
        .strip_prefix("ON_SANDBOX_")
        .is_some_and(|rest| rest.contains("MAP"))
}

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
    bank_track: HashMap<String, String>,
    alias: HashMap<String, String>,
}

impl Registry {
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

    fn pair_mut(&mut self, play: &ScriptPlay) -> Option<&mut TrackAcc> {
        let key = self.pair(&play.loop_path, play.intro.as_deref())?;
        self.tracks.get_mut(&key)
    }

    fn bank(&self, name: &str) -> Option<&String> {
        self.own_bank(name)
            .or_else(|| self.bank_track.get(self.alias.get(name)?))
    }

    fn own_bank(&self, name: &str) -> Option<&String> {
        self.bank_track.get(name)
    }

    fn track_mut(&mut self, name: &str) -> Option<&mut TrackAcc> {
        let key = self.bank(name)?.clone();
        self.tracks.get_mut(&key)
    }
}

#[derive(Default)]
struct Origins(HashMap<String, String>);

impl Origins {
    fn claim(&mut self, key: Option<&String>, name: Option<String>) {
        if let (Some(key), Some(name)) = (key, name) {
            self.0.entry(key.clone()).or_insert(name);
        }
    }
}

struct MusicRows<'a>(HashMap<&'a str, &'a str>);

impl MusicRows<'_> {
    fn track(&self, reg: &Registry, id: &str) -> Option<String> {
        reg.bank(self.0.get(id)?).cloned()
    }
}

fn battle_bank(event: &str) -> String {
    format!("battle.ON_GAME_READY.{event}")
}

fn sys_parts(bank: &str) -> Option<(&str, Option<&str>)> {
    let mut parts = bank.split('.');
    if parts.next()? != "sys" {
        return None;
    }
    let event = parts.next()?;
    Some((event, parts.next()))
}

struct Names<'a> {
    gd: &'a GameModel,
    seasons: HashMap<String, String>,
    sandboxes: HashMap<String, String>,
}

impl Names<'_> {
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

    fn sandbox(&self, id: &str) -> Option<String> {
        self.sandboxes.get(id).cloned()
    }

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

    fn archive_owner(&self, id: &str) -> Option<String> {
        self.activity(id)
            .or_else(|| self.theme(id))
            .or_else(|| self.sandbox(id))
    }

    fn folder(&self, folder: &str) -> Option<String> {
        if !folder.starts_with("act") {
            return None;
        }
        let mut candidate = folder;
        loop {
            if let Some(name) = self.activity(candidate) {
                return Some(name);
            }
            candidate = candidate.strip_suffix("d0")?;
        }
    }
}

struct Chronology {
    original: HashMap<String, String>,
    seasons: HashMap<String, i64>,
    themes: HashMap<String, i64>,
    mainline: HashMap<String, i64>,
}

impl Chronology {
    fn activity(&self, names: &Names<'_>, id: &str) -> Option<(i64, String)> {
        let id = self.original.get(id).map_or(id, String::as_str);
        let start = names.gd.activities.get(id)?.start_time;
        Some((start, names.activity(id)?))
    }

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
            e if e.starts_with("ON_SANDBOX_") => Some((UNDATED, names.sandbox_entity(entity)?)),
            _ => None,
        }
    }
}

fn sys_place(
    names: &Names<'_>,
    bank: &str,
    map_banks: &HashSet<String>,
) -> Option<(MusicPlaceKind, Option<String>, Option<MusicRole>)> {
    let (event, entity) = sys_parts(bank)?;
    let place = match event {
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
    (place.1.is_some() || place.0 == MusicPlaceKind::Lobby).then_some(place)
}

struct SeasonRow {
    id: String,
    name: String,
    start: i64,
    main: Option<String>,
    challenge: Option<String>,
}

fn season_rows(tables: &MusicTables) -> Vec<SeasonRow> {
    let mut v2 = tables.crisis_v2.season_info_data_map.values();
    v2.sort_by(|a, b| a.season_id.cmp(&b.season_id));
    let v1 = tables.crisis.season_info.iter().map(|s| SeasonRow {
        id: s.season_id.clone(),
        name: s.name.clone(),
        start: s.start_ts,
        main: s.perm_bgm.clone(),
        challenge: s.perm_bgm_hard.clone(),
    });
    let v2 = v2.into_iter().map(|s| SeasonRow {
        id: s.season_id.clone(),
        name: s.name.clone(),
        start: s.start_ts,
        main: s.season_bgm.clone(),
        challenge: s.season_bgm_challenge.clone(),
    });
    v1.chain(v2).collect()
}

fn rerun_originals(gd: &GameModel) -> HashMap<String, String> {
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

fn zone_name(gd: &GameModel, zone: &str) -> Option<String> {
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

#[must_use]
pub fn build_music_index(gd: &GameModel, src: &MusicSources) -> MusicIndex {
    let tables = &src.tables;
    let seasons = season_rows(tables);
    let mut season_names = HashMap::new();
    let mut season_starts = HashMap::new();
    for s in &seasons {
        season_starts.insert(s.id.clone(), s.start);
        if let Some(n) = trimmed(&s.name) {
            season_names.insert(s.id.clone(), n);
        }
    }
    let sandboxes = tables
        .sandbox
        .basic_info
        .values()
        .into_iter()
        .filter_map(|t| Some((t.topic_id.clone(), trimmed(&t.topic_name)?)))
        .collect();
    let names = Names {
        gd,
        seasons: season_names,
        sandboxes,
    };
    let rogue: Vec<(String, &RoguelikeDetail)> = tables.rogue.details.pairs();
    let theme_starts: HashMap<String, i64> = tables
        .rogue
        .topics
        .pairs()
        .into_iter()
        .map(|(id, t)| (id, t.start_time))
        .collect();
    let map_banks = rogue_map_banks(&rogue);

    let (mut reg, named_banks) = register_banks(gd);
    let mut origins = Origins::default(); // claims land in call order, the first wins
    place_banks(&mut reg, &names, &named_banks, &map_banks);

    let rows = MusicRows(
        gd.music
            .musics()
            .map(|m| (m.id.as_str(), m.bank.as_str()))
            .collect(),
    );

    let mut chron = Chronology {
        original: rerun_originals(gd),
        seasons: season_starts,
        themes: theme_starts,
        mainline: tables
            .zones
            .mainline_zone_id_list
            .iter()
            .zip(0_i64..)
            .map(|(zone, at)| {
                let open = gd.zone_open_times.get(zone).copied();
                (zone.clone(), open.unwrap_or(i64::MIN / 2 + at))
            })
            .collect(),
    };
    let story_sets: Vec<(String, &StorySetMusic)> = tables.story_sets.storyline_story_sets.pairs();
    let chapter_zone = chapter_zones(
        gd,
        &tables.zones.mainline_addition_info,
        &story_sets,
        &mut chron.mainline,
    );
    place_titles_and_chapters(gd, &mut reg, &chapter_zone);
    place_lobby(tables, &mut reg, &rows, &named_banks);
    claim_archives(gd, &reg, &names, &rows, &mut origins);

    place_sandbox_runs(
        &mut reg,
        &mut origins,
        &names,
        &rows,
        &tables.sandbox.detail,
    );
    place_seasons(&mut reg, &mut origins, &names, seasons);
    place_rogue(&mut reg, &mut origins, &names, &rogue);
    claim_story_sets(&reg, &mut origins, &names, &rows, &story_sets);
    place_activity_music(tables, &mut reg, &mut origins, &names, &rows, &named_banks);

    place_stages(gd, src, &mut reg);

    let scripts: HashMap<&str, &StoryScript> =
        src.scripts.iter().map(|s| (s.path.as_str(), s)).collect();
    let listed = place_listed_stories(gd, &scripts, &mut reg);
    place_unlisted_stories(&src.scripts, &listed, &mut reg, &names);

    assemble(reg, origins, &names, &chron, src)
}

fn rogue_map_banks(rogue: &[(String, &RoguelikeDetail)]) -> HashSet<String> {
    let mut out = HashSet::new();
    for (theme, d) in rogue {
        let signals = d
            .zones
            .values()
            .into_iter()
            .flat_map(|z| [z.bgm_signal.as_ref(), z.bgm_signal_with_low_san.as_ref()])
            .chain(
                d.variation_data
                    .values()
                    .into_iter()
                    .map(|v| v.sound.as_ref()),
            )
            .flatten();
        for sig in signals {
            out.insert(format!("sys.{sig}.{theme}"));
        }
    }
    out
}

fn register_banks(gd: &GameModel) -> (Registry, Vec<String>) {
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

fn chapter_zones(
    gd: &GameModel,
    additions: &KeyedOrMap<MainlineAddition>,
    story_sets: &[(String, &StorySetMusic)],
    mainline: &mut HashMap<String, i64>,
) -> HashMap<String, String> {
    let mut chapter_zone: HashMap<String, String> = HashMap::new();
    for info in additions.values() {
        if !info.zone_id.is_empty() {
            let open = gd.zone_open_times.get(&info.zone_id).copied();
            mainline
                .entry(info.zone_id.clone())
                .or_insert_with(|| open.unwrap_or(UNDATED));
        }
        if !info.game_music_id.is_empty() && !info.zone_id.is_empty() {
            chapter_zone
                .entry(info.game_music_id.clone())
                .or_insert_with(|| info.zone_id.clone());
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

fn place_titles_and_chapters(
    gd: &GameModel,
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

fn place_lobby(
    tables: &MusicTables,
    reg: &mut Registry,
    rows: &MusicRows<'_>,
    named_banks: &[String],
) {
    for bg in &tables.display.home_background_data.home_bg_data_list {
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
    let mut jukebox = tables.building.music_data.music_datas.values();
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
    for (_, furniture) in tables.building.custom_data.furnitures.pairs() {
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

fn claim_archives(
    gd: &GameModel,
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

fn place_rogue(
    reg: &mut Registry,
    origins: &mut Origins,
    names: &Names<'_>,
    rogue: &[(String, &RoguelikeDetail)],
) {
    for (theme, detail) in rogue {
        let name = names.theme(theme);
        let mut spots: Vec<(&String, MusicRole, Option<String>)> = Vec::new();
        for zone in detail.zones.values() {
            for sig in [&zone.bgm_signal, &zone.bgm_signal_with_low_san]
                .into_iter()
                .flatten()
            {
                spots.push((sig, MusicRole::Map, trimmed(&zone.name)));
            }
        }
        for variation in detail.variation_data.values() {
            if let Some(sig) = &variation.sound {
                spots.push((sig, MusicRole::Map, trimmed(&variation.outer_name)));
            }
        }
        if let Some(sig) = &detail.game_const.sp_zone_shop_bgm_signal {
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

fn claim_story_sets(
    reg: &Registry,
    origins: &mut Origins,
    names: &Names<'_>,
    rows: &MusicRows<'_>,
    story_sets: &[(String, &StorySetMusic)],
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

fn place_activity_music(
    tables: &MusicTables,
    reg: &mut Registry,
    origins: &mut Origins,
    names: &Names<'_>,
    rows: &MusicRows<'_>,
    named_banks: &[String],
) {
    let gd = names.gd;
    let mut act_rows: Vec<_> = tables
        .activity
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

fn place_stages(gd: &GameModel, src: &MusicSources, reg: &mut Registry) {
    for entry in &gd.stage_index {
        let Some(level) = entry.level_id.as_deref() else {
            continue;
        };
        let Some(event) = src.level_bgm.get(&level_key(level)) else {
            continue;
        };
        let bank = battle_bank(event);
        let Some(key) = reg.bank(&bank).or_else(|| reg.bank(event)).cloned() else {
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

fn listed_script<'a>(
    scripts: &HashMap<&str, &'a StoryScript>,
    story_txt: &str,
) -> Option<&'a StoryScript> {
    [
        format!("{story_txt}.txt.txt"),
        format!("[uc]info/{story_txt}.txt.txt"),
    ]
    .iter()
    .filter_map(|p| scripts.get(p.as_str()).copied())
    .find(|s| s.is_script)
}

fn place_listed_stories(
    gd: &GameModel,
    scripts: &HashMap<&str, &StoryScript>,
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
            let Some(script) = listed_script(scripts, &story.story_txt) else {
                continue;
            };
            for play in &script.plays {
                let Some(acc) = reg.pair_mut(play) else {
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

fn place_unlisted_stories(
    scripts: &[StoryScript],
    listed: &HashSet<String>,
    reg: &mut Registry,
    names: &Names<'_>,
) {
    let mut unlisted: Vec<&StoryScript> = scripts
        .iter()
        .filter(|s| s.path.ends_with(".txt.txt"))
        .collect();
    unlisted.sort_by(|a, b| a.path.split('/').cmp(b.path.split('/')));
    for script in unlisted {
        let rel = script.path.to_ascii_lowercase();
        if rel.starts_with("[uc]info/") {
            continue;
        }
        let txt = rel.trim_end_matches(".txt.txt");
        if listed.contains(txt) || !script.utf8 || script.plays.is_empty() {
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
        for play in &script.plays {
            if let Some(acc) = reg.pair_mut(play) {
                acc.place(MusicPlaceKind::Story, group.clone(), None).item(
                    None,
                    None,
                    file.clone(),
                );
            }
        }
    }
}

fn assemble(
    mut reg: Registry,
    mut origins: Origins,
    names: &Names<'_>,
    chron: &Chronology,
    src: &MusicSources,
) -> MusicIndex {
    let trees: Vec<(Option<String>, HashMap<String, String>)> = src
        .clip_trees
        .iter()
        .map(|t| (t.server.clone(), clip_index(&t.paths)))
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
