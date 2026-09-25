//! The story reader's core: script loading, parsing and asset resolution.
//! `docs/story-reader.md` holds the data facts and the wire contract.
//!
//! The script is at `story/<txt>.txt.txt` and the one-line summary at
//! `story/[uc]info/<txt>.txt.txt`, which is the client's own layout: the two
//! are separate assets in separate bundles that share one asset name, and the
//! unpacker used to route by that name alone, so the summary landed on the
//! script (`assets/unpacker/src/export/manifest.rs`). Corrected on disk
//! 2026-09-23: 1,862 of 1,887 library stories are a script at the PRIMARY
//! path and nothing under `[uc]info/` starts with `[`. [`load_script`] still
//! probes both and takes whichever starts with `[`, primary first, because a
//! tree extracted before the fix is still swapped and both layouts must read.

pub mod assets;
pub mod parser;
pub mod variables;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use assets::{
    CharacterSprite, FacePos, ImageSize, MusicCue, PlateRect, StoryAssetIndex, StoryAssets,
    VideoSources,
};
pub use parser::StoryCommand;

/// One story, parsed, with every asset it references resolved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StoryScript {
    pub id: String,
    pub name: String,
    pub group_id: String,
    pub commands: Vec<StoryCommand>,
    pub assets: StoryAssets,
    pub word_count: u32,
    /// The game's own "story summary" for this story, the text its Skip
    /// dialog shows: the `[uc]info` file `StoryInfo` names, trimmed. Absent
    /// when the table names none or the file is not on disk. It rides the
    /// SCRIPT response and not the library index, because the index would
    /// carry all 1,881 of them on every library load.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub synopsis: Option<String>,
}

/// Why a script could not be loaded.
#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    /// Neither probe path exists.
    #[error(
        "no script file for `{story_txt}` under gamedata/story (probed the primary and [uc]info paths)"
    )]
    Missing { story_txt: String },
    /// A file exists but neither copy is a script (neither starts with `[`).
    #[error("`{story_txt}` exists but holds no script (neither copy starts with `[`)")]
    NotAScript { story_txt: String },
    #[error("reading `{story_txt}`: {source}")]
    Io {
        story_txt: String,
        #[source]
        source: std::io::Error,
    },
}

/// The two places a script can sit, primary first.
#[must_use]
pub fn script_paths(server_assets_dir: &Path, story_txt: &str) -> [PathBuf; 2] {
    let story = server_assets_dir.join("gamedata/story");
    [
        story.join(format!("{story_txt}.txt.txt")),
        story.join(format!("[uc]info/{story_txt}.txt.txt")),
    ]
}

fn looks_like_script(content: &str) -> bool {
    content
        .trim_start_matches('\u{feff}')
        .trim_start()
        .starts_with('[')
}

/// True when [`load_script`] would succeed, by the same probe, without
/// reading more than the first bytes of each candidate.
#[must_use]
pub fn has_script(server_assets_dir: &Path, story_txt: &str) -> bool {
    use std::io::Read;
    for path in script_paths(server_assets_dir, story_txt) {
        let Ok(mut f) = std::fs::File::open(&path) else {
            continue;
        };
        let mut head = [0u8; 16];
        let n = f.read(&mut head).unwrap_or(0);
        if looks_like_script(&String::from_utf8_lossy(&head[..n])) {
            return true;
        }
    }
    false
}

/// One sprite folder a script references, with the `#face` indices it asks
/// for. The base is the name with `#N$M` stripped, which is the folder under
/// `textures/avg/characters/`, so every body and face of one character
/// collapses to one row.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpriteFacts {
    pub base: String,
    /// Distinct face keys in first-appearance order: the `#N` index with its
    /// leading zeros trimmed, `@alias` for the alias form, and `1` for a bare
    /// name, because a missing index IS the first face.
    pub faces: Vec<String>,
}

