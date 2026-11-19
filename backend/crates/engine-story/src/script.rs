use std::collections::BTreeMap;
use std::io;

use myrtle_gamedata_raw::TableSource;
use serde::{Deserialize, Serialize};

use crate::assets::{CharacterSprite, StoryAssetIndex, StoryAssets, parse_sprite_name, trim_index};
use crate::parser::{self, StoryCommand};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryScript {
    pub id: String,
    pub name: String,
    pub group_id: String,
    pub commands: Vec<StoryCommand>,
    pub assets: StoryAssets,
    pub word_count: u32,
    /// The `[uc]info` summary Skip dialog shows
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synopsis: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    #[error(
        "no script file for `{story_txt}` under gamedata/story (probed the primary and [uc]info paths)"
    )]
    Missing { story_txt: String },
    #[error("`{story_txt}` exists but holds no script (neither copy starts with `[`)")]
    NotAScript { story_txt: String },
    #[error("reading `{story_txt}`: {source}")]
    Io {
        story_txt: String,
        #[source]
        source: io::Error,
    },
}

fn looks_like_script(content: &str) -> bool {
    content
        .trim_start_matches('\u{feff}')
        .trim_start()
        .starts_with('[')
}

/// The script at `story/<txt>.txt.txt`, else `story/[uc]info/<txt>.txt.txt`
pub fn load_script(text: &dyn TableSource, story_txt: &str) -> Result<String, ScriptError> {
    let mut seen_any = false;
    for rel in [
        format!("gamedata/story/{story_txt}.txt.txt"),
        format!("gamedata/story/[uc]info/{story_txt}.txt.txt"),
    ] {
        let bytes = text.read_asset(&rel).map_err(|source| ScriptError::Io {
            story_txt: story_txt.to_owned(),
            source,
        })?;
        let Some(bytes) = bytes else { continue };
        seen_any = true;
        let content = String::from_utf8(bytes)
            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned());
        if looks_like_script(&content) {
            return Ok(content);
        }
    }
    let story_txt = story_txt.to_owned();
    Err(if seen_any {
        ScriptError::NotAScript { story_txt }
    } else {
        ScriptError::Missing { story_txt }
    })
}

