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
    pub aliases: Vec<String>,
}

impl StorySprite {
    #[must_use]
    pub fn body_path(&self) -> String {
        format!("/textures/avg/characters/{}/{}.png", self.folder, self.body)
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct StorySprites {
    sprites: BTreeMap<String, StorySprite>,
}

impl StorySprites {
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&StorySprite> {
        self.sprites.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &StorySprite> {
        self.sprites.values()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    #[must_use]
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

#[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Folders that each hold a body spelled like the folder itself.
    fn tree(names: &[&str]) -> HashMap<String, Vec<String>> {
        names
            .iter()
            .map(|n| ((*n).to_owned(), vec![(*n).to_owned()]))
            .collect()
    }

    fn folders(names: &[&str]) -> HashMap<String, String> {
        character_ids(&tree(names))
    }

    #[test]
    fn a_bare_avg_npc_number_is_the_character_not_a_set() {
        let t = tree(&[
            "avg_npc_001",
            "avg_npc_061",
            "avg_npc_102",
            "avg_npc_102_2",
            "avg_npc_935_1",
            "avg_npc_935_2",
            "avg_doc_1",
            "npc_10002",
            "char_003_kalts_1",
            "char_242_mayer",
        ]);
        let ids = character_ids(&t);
        let id = |f: &str| ids[f].as_str();
        // Maria Nearl's folder is hers, not one set of a shared `avg_npc`.
        assert_eq!(id("avg_npc_061"), "avg_npc_061");
        // The folder the old rule kept keeps the id it was saved under.
        assert_eq!(id("avg_npc_001"), "avg_npc");
        // A bare folder whose number a set-numbered character already holds
        // is someone else: the sets keep the id, the bare folder takes `_0`.
        assert_eq!(id("avg_npc_102_2"), "avg_npc_102");
        assert_eq!(id("avg_npc_102"), "avg_npc_102_0");
        let sets = sets_by_character(&t, &ids);
        assert_eq!(sets["avg_npc_102"], vec![(2, "avg_npc_102_2")]);
        assert_eq!(sets["avg_npc_102_0"], vec![(102, "avg_npc_102")]);
        // Everything else cuts its set number as before.
        assert_eq!(id("avg_npc_935_2"), "avg_npc_935");
        assert_eq!(id("avg_doc_1"), "avg_doc");
        assert_eq!(id("npc_10002"), "npc");
        assert_eq!(id("char_003_kalts_1"), "char_003_kalts");
        assert_eq!(id("char_242_mayer"), "char_242_mayer");
        assert_eq!(sets.len(), 9);
    }

    #[test]
    fn a_line_spoken_over_a_bare_avg_npc_votes_for_that_character() {
        let ids = folders(&[
            "avg_npc_001",
            "avg_npc_061",
            "avg_npc_120",
            "avg_npc_208",
            "avg_npc_208_1",
        ]);
        let mut votes = HashMap::new();
        vote_script(
            "[Character(name=\"avg_npc_061#2\",fadetime=1,block=true)]\n[name=\"Maria\"]  Hi, everyone!\n\
             [Character(name=\"avg_npc_120\", name2=\"avg_npc_061#7\", focus=2)]\n[name=\"Maria\"]  ......\n\
             [Character(name=\"avg_npc_208\")]\n[name=\"Cannot\"]  Huh.\n\
             [Character(name=\"avg_npc_208_1#2\")]\n[name=\"Monique\"]  Hm.\n",
            &ids,
            &mut votes,
        );
        assert_eq!(votes["avg_npc_061"]["Maria"], 2);
        assert!(!votes.contains_key("avg_npc"));
        assert_eq!(votes["avg_npc_208_0"]["Cannot"], 1);
        assert_eq!(votes["avg_npc_208"]["Monique"], 1);
    }

    #[test]
    fn runner_up_names_over_the_floor_are_aliases() {
        let votes = |v: &[(&str, u32)]| -> HashMap<String, u32> {
            v.iter().map(|(n, c)| ((*n).to_owned(), *c)).collect()
        };
        let names = |v: &[(&str, u32)]| pick_names(votes(v));
        // avg_npc_061 on EN: Maria wins, Blemishine (93 lines) clears 10% of 772.
        let maria = names(&[("Maria", 772), ("Blemishine", 93), ("Guard", 4)]).unwrap();
        assert_eq!(maria.speaker, "Maria");
        assert_eq!(maria.aliases, vec!["Blemishine".to_owned()]);
        // The floor is max(5, winner / 10), inclusive on both, unrounded.
        let small = names(&[("A", 20), ("B", 5), ("C", 4)]).unwrap();
        assert_eq!(small.aliases, vec!["B".to_owned()]);
        let big = names(&[("A", 101), ("B", 11), ("C", 10)]).unwrap();
        assert_eq!(big.aliases, vec!["B".to_owned()]);
        // Never the winner again, in any case or wrapping; never a letterless
        // `???`; most votes first, ties by name, at most four.
        let messy = names(&[
            ("Talulah", 500),
            ("talulah?", 90),
            ("???", 80),
            ("'Deathless Black Snake'", 60),
            ("E", 50),
            ("D", 50),
            ("F", 50),
            ("G", 50),
            ("H", 40),
        ])
        .unwrap();
        assert_eq!(
            messy.aliases,
            vec!["'Deathless Black Snake'", "D", "E", "F"]
        );
        // No winner, no aliases.
        assert_eq!(names(&[("A", 10), ("B", 9), ("C", 9)]), None);
        assert_eq!(names(&[("A", 1)]), None);
        assert!(names(&[("A", 30)]).unwrap().aliases.is_empty());
    }

    #[test]
    fn a_folder_no_file_rule_spells_falls_back_to_its_hub() {
        // Ported from main: the hub is now the parsed `hub.json`, not its path.
        let hub: Value = serde_json::from_str(
            r#"{"groups":[{"facePos":{"x":-1,"y":-1},"sprites":[{"name":"char_2006_fmzuki_1"}]}]}"#,
        )
        .unwrap();
        let stems = vec!["char_2006_fmzuki_1".to_owned(), "hub".to_owned()];
        assert_eq!(default_body("char_2006_weiywfmzuki_1", &stems), None);
        assert_eq!(
            hub_body(Some(&hub), &stems),
            Some("char_2006_fmzuki_1".to_owned())
        );
        // A hub naming a file the folder lacks gives nothing.
        assert_eq!(hub_body(Some(&hub), &["other".to_owned()]), None);
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
