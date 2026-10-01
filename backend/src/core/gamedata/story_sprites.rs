//! Story character sprites as rankable entries: one per character, not one per
//! face or outfit.
//!
//! The story stage draws characters from `textures/avg/characters/<folder>/`
//! (1,636 folders on EN 2026-10-01). A folder is one sprite set, and a
//! character with several sets numbers them (`avg_npc_935_1`, `_2`), so the
//! entry is the folder with its trailing `_N` cut: 1,446 characters on EN.
//! The entry's art is the set's default body, picked by the same rule the
//! story reader uses for a sprite named with no face or body index
//! (`core::story::assets`, rule 3), from the lowest-numbered set that has
//! one. Bodies carry their default face, so the whole plate reads on its own.
//!
//! Where the face sits comes from the set's `hub.json` (`facePos` and
//! `faceSize` of the first group, in body pixels from the top-left), so a
//! tile can crop to the head; a legacy hub's `(-1, -1)` sentinel, or a zero
//! size, leaves it unknown. The plate those pixels count in is the body PNG
//! itself, read off its header: the hub's `sprites[].size` is the FACE patch
//! of a face-swap set (256 x 256 on Kal'tsit's 1280 x 1280 body), and
//! dividing by it put every one of the 756 face-carrying hubs on EN past the
//! plate's bottom-right corner (2026-10-01).
//!
//! The game gives a sprite no display name. The name here is read off the
//! scripts: every `[name="..."]` line spoken while exactly one sprite is on
//! stage, or while a `focus` picks one, votes for that sprite's character,
//! and the winner is kept when it holds at least half the votes and two of
//! them. On EN that names 1,216 of the 1,446.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

/// One character's sprite entry.
#[derive(Debug, Clone, PartialEq)]
pub struct StorySprite {
    /// The folder with its trailing `_N` cut, lowercased: `avg_npc_935`.
    pub id: String,
    /// The folder the art came from, as spelled on disk.
    pub folder: String,
    /// The body PNG's stem, as spelled on disk.
    pub body: String,
    /// The face patch's centre as a fraction of the body plate, `(x, y)`.
    pub face_center: Option<(f32, f32)>,
    /// The speaker the scripts name this character, when one wins the vote.
    pub speaker: Option<String>,
}

impl StorySprite {
    /// The body's path relative to the assets root, as on disk: a caller
    /// building a URL encodes the `#` and `$` some names carry.
    pub fn body_path(&self) -> String {
        format!("/textures/avg/characters/{}/{}.png", self.folder, self.body)
    }
}

#[derive(Debug, Clone, Default)]
pub struct StorySprites {
    sprites: BTreeMap<String, StorySprite>,
}

impl StorySprites {
    pub fn get(&self, id: &str) -> Option<&StorySprite> {
        self.sprites.get(id)
    }