/// What the library index reads out of one script, in a single pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScriptFacts {
    /// The prose word count, by the same rule as [`StoryScript::word_count`].
    pub word_count: u32,
    /// The name the FIRST background command writes, as written: the group's
    /// derived cover when the group has no `StoryEntryPic`. It is
    /// `backgrounds.first()`, the same walk in the same order.
    pub first_background: Option<String>,
    /// A `[Video]` command is present, so the story plays a movie rather than
    /// (or as well as) a scene.
    pub has_video: bool,
    /// Distinct background names in first-appearance order, as written.
    pub backgrounds: Vec<String>,
    /// Distinct CG names in first-appearance order, as written.
    pub images: Vec<String>,
    /// Distinct sprite folders in first-appearance order.
    pub sprites: Vec<SpriteFacts>,
}

fn push_distinct(names: &mut Vec<String>, name: &str) {
    if !names.iter().any(|n| n == name) {
        names.push(name.to_owned());
    }
}

/// Load one script, parse it, and read the facts the index carries. `None`
/// when no script file backs the path, which is exactly when [`has_script`]
/// is false, so the index builder gets every answer from one read. The
/// distinct-name lists are classified by [`walk_refs`], the same rules
/// `GET /story/{id}` resolves through, so the illustrations tab and the
/// reader never disagree about what a name IS.
#[must_use]
pub fn script_facts(
    server_assets_dir: &Path,
    story_txt: &str,
    index: &StoryAssetIndex,
) -> Option<ScriptFacts> {
    let script = load_script(server_assets_dir, story_txt).ok()?;
    let commands = parser::parse(&script);
    let mut facts = ScriptFacts {
        word_count: parser::word_count(&commands),
        has_video: commands.iter().any(|c| c.kind == "video"),
        ..ScriptFacts::default()
    };
    walk_refs(&commands, index, |kind, name| match kind {
        RefKind::Background => push_distinct(&mut facts.backgrounds, name),
        RefKind::Image | RefKind::InterludeImage => push_distinct(&mut facts.images, name),
        RefKind::Character => {
            let parts = assets::parse_sprite_name(name);
            if parts.base.is_empty() {
                return;
            }
            let face = parts.alias.map_or_else(
                || assets::trim_index(parts.face.unwrap_or("1")).to_owned(),
                |a| format!("@{a}"),
            );
            if let Some(seen) = facts.sprites.iter_mut().find(|s| s.base == parts.base) {
                if !seen.faces.contains(&face) {
                    seen.faces.push(face);
                }
            } else {
                facts.sprites.push(SpriteFacts {
                    base: parts.base.to_owned(),
                    faces: vec![face],
                });
            }
        }
        RefKind::Music | RefKind::Sound | RefKind::Avatar => {}
    });
    facts.first_background = facts.backgrounds.first().cloned();
    Some(facts)
}

/// Load a script's text by its `StoryTxt` path, probing the primary and the
/// `[uc]info/` locations and returning whichever content starts with `[`
/// (the primary when both do).
pub fn load_script(server_assets_dir: &Path, story_txt: &str) -> Result<String, ScriptError> {
    let mut seen_any = false;
    for path in script_paths(server_assets_dir, story_txt) {
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                seen_any = true;
                if looks_like_script(&content) {
                    return Ok(content);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                // Not UTF-8: read lossily rather than fail the story.
                seen_any = true;
                if let Ok(bytes) = std::fs::read(&path) {
                    let content = String::from_utf8_lossy(&bytes).into_owned();
                    if looks_like_script(&content) {
                        return Ok(content);
                    }
                }
            }
            Err(source) => {
                return Err(ScriptError::Io {
                    story_txt: story_txt.to_owned(),
                    source,
                });
            }
        }
    }
    if seen_any {
        Err(ScriptError::NotAScript {
            story_txt: story_txt.to_owned(),
        })
    } else {
        Err(ScriptError::Missing {
            story_txt: story_txt.to_owned(),
        })
    }
}

