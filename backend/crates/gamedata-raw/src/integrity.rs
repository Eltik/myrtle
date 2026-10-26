use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedTable {
    /// File stem, eg. `activity_table`
    pub table: String,
    /// RFC 3339 UTC of first extract that skipped table
    pub since: String,
    /// Resource version
    pub res_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IntegrityReport {
    pub skipped: Vec<SkippedTable>,
    pub warnings: Vec<String>,
}

pub fn check(raw: &crate::RawGameData) -> IntegrityReport {
    let _ = raw;
    todo!()
}
