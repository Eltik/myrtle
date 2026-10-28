use std::collections::{BTreeMap, HashMap, HashSet};

use myrtle_gamedata_raw::tables::story::{AVG_CHARACTERS, StorySpriteFiles};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StorySprite {
    pub id: String,
    pub folder: String,
    pub body: String,
    pub face_center: Option<(f32, f32)>,
    pub speaker: Option<String>,
    pub aliases: Vec<String>
}

impl StorySprite {
    pub fn body_path(&self) -> String {
        format!("/textures/avg/characters/{}/{}.png", self.folder, self.body)
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct StorySprites {
    sprites: BTreeMap<String, StorySprite>,
}

impl StorySprites {
    pub fn get(&self, id: &str) -> Option<&StorySprite> {
        self.sprites.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &StorySprite> {
        self.sprites.values()
    }

    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty()
    }

    pub fn build(
        files: &StorySpriteFiles,
        scripts: &[String],
        folders: &HashMap<String, Vec<String>>,
    ) -> Self {
        let ids = character_ids(folders);
        let speakers = scan_speakers(scripts, &ids);
        let mut sprites = BTreeMap::new();

        for (base, sets) in sets_by_character(folders, &ids) {
            let found = sets.iter().find_map(|&(_, folder)| {
                let stems = folders.get(folder)?;
                default_body(folder, stems)
                    .or_else(|| hub_body(files.hubs.get(folder), stems))
                    .map(|body| (folder, body))
            });

            let Some((folder, body)) = found else {
                continue;
            };

            let face = face_center(
                files.hubs.get(folder),
                files
                    .png_heads
                    .get(&format!("{AVG_CHARACTERS}{folder}/{body}.png"))
                    .map(Vec::as_slice),
            );
            sprites.insert(
                base.clone(),
                StorySprite {
                    speaker: speakers.get(&base).map(|n| n.speaker.clone()),
                    aliases: speakers
                        .get(&base)
                        .map(|n| n.aliases.clone())
                        .unwrap_or_default(),
                    id: base,
                    folder: folder.to_owned(),
                    body,
                    face_center: face,
                },
            );
        }
        Self { sprites }
    }
}

fn character_ids(folders: &HashMap<String, Vec<String>>) -> HashMap<String, String> {
    let mut by_head: HashMap<&str, Vec<(u32, &str, String)>> = HashMap::new();
    let mut ids = HashMap::new();
    let lowered: Vec<(String, &String)> = folders
        .keys()
        .map(|f| (f.to_ascii_lowercase(), f))
        .collect();

    for (lower, folder) in &lowered {
        let (head, n) = split_set_number(lower);
        match n {
            Some(n) if !head.bytes().any(|b| b.is_ascii_digit()) => by_head
                .entry(head)
                .or_default()
                .push((n, folder.as_str(), lower.clone())),
            _ => {
                ids.insert(lower.clone(), head.to_owned());
            }
        }
    }

    let taken: HashSet<String> = ids.values().cloned().collect();
    for (head, mut members) in by_head {
        if members.len() == 1 {
            ids.insert(members.remove(0).2, head.to_owned());
            continue;
        }
        members.sort_unstable();
        let kept = members
            .iter()
            .find(|(_, folder, _)| {
                folders
                    .get(*folder)
                    .is_some_and(|stems| default_body(folder, stems).is_some())
            })
            .map(|(_, _, lower)| lower.clone());
        for (_, _, lower) in members {
            let id = if kept.as_ref() == Some(&lower) {
                head.to_owned()
            } else if taken.contains(&lower) {
                format!("{lower}_0")
            } else {
                lower.clone()
            };
            ids.insert(lower, id);
        }
    }
    ids
}

fn sets_by_character<'a>(
    folders: &'a HashMap<String, Vec<String>>,
    ids: &HashMap<String, String>,
) -> BTreeMap<String, Vec<(u32, &'a str)>> {
    let mut by_base: BTreeMap<String, Vec<(u32, &str)>> = BTreeMap::new();
    for folder in folders.keys() {
        let lower = folder.to_ascii_lowercase();
        let Some(id) = ids.get(&lower) else {
            continue;
        };
        let n = if *id == lower {
            0
        } else {
            split_set_number(&lower).1.unwrap_or(0)
        };
        by_base
            .entry(id.clone())
            .or_default()
            .push((n, folder.as_str()));
    }
    for sets in by_base.values_mut() {
        sets.sort_unstable();
    }
    by_base
}

fn split_set_number(folder: &str) -> (&str, Option<u32>) {
    if let Some((head, tail)) = folder.rsplit_once('_')
        && !head.is_empty()
        && !tail.is_empty()
        && tail.bytes().all(|b| b.is_ascii_digit())
    {
        return (head, tail.parse().ok());
    }
    (folder, None)
}

fn default_body(folder: &str, stems: &[String]) -> Option<String> {
    let f = folder.to_ascii_lowercase();
    let (s, _) = split_set_number(&f);
    let candidates = [
        format!("{s}_1"),
        f.clone(),
        format!("{f}_1"),
        format!("{f}$1"),
        format!("{f}#1"),
        format!("{f}_1#1"),
    ];
    candidates.iter().find_map(|want| {
        stems
            .iter()
            .find(|stem| stem.eq_ignore_ascii_case(want))
            .cloned()
    })
}

fn hub_body(hub: Option<&Value>, stems: &[String]) -> Option<String> {
    let name = hub?
        .get("groups")?
        .as_array()?
        .first()?
        .get("sprites")?
        .as_array()?
        .first()?
        .get("name")?
        .as_str()?;
    stems.iter().find(|stem| stem.as_str() == name).cloned()
}

pub fn face_center(hub: Option<&Value>, body_head: Option<&[u8]>) -> Option<(f32, f32)> {
    let group = hub?.get("groups")?.as_array()?.first()?;
    let num = |o: &Value, k: &str| o.get(k).and_then(Value::as_f64);
    let pos = group.get("facePos")?;
    let size = group.get("faceSize")?;
    let (x, y) = (num(pos, "x")?, num(pos, "y")?);
    let (w, h) = (num(size, "w")?, num(size, "h")?);
    if x < 0.0 || y < 0.0 || w <= 0.0 || h <= 0.0 {
        return None;
    }
    let (pw, ph) = png_size(body_head?)?;
    let (pw, ph) = (f64::from(pw), f64::from(ph));
    let cx = (x + w / 2.0) / pw;
    let cy = (y + h / 2.0) / ph;
    // A face outside its own plate is a hub this reading does not understand,
    // not a face to clamp onto the edge.
    if !(0.0..=1.0).contains(&cx) || !(0.0..=1.0).contains(&cy) {
        return None;
    }
    Some((cx as f32, cy as f32))
}

fn png_size(head: &[u8]) -> Option<(u32, u32)> {
    let head: &[u8; 24] = head.get(..24)?.try_into().ok()?;
    if &head[..8] != b"\x89PNG\r\n\x1a\n" || &head[12..16] != b"IHDR" {
        return None;
    }
    let w = u32::from_be_bytes([head[16], head[17], head[18], head[19]]);
    let h = u32::from_be_bytes([head[20], head[21], head[22], head[23]]);
    (w > 0 && h > 0).then_some((w, h))
}

const MIN_VOTES: u32 = 2;
const ALIAS_MIN_VOTES: u32 = 5;
const ALIAS_MIN_SHARE_DENOM: u32 = 10;
const MAX_ALIASES: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Names {
    speaker: String,
    aliases: Vec<String>,
}

fn pick_names(names: HashMap<String, u32>) -> Option<Names> {
    let total: u32 = names.values().sum();
    let mut ranked: Vec<(String, u32)> = names.into_iter().collect();
    ranked.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut ranked = ranked.into_iter();

    let (speaker, count) = ranked.next()?;
    if count < MIN_VOTES || count * 2 < total {
        return None;
    }

    let floor = |votes: u32| votes >= ALIAS_MIN_VOTES && votes * ALIAS_MIN_SHARE_DENOM >= count;

    let mut seen = vec![alias_key(&speaker)];
    let mut aliases = Vec::new();
    for (name, votes) in ranked {
        if aliases.len() == MAX_ALIASES || !floor(votes) {
            break;
        }
        let key = alias_key(&name);
        if key.is_empty() || seen.contains(&key) {
            continue;
        }
        seen.push(key);
        aliases.push(name);
    }

    Some(Names { speaker, aliases })
}

fn alias_key(name: &str) -> String {
    let key = name
        .trim_matches(|c: char| {
            c.is_whitespace() || matches!(c, '?' | '\'' | '"' | '\u{2018}' | '\u{2019}')
        })
        .to_lowercase();
    if key.chars().any(char::is_alphanumeric) {
        key
    } else {
        String::new()
    }
}

fn scan_speakers(scripts: &[String], ids: &HashMap<String, String>) -> HashMap<String, Names> {
    let mut votes: HashMap<String, HashMap<String, u32>> = HashMap::new();
    for text in scripts {
        vote_script(text, ids, &mut votes);
    }
    votes
        .into_iter()
        .filter_map(|(base, names)| pick_names(names).map(|n| (base, n)))
        .collect()
}

#[derive(Default)]
struct Stage {
    slots: Vec<(String, String)>,
    focus: Option<String>,
}

impl Stage {
    fn speaker_sprite(&self) -> Option<&str> {
        let sprite = match self.focus.as_deref() {
            Some(slot) => self.slots.iter().find(|(s, _)| s == slot),
            None if self.slots.len() == 1 => self.slots.first(),
            None => None,
        };
        sprite.map(|(_, name)| name.as_str())
    }

