//! Tables the unpacker kept from an earlier extract because this one would not decode.
//!
//! When a patch changes a `FlatBuffer` table's layout so it verifies under neither the CN
//! nor the Yostar schema, the unpacker SKIPS it and keeps the previous `.json`, then lists
//! it in `gamedata/.skipped.json` (see `assets/unpacker/src/export/gamedata.rs`). The
//! table still parses here, so `table_warnings` never sees it: on 2026-10-14 EN
//! `activity_table` went stale this way and the release planner missed a new event for
//! days with a clean `/health`. This file is the only place that staleness is visible.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// One table the last extract skipped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StaleTable {
    /// The table's file stem, e.g. `activity_table`.
    pub table: String,
    /// RFC 3339 UTC of the first extract that skipped it; every extract since has too.
    pub since: String,
    /// The resource version whose bundles failed to verify (the downloader's
    /// `.downloaded_version`); absent when the input was not fetched by it.
    #[serde(default)]
    pub res_version: Option<String>,
}

#[derive(Deserialize)]
struct SkippedFile {
    tables: Vec<StaleTable>,
}

/// Read `<gamedata_dir>/.skipped.json`, where `gamedata_dir` holds `excel/`.
///
/// Missing is normal (an extract older than the file, or a tree never extracted
/// here) and means nothing is known to be stale. Unparseable is a warning and also
/// reads as nothing stale: gamedata load must never fail over a diagnostic file.
pub fn read_stale_tables(gamedata_dir: &Path) -> Vec<StaleTable> {
    let path = gamedata_dir.join(".skipped.json");
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(e) => {
            tracing::warn!("{}: {e}, treating no table as stale", path.display());
            return Vec::new();
        }
    };
    match serde_json::from_slice::<SkippedFile>(&bytes) {
        Ok(file) => {
            for t in &file.tables {
                tracing::warn!(
                    table = %t.table,
                    since = %t.since,
                    "stale gamedata table: skipped by the unpacker, previous extract served"
                );
            }
            file.tables
        }
        Err(e) => {
            tracing::warn!("{}: {e}, treating no table as stale", path.display());
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("myrtle_stale_{name}_{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn reads_the_unpacker_shape() {
        let d = dir("shape");
        // Written by the unpacker: `previousOutput` is ignored, `resVersion` may be absent.
        std::fs::write(
            d.join(".skipped.json"),
            r#"{"tables":[
                {"table":"activity_table","since":"2026-10-14T05:00:00Z","resVersion":"26-10-03-16-06-39_0a6acf","previousOutput":true},
                {"table":"building_data","since":"2026-10-15T00:00:00Z","previousOutput":false}
            ]}"#,
        )
        .unwrap();
        let stale = read_stale_tables(&d);
        assert_eq!(
            stale,
            vec![
                StaleTable {
                    table: "activity_table".into(),
                    since: "2026-10-14T05:00:00Z".into(),
                    res_version: Some("26-10-03-16-06-39_0a6acf".into()),
                },
                StaleTable {
                    table: "building_data".into(),
                    since: "2026-10-15T00:00:00Z".into(),
                    res_version: None,
                },
            ]
        );
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn missing_or_unparseable_is_nothing_stale() {
        let d = dir("bad");
        assert!(read_stale_tables(&d).is_empty());
        std::fs::write(d.join(".skipped.json"), "{not json").unwrap();
        assert!(read_stale_tables(&d).is_empty());
        std::fs::remove_dir_all(&d).unwrap();
    }
}