#[must_use]
pub fn load_synopsis(text: &dyn TableSource, story_info: &str) -> Option<String> {
    if story_info.trim().is_empty() {
        return None;
    }
    let rest = story_info.strip_prefix("info/").unwrap_or(story_info);
    [
        format!("gamedata/story/[uc]info/{rest}.txt.txt"),
        format!("gamedata/story/{rest}.txt.txt"),
    ]
    .iter()
    .find_map(|rel| {
        let bytes = text.read_asset(rel).ok()??;
        let content = String::from_utf8_lossy(&bytes);
        if looks_like_script(&content) {
            return None;
        }
        let trimmed = content.trim_start_matches('\u{feff}').trim();
        (!trimmed.is_empty()).then(|| trimmed.replace("\r\n", "\n"))
    })
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpriteFacts {
    pub base: String,
    /// `#N` with zeros trimmed, `@alias`, or `1` for a bare name
    pub faces: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScriptFacts {
    pub word_count: u32,
    pub has_video: bool,
    pub backgrounds: Vec<String>,
    pub images: Vec<String>,
    pub sprites: Vec<SpriteFacts>,
}

impl ScriptFacts {
    #[must_use]
    pub fn first_background(&self) -> Option<&str> {
        self.backgrounds.first().map(String::as_str)
    }
}

fn push_distinct(names: &mut Vec<String>, name: &str) {
    if !names.iter().any(|n| n == name) {
        names.push(name.to_owned());
    }
}

#[must_use]
pub fn script_facts(
    text: &dyn TableSource,
    story_txt: &str,
    index: &StoryAssetIndex,
) -> Option<ScriptFacts> {
    let commands = parser::parse(&load_script(text, story_txt).ok()?);
    let mut facts = ScriptFacts {
        word_count: parser::word_count(&commands),
        has_video: commands.iter().any(|c| c.kind == "video"),
        ..ScriptFacts::default()
    };
    walk_refs(&commands, index, |kind, name| match kind {
        RefKind::Background => push_distinct(&mut facts.backgrounds, name),
        RefKind::Image | RefKind::InterludeImage => push_distinct(&mut facts.images, name),
        RefKind::Character => {
            let parts = parse_sprite_name(name);
            if parts.base.is_empty() {
                return;
            }
            let face = parts.alias.map_or_else(
                || trim_index(parts.face.unwrap_or("1")).to_owned(),
                |a| format!("@{a}"),
            );
            match facts.sprites.iter_mut().find(|s| s.base == parts.base) {
                Some(seen) if !seen.faces.contains(&face) => seen.faces.push(face),
                Some(_) => {}
                None => facts.sprites.push(SpriteFacts {
                    base: parts.base.to_owned(),
                    faces: vec![face],
                }),
            }
        }
        RefKind::Music | RefKind::Sound | RefKind::Avatar => {}
    });
    Some(facts)
}

const BACKGROUND_KINDS: &[&str] = &[
    "background",
    "backgroundtween",
    "largebg",
    "gridbg",
    "verticalbg",
];
const CHARACTER_KINDS: &[&str] = &["character", "charslot", "charactercutin", "interlude"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefKind {
    Background,
    Image,
    Character,
    InterludeImage,
    Music,
    Sound,
    Avatar,
}

fn arg<'a>(c: &'a StoryCommand, key: &str) -> Option<&'a str> {
    c.args.get(key).map(|v| v.trim()).filter(|v| !v.is_empty())
}

pub fn walk_refs(
    commands: &[StoryCommand],
    index: &StoryAssetIndex,
    mut emit: impl FnMut(RefKind, &str),
) {
    for c in commands {
        let kind = c.kind.as_str();
        if BACKGROUND_KINDS.contains(&kind) {
            for key in ["image", "imagegroup", "cggroup"] {
                for name in c
                    .args
                    .get(key)
                    .into_iter()
                    .flat_map(|v| v.split('/'))
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    emit(RefKind::Background, name);
                }
            }
        } else if let Some(name) = arg(c, "image") {
            emit(RefKind::Image, name);
        }
        if CHARACTER_KINDS.contains(&kind) {
            for (k, v) in &c.args {
                let name = v.trim();
                if !k.starts_with("name") || name.is_empty() {
                    continue;
                }
                if kind == "interlude" && index.resolve_character(name).is_none() {
                    emit(RefKind::InterludeImage, name);
                } else {
                    emit(RefKind::Character, name);
                }
            }
        }
        match kind {
            "popupdialog" => {
                if let Some(head) = arg(c, "dialoghead") {
                    emit(RefKind::Avatar, head);
                }
            }
            "playmusic" => {
                for v in ["intro", "key"].into_iter().filter_map(|k| arg(c, k)) {
                    emit(RefKind::Music, v);
                }
            }
            "playsound" => {
                if let Some(v) = arg(c, "key") {
                    emit(RefKind::Sound, v);
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct AssetRefs {
    pub backgrounds: BTreeMap<String, Option<String>>,
    pub images: BTreeMap<String, Option<String>>,
    pub characters: BTreeMap<String, Option<CharacterSprite>>,
    pub music: BTreeMap<String, Option<String>>,
    pub sounds: BTreeMap<String, Option<String>>,
    pub avatars: BTreeMap<String, Option<String>>,
}

#[must_use]
pub fn resolve_refs(commands: &[StoryCommand], index: &StoryAssetIndex) -> AssetRefs {
    let mut refs = AssetRefs::default();
    walk_refs(commands, index, |kind, name| {
        let key = name.to_owned();
        match kind {
            RefKind::Background => {
                refs.backgrounds
                    .entry(key)
                    .or_insert_with(|| index.resolve_background(name).map(|(u, _)| u));
            }
            RefKind::Image => {
                refs.images
                    .entry(key)
                    .or_insert_with(|| index.resolve_image(name).map(|(u, _)| u));
            }
            RefKind::Character => {
                refs.characters
                    .entry(key)
                    .or_insert_with(|| index.resolve_character(name));
            }
            RefKind::InterludeImage => {
                refs.characters.remove(name);
                refs.images
                    .entry(key)
                    .or_insert_with(|| index.resolve_image(name).map(|(u, _)| u));
            }
            RefKind::Music => {
                refs.music
                    .entry(key)
                    .or_insert_with(|| index.resolve_audio(name));
            }
            RefKind::Sound => {
                refs.sounds
                    .entry(key)
                    .or_insert_with(|| index.resolve_audio(name));
            }
            RefKind::Avatar => {
                refs.avatars
                    .entry(key)
                    .or_insert_with(|| index.resolve_avatar(name));
            }
        }
    });
    refs
}

fn hits<T>(map: BTreeMap<String, Option<T>>) -> BTreeMap<String, T> {
    map.into_iter().filter_map(|(k, v)| Some((k, v?))).collect()
}

fn collect_assets(commands: &[StoryCommand], index: &StoryAssetIndex) -> StoryAssets {
    let refs = resolve_refs(commands, index);
    let mut assets = StoryAssets {
        backgrounds: hits(refs.backgrounds),
        images: hits(refs.images),
        characters: hits(refs.characters),
        sounds: hits(refs.sounds),
        avatars: hits(refs.avatars),
        ..StoryAssets::default()
    };
    // Sized by the same lookup order the URL came from
    for name in assets.backgrounds.keys() {
        if let Some(size) = index.resolve_background_size(name) {
            assets.image_sizes.insert(name.clone(), size);
        }
    }
    for name in assets.images.keys() {
        if let Some(size) = index.resolve_image_size(name) {
            assets.image_sizes.insert(name.clone(), size);
        }
    }
    for c in commands {
        match c.kind.as_str() {
            "video" => {
                if let Some(res) = arg(c, "res")
                    && !assets.videos.contains_key(res)
                    && let Some(sources) = index.resolve_video(res)
                {
                    assets.videos.insert(res.to_owned(), sources);
                }
            }
            "playmusic" => {
                let loop_key = arg(c, "key");
                let intro_key = arg(c, "intro");
                let Some(cue_key) = loop_key.or(intro_key) else {
                    continue;
                };
                let url = |k: Option<&str>| k.and_then(|k| refs.music.get(k).cloned().flatten());
                let (loop_url, intro_url) = (url(loop_key), url(intro_key));
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
            _ => {}
        }
    }
    assets
}

#[must_use]
pub fn parse_story(
    id: &str,
    name: &str,
    group_id: &str,
    script: &str,
    index: &StoryAssetIndex,
) -> StoryScript {
    let commands = parser::parse(script);
    StoryScript {
        id: id.to_owned(),
        name: name.to_owned(),
        group_id: group_id.to_owned(),
        word_count: parser::word_count(&commands),
        assets: collect_assets(&commands, index),
        commands,
        synopsis: None,
    }
}
