use std::collections::{BTreeSet, HashMap};
use std::sync::LazyLock;

use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::story::StoryVariables;
use regex::Regex;

use crate::tables::MusicTables;

pub const CLIP_ROOTS: [&str; 2] = ["audio/audio/sound_beta_2", "audio/anon"];

#[derive(Default)]
pub struct MusicSources {
    pub tables: MusicTables,
    pub scripts: Vec<StoryScript>,
    /// Level id -> `BgmEvent`
    pub level_bgm: HashMap<String, String>,
    pub clip_trees: Vec<ClipTree>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClipTree {
    pub server: Option<String>,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptPlay {
    pub loop_path: String,
    pub intro: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StoryScript {
    pub path: String,
    pub plays: Vec<ScriptPlay>,
    pub utf8: bool,
    /// Opens with a `[` command
    pub is_script: bool,
}

impl StoryScript {
    pub fn read(path: impl Into<String>, bytes: &[u8], vars: &StoryVariables) -> Self {
        let text = String::from_utf8_lossy(bytes);
        Self {
            path: path.into().replace('\\', "/"),
            plays: script_music(&text, vars),
            utf8: std::str::from_utf8(bytes).is_ok(),
            is_script: text
                .trim_start_matches('\u{feff}')
                .trim_start()
                .starts_with('['),
        }
    }
}

static PLAY_MUSIC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\[\s*playmusic\s*\(([^)\]]*)\)").expect("static regex"));
static MUSIC_ARG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?i)\b(intro|key)\s*=\s*"([^"]*)""#).expect("static regex"));

pub fn script_music(text: &str, vars: &StoryVariables) -> Vec<ScriptPlay> {
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
        if let Some(key) = key.filter(|k| k.contains('/')) {
            out.push(ScriptPlay {
                loop_path: key,
                intro: intro.filter(|i| i.contains('/')),
            });
        }
    }
    out
}

#[must_use]
pub fn level_ids(model: &GameModel) -> Vec<String> {
    model
        .stage_index
        .iter()
        .filter_map(|e| e.level_id.as_deref().map(level_key))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(crate) fn level_key(level_id: &str) -> String {
    level_id.trim_start_matches('/').to_ascii_lowercase()
}

#[must_use]
pub fn bgm_event(bytes: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(bytes).ok()?;
    let at = text.find("\"BgmEvent\"")?;
    let rest = text[at + "\"BgmEvent\"".len()..].trim_start();
    let rest = rest.strip_prefix(':')?.trim_start().strip_prefix('"')?;
    let end = rest.find('"')?;
    let event = rest[..end].trim();
    (!event.is_empty()).then(|| event.to_owned())
}

/// Lowercase stem -> `/{path}`
#[must_use]
pub fn clip_index(paths: &[String]) -> HashMap<String, String> {
    let paths: Vec<String> = paths.iter().map(|p| p.replace('\\', "/")).collect();
    let mut out = HashMap::new();
    for root in CLIP_ROOTS {
        let mut clips: Vec<(&str, &str)> = paths
            .iter()
            .filter(|p| p.strip_prefix(root).is_some_and(|r| r.starts_with('/')))
            .filter_map(|p| {
                let name = p.rsplit('/').next()?;
                let stem = name.strip_suffix(".ogg").filter(|s| !s.is_empty())?;
                Some((p.as_str(), stem))
            })
            .collect();
        clips.sort_by(|a, b| a.0.split('/').cmp(b.0.split('/')));
        for (path, stem) in clips {
            out.entry(stem.to_ascii_lowercase())
                .or_insert_with(|| format!("/{path}"));
        }
    }
    out
}
