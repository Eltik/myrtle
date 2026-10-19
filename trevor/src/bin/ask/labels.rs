//! How a passage is labelled and whose it is: the names table, story labels, and the operator a typed unit or an
//! operator-file chunk belongs to.

use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use serde::Deserialize;
use trevor::tools::Tools;


#[derive(Deserialize)]
pub(crate) struct Names {
    pub(crate) story: String,
    pub(crate) group: String,
}

/// Speaker labels a story renames by a self-introduction (`artifacts/entities/label_renames.json`,
/// scripts/label_renames.py): story id -> description label -> name. Empty (and logged) when missing or unparsable.
fn label_renames() -> &'static HashMap<String, BTreeMap<String, String>> {
    static RENAMES: OnceLock<HashMap<String, BTreeMap<String, String>>> = OnceLock::new();
    RENAMES.get_or_init(|| {
        trevor::util::load_json_opt(std::path::Path::new("artifacts/entities/label_renames.json")).unwrap_or_default()
    })
}

/// Chunks whose "???" speaker label an introduction names (`artifacts/entities/label_renames_chunks.json`,
/// scripts/label_renames.py, 2026-10-19): chunk id -> "???" -> name. "???" names different people within one story,
/// so these are scoped to the chunk. Empty (and logged) when missing or unparsable.
fn chunk_label_renames() -> &'static HashMap<String, BTreeMap<String, String>> {
    static RENAMES: OnceLock<HashMap<String, BTreeMap<String, String>>> = OnceLock::new();
    RENAMES.get_or_init(|| {
        trevor::util::load_json_opt(std::path::Path::new("artifacts/entities/label_renames_chunks.json"))
            .unwrap_or_default()
    })
}

/// The label renames of chunk `chunk_id` ("???" -> "Mon3tr"), if it has any.
pub(crate) fn chunk_renames(chunk_id: &str) -> Option<&'static BTreeMap<String, String>> {
    chunk_label_renames().get(chunk_id)
}

/// A passage of story `story_id` with each renamed speaker label given its name ("Gap-Toothed Girl:" becomes
/// "Gap-Toothed Girl (Idida):"), so an answer asked for the name can give it (2026-10-19, `--no-label-renames`).
/// With `chunk_id` (None under `--no-chunk-renames`) the chunk's own renames apply too ("???:" becomes
/// "??? (Mon3tr):").
pub(crate) fn with_label_names(story_id: &str, chunk_id: Option<&str>, text: &str) -> String {
    let maps: Vec<&BTreeMap<String, String>> = label_renames().get(story_id).into_iter()
        .chain(chunk_id.and_then(chunk_renames)).collect();
    rename_lines(text, &maps)
}

/// `text` with each line whose label one of `maps` renames given its name, the first map that has the label winning.
pub(crate) fn rename_lines(text: &str, maps: &[&BTreeMap<String, String>]) -> String {
    if maps.is_empty() {
        return text.to_owned();
    }
    text.split('\n')
        .map(|l| maps.iter().flat_map(|m| m.iter())
            .find_map(|(label, name)| l.strip_prefix(label.as_str()).and_then(|r| r.strip_prefix(':'))
                .map(|r| format!("{label} ({name}):{r}")))
            .unwrap_or_else(|| l.to_owned()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The gold-set pipeline's names table (`artifacts/goldgen/names.json`), empty (and logged) when missing or unparsable.
pub(crate) fn load_names() -> HashMap<String, Names> {
    trevor::util::load_json_opt(std::path::Path::new("artifacts/goldgen/names.json")).unwrap_or_default()
}

/// A story passage's label: "story: <group>, <story name>" from the names table, or "story: <id>" for a story it lacks.
pub(crate) fn story_label(names: &HashMap<String, Names>, story_id: &str) -> String {
    names.get(story_id).map_or_else(|| format!("story: {story_id}"), |n| format!("story: {}, {}", n.group, n.story))
}

/// The operator a typed unit belongs to, from its heading line: "Outfit: Newsgirl (Amiya)", "Voice lines: Texas",
/// "Module story: Drone Control Module.P (Magallan's module, SUM-X)"; None for IS, enemy and item units.
pub(crate) fn unit_owner(heading: &str) -> Option<String> {
    if let Some(n) = heading.strip_prefix("Voice lines: ") {
        return Some(n.trim().to_owned());
    }
    let open = heading.rfind('(')?;
    let inner = heading[open + 1..].trim_end_matches(')');
    let name = inner.split(['\'', '\u{2019}', ',']).next()?.trim();
    (!name.is_empty()).then(|| name.to_owned())
}

/// The operator a cited operator-file chunk belongs to ("archive_char_338_iris#0001" -> Iris).
pub(crate) fn file_owner(tools: &Tools, id: &str) -> Option<String> {
    id.starts_with("archive_").then(|| tools.source_owner(id.split('#').next().unwrap_or(id))).flatten()
}

/// The operator a typed unit belongs to, from its first line ("Voice lines: X", "Module story: ... (X's module, ...)",
/// "Operator art: X, E2 ...").
pub(crate) fn typed_owner(heading: &str) -> Option<String> {
    if let Some(r) = heading.strip_prefix("Operator art: ") {
        return r.split(',').next().map(|n| n.trim().to_owned());
    }
    unit_owner(heading)
}
