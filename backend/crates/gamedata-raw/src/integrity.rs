use serde::{Deserialize, Serialize};

use crate::RawGameData;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedTable {
    /// File stem, eg. `activity_table`
    pub table: String,
    /// RFC 3339 UTC of first extract that skipped table
    pub since: String,
    /// Resource version
    #[serde(default)]
    pub res_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IntegrityReport {
    pub skipped: Vec<SkippedTable>,
    pub warnings: Vec<String>,
}

impl IntegrityReport {
    pub const fn is_clean(&self) -> bool {
        self.skipped.is_empty() && self.warnings.is_empty()
    }
}

/// Load warnings, plus one for each core table that parsed but has no rows
pub fn check(raw: &RawGameData) -> IntegrityReport {
    let populated = [
        ("character_table", raw.character.characters.characters.len()),
        ("item_table", raw.item.items.items.len()),
        ("skill_table", raw.character.skills.skills.len()),
        ("charword_table", raw.audio.voices.char_words.len()),
    ];
    let mut warnings = raw.warnings.clone();
    for (table, rows) in populated {
        let prefix = format!("table {table}:");
        if rows == 0 && !warnings.iter().any(|w| w.starts_with(&prefix)) {
            warnings.push(format!("table {table}: parsed with no rows"));
        }
    }
    IntegrityReport {
        skipped: raw.skipped.clone(),
        warnings,
    }
}
