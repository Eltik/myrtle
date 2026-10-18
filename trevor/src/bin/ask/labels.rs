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
/// scripts/label_renames.py): story id -> description label -> name. Empty when missing.
fn label_renames() -> &'static HashMap<String, BTreeMap<String, String>> {
    static RENAMES: OnceLock<HashMap<String, BTreeMap<String, String>>> = OnceLock::new();
    RENAMES.get_or_init(|| {
        trevor::util::read_json_opt(std::path::Path::new("artifacts/entities/label_renames.json")).unwrap_or_default()
    })
}

/// A passage of story `story_id` with each renamed speaker label given its name ("Gap-Toothed Girl:" becomes
/// "Gap-Toothed Girl (Idida):"), so an answer asked for the name can give it (2026-10-19, `--no-label-renames`).
pub(crate) fn with_label_names(story_id: &str, text: &str) -> String {
    let Some(m) = label_renames().get(story_id) else { return text.to_owned() };
    text.split('\n')
        .map(|l| m.iter()
            .find_map(|(label, name)| l.strip_prefix(label.as_str()).and_then(|r| r.strip_prefix(':'))
                .map(|r| format!("{label} ({name}):{r}")))
            .unwrap_or_else(|| l.to_owned()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The gold-set pipeline's names table (`artifacts/goldgen/names.json`), empty when missing or unparsable.
pub(crate) fn load_names() -> HashMap<String, Names> {
    std::fs::read("artifacts/goldgen/names.json").ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
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