    /// Every entry, by id.
    pub fn iter(&self) -> impl Iterator<Item = &StorySprite> {
        self.sprites.values()
    }

    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty()
    }

    /// `folders` maps each folder under `textures/avg/characters`, as spelled
    /// on disk, to the stems of its PNGs (alpha companions included; they are
    /// never picked). `assets_dir` is the server's output root, read for each
    /// folder's `hub.json` and for the scripts under `gamedata/story`.
    pub fn build(assets_dir: &Path, folders: &HashMap<String, Vec<String>>) -> Self {
        let chars_dir = assets_dir.join("textures/avg/characters");
        let lower_folders: HashSet<String> =
            folders.keys().map(|f| f.to_ascii_lowercase()).collect();

        let speakers = scan_speakers(&assets_dir.join("gamedata/story"), &lower_folders);

        let mut sprites = BTreeMap::new();
        for (base, sets) in sets_by_character(folders) {
            // The lowest-numbered set that has a default body.
            let found = sets.iter().find_map(|&(_, folder)| {
                default_body(folder, folders.get(folder)?).map(|body| (folder, body))
            });
            let Some((folder, body)) = found else {
                continue;
            };
            let set_dir = chars_dir.join(folder);
            let face = face_center(
                &set_dir.join("hub.json"),
                &set_dir.join(format!("{body}.png")),
            );
            sprites.insert(
                base.clone(),
                StorySprite {
                    speaker: speakers.get(&base).cloned(),
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

/// Character id -> its sets' folders, lowest set number first.
fn sets_by_character(folders: &HashMap<String, Vec<String>>) -> BTreeMap<String, Vec<(u32, &str)>> {
    let mut by_base: BTreeMap<String, Vec<(u32, &str)>> = BTreeMap::new();
    for folder in folders.keys() {
        let lower = folder.to_ascii_lowercase();
        let (base, n) = split_set_number(&lower);
        by_base
            .entry(base.to_owned())
            .or_default()
            .push((n.unwrap_or(0), folder.as_str()));
    }
    for sets in by_base.values_mut() {
        sets.sort_unstable();
    }
    by_base
}

/// `avg_npc_935_1` -> (`avg_npc_935`, Some(1)); a folder with no trailing
/// number is its own base.
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

/// The set's default body: the story reader's rule 3 order.
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

/// The first hub group's face centre as fractions of the body plate, `(x, y)`.
/// The plate is the body PNG's own size, see the module doc.
pub fn face_center(hub: &Path, body_png: &Path) -> Option<(f32, f32)> {
    let text = std::fs::read_to_string(hub).ok()?;
    let hub: serde_json::Value = serde_json::from_str(&text).ok()?;
    let group = hub.get("groups")?.as_array()?.first()?;
    let num = |o: &serde_json::Value, k: &str| o.get(k).and_then(serde_json::Value::as_f64);
    let pos = group.get("facePos")?;
    let size = group.get("faceSize")?;
    let (x, y) = (num(pos, "x")?, num(pos, "y")?);
    let (w, h) = (num(size, "w")?, num(size, "h")?);
    if x < 0.0 || y < 0.0 || w <= 0.0 || h <= 0.0 {
        return None;
    }
    let (pw, ph) = png_size(body_png)?;
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

/// A PNG's `(width, height)` from its header, without decoding the image.
fn png_size(path: &Path) -> Option<(u32, u32)> {
    use std::io::Read;
    let mut head = [0u8; 24];
    std::fs::File::open(path).ok()?.read_exact(&mut head).ok()?;
    if &head[..8] != b"\x89PNG\r\n\x1a\n" || &head[12..16] != b"IHDR" {
        return None;
    }
    let w = u32::from_be_bytes([head[16], head[17], head[18], head[19]]);
    let h = u32::from_be_bytes([head[20], head[21], head[22], head[23]]);
    (w > 0 && h > 0).then_some((w, h))
}

/// A winning name needs at least this many votes, and at least half of all
/// the votes its character got: one stray line does not name anyone.
const MIN_VOTES: u32 = 2;

/// Character id -> the speaker name the scripts give it, by vote.
fn scan_speakers(story_dir: &Path, lower_folders: &HashSet<String>) -> HashMap<String, String> {
    let mut votes: HashMap<String, HashMap<String, u32>> = HashMap::new();
    for entry in walkdir::WalkDir::new(story_dir).into_iter().flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "txt") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        vote_script(&text, lower_folders, &mut votes);
    }
    votes
        .into_iter()
        .filter_map(|(base, names)| {
            let total: u32 = names.values().sum();
            let (name, count) = names
                .into_iter()
                .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(&a.0)))?;
            (count >= MIN_VOTES && count * 2 >= total).then_some((base, name))
        })
        .collect()
}

/// Who is on stage as a script runs.
#[derive(Default)]
struct Stage {
    /// Slot -> sprite name. `[Character]` fills `name`/`name2`; `[charslot]`
    /// fills `l`, `m`, `r`.
    slots: Vec<(String, String)>,
    focus: Option<String>,
}

impl Stage {
    /// The sprite speaking a named line: the focused one, else the only one
    /// on stage.
    fn speaker_sprite(&self) -> Option<&str> {
        let sprite = match self.focus.as_deref() {
            Some(slot) => self.slots.iter().find(|(s, _)| s == slot),
            None if self.slots.len() == 1 => self.slots.first(),
            None => None,
        };
        sprite.map(|(_, name)| name.as_str())
    }

    /// `[Character(name=, name2=, focus=)]`: a whole new stage.
    fn character(&mut self, a: &[(String, String)]) {
        self.slots.clear();
        for key in ["name", "name2"] {
            if let Some(v) = arg(a, key)
                && !v.is_empty()
            {
                self.slots.push((key.to_owned(), v.to_owned()));
            }
        }
        self.focus = match arg(a, "focus") {
            Some("1") => Some("name".to_owned()),
            Some("2") => Some("name2".to_owned()),
            _ => None,
        };
    }

    /// `[charslot(slot=, name=, focus=)]`: one slot changes; a bare
    /// `[charslot]` empties the stage.
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

/// One script's votes: who is on stage when each named line is spoken.
fn vote_script(
    text: &str,
    lower_folders: &HashSet<String>,
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
                .and_then(|sprite| sprite_base(sprite, lower_folders))
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

/// A script sprite name (`avg_npc_935_1#2$1`) -> its character id, when the
/// extract has the folder it names.
fn sprite_base(sprite: &str, lower_folders: &HashSet<String>) -> Option<String> {
    let raw = sprite
        .split(['#', '$'])
        .next()
        .unwrap_or(sprite)
        .trim()
        .to_ascii_lowercase();
    let folder = if lower_folders.contains(&raw) {
        raw
    } else {
        let with_set = format!("{raw}_1");
        if !lower_folders.contains(&with_set) {
            return None;
        }
        with_set
    };
    Some(split_set_number(&folder).0.to_owned())
}

/// The first value of `key` among a command's [`args`].
fn arg<'a>(a: &'a [(String, String)], key: &str) -> Option<&'a str> {
    a.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

/// `key="value"` and `key=value` pairs of a command head (`Character(name="a",
/// focus=1)`), keys lowercased.
fn args(head: &str) -> Vec<(String, String)> {
    let body = match head.find('(') {
        Some(open) => &head[open + 1..head.rfind(')').filter(|&c| c > open).unwrap_or(head.len())],
        // `[name="Amiya"]`: the head itself is one pair.
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
            // Not a pair: skip to the next separator.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn folders(names: &[&str]) -> HashSet<String> {
        names.iter().map(|n| n.to_ascii_lowercase()).collect()
    }

    #[test]
    fn args_read_quoted_and_bare_values() {
        let a = args(r#"Character(name="char_003_kalts_1", name2="avg_npc_1#2", focus=2)"#);
        assert_eq!(
            a,
            vec![
                ("name".to_owned(), "char_003_kalts_1".to_owned()),
                ("name2".to_owned(), "avg_npc_1#2".to_owned()),
                ("focus".to_owned(), "2".to_owned()),
            ]
        );
        assert_eq!(
            args(r#"name="Kal'tsit""#),
            vec![("name".to_owned(), "Kal'tsit".to_owned())]
        );
    }

    #[test]
    fn a_lone_or_focused_sprite_gets_the_vote() {
        let lf = folders(&["char_003_kalts_1", "avg_npc_7_1"]);
        let mut votes = HashMap::new();
        vote_script(
            "[Character(name=\"char_003_kalts_1\")]\n[name=\"Kal'tsit\"]  Hello.\n\
             [Character(name=\"char_003_kalts_1\", name2=\"avg_npc_7_1#3\", focus=2)]\n[name=\"Guard\"]  Halt.\n\
             [Character(name=\"char_003_kalts_1\", name2=\"avg_npc_7_1\")]\n[name=\"Nobody\"]  Unattributed.\n",
            &lf,
            &mut votes,
        );
        assert_eq!(votes["char_003_kalts"]["Kal'tsit"], 1);
        assert_eq!(votes["avg_npc_7"]["Guard"], 1);
        assert!(!votes["char_003_kalts"].contains_key("Nobody"));
    }

    #[test]
    fn the_default_body_follows_the_reader_order() {
        let stems = |s: &[&str]| s.iter().map(|x| (*x).to_owned()).collect::<Vec<_>>();
        assert_eq!(
            default_body(
                "avg_npc_935_1",
                &stems(&["1$1", "avg_npc_935_1$1", "avg_npc_935_1$1[alpha]"])
            ),
            Some("avg_npc_935_1$1".to_owned())
        );
        assert_eq!(
            default_body(
                "char_003_kalts_1",
                &stems(&["char_003_kalts_2", "char_003_kalts_1"])
            ),
            Some("char_003_kalts_1".to_owned())
        );
        assert_eq!(default_body("char_empty", &stems(&["1"])), None);
    }
}