/// The two places a story summary can sit, `[uc]info` first. `story_info` is
/// the table's `StoryInfo` (`info/activities/a001/level_a001_01_beg`): its
/// `info/` prefix is the `[uc]info/` directory on disk. The second path is the
/// PRIMARY one, where a tree extracted before the unpacker's routing fix put
/// the summary (see the module doc), so both layouts read.
#[must_use]
pub fn synopsis_paths(server_assets_dir: &Path, story_info: &str) -> [PathBuf; 2] {
    let story = server_assets_dir.join("gamedata/story");
    let rest = story_info.strip_prefix("info/").unwrap_or(story_info);
    [
        story.join(format!("[uc]info/{rest}.txt.txt")),
        story.join(format!("{rest}.txt.txt")),
    ]
}

/// The story summary `story_info` names, trimmed, or `None` when neither
/// probe path holds one. A candidate that starts with `[` is a SCRIPT (the
/// swapped layout) and is passed over, the mirror of [`load_script`]'s rule.
/// Read uncached, like the script it rides with: one file of 18 to 490 bytes
/// on the EN tree (1,881 files, median 195).
#[must_use]
pub fn load_synopsis(server_assets_dir: &Path, story_info: &str) -> Option<String> {
    if story_info.trim().is_empty() {
        return None;
    }
    for path in synopsis_paths(server_assets_dir, story_info) {
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let content = String::from_utf8_lossy(&bytes);
        if looks_like_script(&content) {
            continue;
        }
        let text = content.trim_start_matches('\u{feff}').trim();
        if !text.is_empty() {
            return Some(text.replace("\r\n", "\n"));
        }
    }
    None
}

/// Commands whose `image`/`imagegroup` name a background. `backgroundtween`
/// carries `image=` in 38 EN uses and must look in `bg/` first like the rest.
const BACKGROUND_KINDS: &[&str] = &[
    "background",
    "backgroundtween",
    "largebg",
    "gridbg",
    "verticalbg",
];
/// Commands whose `name*` arguments name character sprites. `interlude`
/// names a sprite in 258 of its 377 named uses and a background or CG in the
/// rest, so it is resolved as a character first and falls through below.
const CHARACTER_KINDS: &[&str] = &["character", "charslot", "charactercutin", "interlude"];

/// Parse a script and resolve the assets it references.
#[must_use]
pub fn parse_story(
    id: &str,
    name: &str,
    group_id: &str,
    script: &str,
    index: &StoryAssetIndex,
) -> StoryScript {
    let commands = parser::parse(script);
    let word_count = parser::word_count(&commands);
    let assets = collect_assets(&commands, index);
    StoryScript {
        id: id.to_owned(),
        name: name.to_owned(),
        group_id: group_id.to_owned(),
        commands,
        assets,
        word_count,
        synopsis: None,
    }
}

/// Every asset name the commands reference, in the order the resolver sees
/// them, and whether each resolved. The parse keeps only the resolved ones;
/// the integration test reads the misses.
#[derive(Debug, Default, Clone)]
pub struct AssetRefs {
    pub backgrounds: BTreeMap<String, Option<String>>,
    pub images: BTreeMap<String, Option<String>>,
    pub characters: BTreeMap<String, Option<CharacterSprite>>,
    pub music: BTreeMap<String, Option<String>>,
    pub sounds: BTreeMap<String, Option<String>>,
    pub avatars: BTreeMap<String, Option<String>>,
}

/// What a referenced name IS, as the resolver classifies it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefKind {
    Background,
    Image,
    Character,
    /// An `[interlude]` panel that names no sprite, so it is a background, a
    /// CG or a `cutin_char_*` plate. It reads as an image AND cancels the
    /// character entry the same name would otherwise hold.
    InterludeImage,
    Music,
    Sound,
    Avatar,
}

