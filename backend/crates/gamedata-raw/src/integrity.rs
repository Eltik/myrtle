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

pub fn check(raw: &RawGameData) -> IntegrityReport {
    // TODO: populated-table checks (operators, items, skills, voice lines) once
    // the table groups have fields; they push into `warnings`.
    IntegrityReport {
        skipped: raw.skipped.clone(),
        warnings: Vec::new(),
    }
}