    fn character(&mut self, a: &[(String, String)]) {
        self.slots.clear();
        for key in ["name", "name2"] {
            if let Some(v) = arg(a, key) && !v.is_empty() {
                self.slots.push((key.to_owned(), v.to_owned()));
            }
        }
        self.focus = match arg(a, "focus") {
            Some("1") => Some("name".to_owned()),
            Some("2") => Some("name2".to_owned()),
            _ => None,
        };
    }

    fn charslot(&mut self, a: &[(String, String)]) {
        if a.is_empty() {
            self.slots.clear();
            self.focus = None;
            return;
        }
        let slot = arg(a, "slot").map_or_else(|| "m".to_owned(), str::to_ascii_lowercase);
        let name = arg(a, "name");
        if let Some(name) = name {
            self.slots.retain(|(s, _)| *s != slot);
            self.slots.push((slot, name.to_owned()));
        }
        match arg(a, "focus") {
            Some(f) => {
                let f = f.to_ascii_lowercase();
                self.focus = self.slots.iter().any(|(s, _)| *s == f).then_some(f);
            }
            None if name.is_some() => self.focus = None,
            None => {}
        }
    }
}

fn vote_script(
    text: &str,
    ids: &HashMap<String, String>,
    votes: &mut HashMap<String, HashMap<String, u32>>,
) {
    let mut stage = Stage::default();
    for line in text.lines() {
        let line = line.trim_start_matches('\u{feff}').trim();
        if !line.starts_with('[') {
            continue;
        }
        let Some(close) = line.find(']') else {
            continue;
        };
        let head = &line[1..close];
        let lower = head.to_ascii_lowercase();
        if lower.starts_with("name") && !lower.starts_with("name2") {
            let spoken = line[close + 1..].trim();
            let a = args(head);
            let Some(speaker) = arg(&a, "name").map(str::trim) else {
                continue;
            };
            if speaker.is_empty() || spoken.is_empty() {
                continue;
            }
            let Some(base) = stage
                .speaker_sprite()
                .and_then(|sprite| sprite_base(sprite, ids))
            else {
                continue;
            };
            *votes
                .entry(base)
                .or_default()
                .entry(speaker.to_owned())
                .or_default() += 1;
        } else if lower == "character" || lower.starts_with("character(") {
            stage.character(&args(head));
        } else if lower == "charslot" || lower.starts_with("charslot(") {
            stage.charslot(&args(head));
        }
    }
}

fn sprite_base(sprite: &str, ids: &HashMap<String, String>) -> Option<String> {
    let raw = sprite
        .split(['#', '$'])
        .next()
        .unwrap_or(sprite)
        .trim()
        .to_ascii_lowercase();
    ids.get(&raw)
        .or_else(|| ids.get(&format!("{raw}_1")))
        .cloned()
}

fn arg<'a>(a: &'a [(String, String)], key: &str) -> Option<&'a str> {
    a.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

fn args(head: &str) -> Vec<(String, String)> {
    let body = match head.find('(') {
        Some(open) => &head[open + 1..head.rfind(')').filter(|&c| c > open).unwrap_or(head.len())],
        None => head,
    };
    let bytes = body.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && (bytes[i].is_ascii_whitespace() || bytes[i] == b',') {
            i += 1;
        }
        let key_start = i;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
            i += 1;
        }
        let key = body[key_start..i].to_ascii_lowercase();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if key.is_empty() || i >= bytes.len() || bytes[i] != b'=' {
            while i < bytes.len() && bytes[i] != b',' {
                i += 1;
            }
            continue;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let value = if i < bytes.len() && bytes[i] == b'"' {
            let start = i + 1;
            let end = body[start..].find('"').map_or(body.len(), |e| start + e);
            i = (end + 1).min(body.len());
            body[start..end].to_owned()
        } else {
            let start = i;
            while i < bytes.len() && bytes[i] != b',' {
                i += 1;
            }
            body[start..i].trim().to_owned()
        };
        out.push((key, value));
    }
    out
}