/// Walk every asset reference in `commands` IN SCRIPT ORDER, classified,
/// repeated as often as it is written. This is the ONE place the command
/// kinds and argument keys are read: [`resolve_refs`] builds the wire maps
/// from it and [`script_facts`] the index's distinct-name lists, so the two
/// can never drift apart.
pub fn walk_refs(
    commands: &[StoryCommand],
    index: &StoryAssetIndex,
    mut emit: impl FnMut(RefKind, &str),
) {
    for c in commands {
        let kind = c.kind.as_str();
        if BACKGROUND_KINDS.contains(&kind) {
            // `largebg`/`gridbg` name 2 or 4 panels joined by `/`;
            // `largebg` writes CG panels under `cggroup` (3 EN uses).
            for key in ["image", "imagegroup", "cggroup"] {
                if let Some(v) = c.args.get(key) {
                    for name in v.split('/').map(str::trim).filter(|s| !s.is_empty()) {
                        emit(RefKind::Background, name);
                    }
                }
            }
        } else if let Some(v) = c.args.get("image") {
            let name = v.trim();
            if !name.is_empty() {
                emit(RefKind::Image, name);
            }
        }
        if CHARACTER_KINDS.contains(&kind) {
            for (k, v) in &c.args {
                if !k.starts_with("name") {
                    continue;
                }
                let name = v.trim();
                if name.is_empty() {
                    continue;
                }
                if kind == "interlude" && index.resolve_character(name).is_none() {
                    emit(RefKind::InterludeImage, name);
                } else {
                    emit(RefKind::Character, name);
                }
            }
        }
        if kind == "popupdialog"
            && let Some(v) = c.args.get("dialoghead")
        {
            let head = v.trim();
            if !head.is_empty() {
                emit(RefKind::Avatar, head);
            }
        }
        match kind {
            "playmusic" => {
                for key in ["intro", "key"] {
                    if let Some(v) = c.args.get(key) {
                        let v = v.trim();
                        if !v.is_empty() {
                            emit(RefKind::Music, v);
                        }
                    }
                }
            }
            "playsound" => {
                if let Some(v) = c.args.get("key") {
                    let v = v.trim();
                    if !v.is_empty() {
                        emit(RefKind::Sound, v);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Resolve every reference in `commands`, keeping misses as `None`.
#[must_use]
pub fn resolve_refs(commands: &[StoryCommand], index: &StoryAssetIndex) -> AssetRefs {
    let mut refs = AssetRefs::default();
    walk_refs(commands, index, |kind, name| match kind {
        RefKind::Background => {
            refs.backgrounds
                .entry(name.to_owned())
                .or_insert_with(|| index.resolve_background(name).map(|(u, _)| u));
        }
        RefKind::Image => {
            refs.images
                .entry(name.to_owned())
                .or_insert_with(|| index.resolve_image(name).map(|(u, _)| u));
        }
        RefKind::Character => {
            refs.characters
                .entry(name.to_owned())
                .or_insert_with(|| index.resolve_character(name));
        }
        RefKind::InterludeImage => {
            // The character entry is dropped rather than left as a miss: it
            // was never a sprite name.
            refs.characters.remove(name);
            refs.images
                .entry(name.to_owned())
                .or_insert_with(|| index.resolve_image(name).map(|(u, _)| u));
        }
        RefKind::Music => {
            refs.music
                .entry(name.to_owned())
                .or_insert_with(|| index.resolve_audio(name));
        }
        RefKind::Sound => {
            refs.sounds
                .entry(name.to_owned())
                .or_insert_with(|| index.resolve_audio(name));
        }
        RefKind::Avatar => {
            refs.avatars
                .entry(name.to_owned())
                .or_insert_with(|| index.resolve_avatar(name));
        }
    });
    refs
}

fn collect_assets(commands: &[StoryCommand], index: &StoryAssetIndex) -> StoryAssets {
    let refs = resolve_refs(commands, index);
    let mut assets = StoryAssets {
        backgrounds: refs
            .backgrounds
            .into_iter()
            .filter_map(|(k, v)| v.map(|u| (k, u)))
            .collect(),
        images: refs
            .images
            .into_iter()
            .filter_map(|(k, v)| v.map(|u| (k, u)))
            .collect(),
        characters: refs
            .characters
            .into_iter()
            .filter_map(|(k, v)| v.map(|s| (k, s)))
            .collect(),
        music: BTreeMap::new(),
        sounds: refs
            .sounds
            .into_iter()
            .filter_map(|(k, v)| v.map(|u| (k, u)))
            .collect(),
        avatars: refs
            .avatars
            .clone()
            .into_iter()
            .filter_map(|(k, v)| v.map(|u| (k, u)))
            .collect(),
        image_sizes: BTreeMap::new(),
        videos: BTreeMap::new(),
    };
    // What SIZE each of those plates is drawn at, under the same key and by
    // the same lookup order the URL came from, because a name that lives in
    // both trees is sized by whichever file answered.
    let mut sizes: BTreeMap<String, assets::ImageSize> = BTreeMap::new();
    for name in assets.backgrounds.keys() {
        if let Some(size) = index.resolve_background_size(name) {
            sizes.insert(name.clone(), size);
        }
    }
    for name in assets.images.keys() {
        if let Some(size) = index.resolve_image_size(name) {
            sizes.insert(name.clone(), size);
        }
    }
    assets.image_sizes = sizes;
    // Cutscenes are keyed by the command's own `res`, verbatim and in the case
    // the script wrote, so the engine looks a halt up by the string it holds.
    // An unresolved reference is LEFT OUT rather than carried as a null: the
    // engine treats a missing key as a clip to skip.
    for c in commands.iter().filter(|c| c.kind == "video") {
        let Some(res) = c
            .args
            .get("res")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        else {
            continue;
        };
        if assets.videos.contains_key(res) {
            continue;
        }
        if let Some(sources) = index.resolve_video(res) {
            assets.videos.insert(res.to_owned(), sources);
        }
    }
    // Music is keyed by the loop key; the intro rides along on the same cue.
    // A `PlayMusic` with only an intro is keyed by that intro.
    for c in commands.iter().filter(|c| c.kind == "playmusic") {
        let loop_key = c
            .args
            .get("key")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        let intro_key = c
            .args
            .get("intro")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        let Some(cue_key) = loop_key.or(intro_key) else {
            continue;
        };
        let loop_url = loop_key.and_then(|k| refs.music.get(k).cloned().flatten());
        let intro_url = intro_key.and_then(|k| refs.music.get(k).cloned().flatten());
        if loop_url.is_none() && intro_url.is_none() {
            continue;
        }
        let cue = assets.music.entry(cue_key.to_owned()).or_default();
        if cue.r#loop.is_none() {
            cue.r#loop = loop_url;
        }
        if cue.intro.is_none() {
            cue.intro = intro_url;
        }
    }
    assets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_probe_prefers_primary_and_accepts_swapped() {
        let dir = std::env::temp_dir().join(format!("story-probe-{}", std::process::id()));
        let story = dir.join("gamedata/story");
        std::fs::create_dir_all(story.join("[uc]info/act/x")).unwrap();
        std::fs::create_dir_all(story.join("act/x")).unwrap();
        // Swapped: summary at the primary path, script under [uc]info.
        std::fs::write(story.join("act/x/a.txt.txt"), "A summary line.\n").unwrap();
        std::fs::write(story.join("[uc]info/act/x/a.txt.txt"), "[Dialog]\n").unwrap();
        assert_eq!(load_script(&dir, "act/x/a").unwrap(), "[Dialog]\n");
        assert!(has_script(&dir, "act/x/a"));
        // Both scripts: primary wins.
        std::fs::write(story.join("act/x/b.txt.txt"), "[HEADER] primary\n").unwrap();
        std::fs::write(story.join("[uc]info/act/x/b.txt.txt"), "[HEADER] info\n").unwrap();
        assert_eq!(load_script(&dir, "act/x/b").unwrap(), "[HEADER] primary\n");
        // Only a summary: NotAScript.
        std::fs::write(story.join("act/x/c.txt.txt"), "Only prose.\n").unwrap();
        assert!(matches!(
            load_script(&dir, "act/x/c"),
            Err(ScriptError::NotAScript { .. })
        ));
        assert!(!has_script(&dir, "act/x/c"));
        assert!(matches!(
            load_script(&dir, "act/x/nope"),
            Err(ScriptError::Missing { .. })
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn synopsis_probe_reads_uc_info_and_passes_over_a_script() {
        let dir = std::env::temp_dir().join(format!("story-synopsis-{}", std::process::id()));
        let story = dir.join("gamedata/story");
        std::fs::create_dir_all(story.join("[uc]info/act/x")).unwrap();
        std::fs::create_dir_all(story.join("act/x")).unwrap();
        // Corrected layout: script at the primary path, summary under [uc]info.
        std::fs::write(story.join("act/x/a.txt.txt"), "[Dialog]\n").unwrap();
        std::fs::write(
            story.join("[uc]info/act/x/a.txt.txt"),
            "\u{feff}First paragraph.\r\nSecond.\r\n",
        )
        .unwrap();
        assert_eq!(
            load_synopsis(&dir, "info/act/x/a").as_deref(),
            Some("First paragraph.\nSecond.")
        );
        // Swapped layout: the summary sits at the primary path.
        std::fs::write(story.join("act/x/b.txt.txt"), "Swapped summary.\n").unwrap();
        std::fs::write(story.join("[uc]info/act/x/b.txt.txt"), "[HEADER] x\n").unwrap();
        assert_eq!(
            load_synopsis(&dir, "info/act/x/b").as_deref(),
            Some("Swapped summary.")
        );
        // Nothing but scripts, an empty file, a missing file, an empty name.
        std::fs::write(story.join("act/x/c.txt.txt"), "[Dialog]\n").unwrap();
        std::fs::write(story.join("[uc]info/act/x/c.txt.txt"), "  \n").unwrap();
        assert_eq!(load_synopsis(&dir, "info/act/x/c"), None);
        assert_eq!(load_synopsis(&dir, "info/act/x/nope"), None);
        assert_eq!(load_synopsis(&dir, ""), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn script_facts_list_distinct_names_in_first_appearance_order() {
        let dir = std::env::temp_dir().join(format!("story-facts-{}", std::process::id()));
        let story = dir.join("gamedata/story/act/x");
        std::fs::create_dir_all(&story).unwrap();
        std::fs::write(
            story.join("a.txt.txt"),
            concat!(
                "[Background(image=\"bg_two\")]\n",
                "[charslot(slot=\"l\", name=\"avg_1_a_1#03$1\")]\n",
                "[Image(image=\"cg_one\")]\n",
                "[Background(image=\"bg_two\")]\n",
                "[charslot(slot=\"l\", name=\"avg_1_a_1#11$2\")]\n",
                "[character(name=\"char_002_amiya_1\")]\n",
                "[largebg(imagegroup=\"bg_p1/bg_p2\")]\n",
                "[Video(id=\"v\")]\n",
            ),
        )
        .unwrap();
        let facts = script_facts(&dir, "act/x/a", &StoryAssetIndex::default()).unwrap();
        // Distinct, in the order written, `largebg`'s two panels included.
        assert_eq!(facts.backgrounds, ["bg_two", "bg_p1", "bg_p2"]);
        assert_eq!(facts.images, ["cg_one"]);
        assert_eq!(facts.first_background.as_deref(), Some("bg_two"));
        assert!(facts.has_video);
        // One row per FOLDER; `#03` trims to `3` and a bare name is face 1.
        let bases: Vec<&str> = facts.sprites.iter().map(|s| s.base.as_str()).collect();
        assert_eq!(bases, ["avg_1_a_1", "char_002_amiya_1"]);
        assert_eq!(facts.sprites[0].faces, ["3", "11"]);
        assert_eq!(facts.sprites[1].faces, ["1"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_story_collects_only_referenced_and_resolved_assets() {
        let index = StoryAssetIndex::default();
        let s = parse_story(
            "id",
            "Name",
            "grp",
            "[Background(image=\"bg_x\")]\n[name=\"A\"] one two\n[PlayMusic(key=\"$m\")]\n",
            &index,
        );
        assert_eq!(s.commands.len(), 3);
        assert_eq!(s.word_count, 2);
        assert!(s.assets.backgrounds.is_empty());
        assert!(s.assets.music.is_empty());
        let refs = resolve_refs(&s.commands, &index);
        assert_eq!(refs.backgrounds.get("bg_x"), Some(&None));
        assert_eq!(refs.music.get("$m"), Some(&None));
    }
}
