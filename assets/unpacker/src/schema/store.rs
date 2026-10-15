//! The on-disk schema store, `assets/schemas/{cn,yostar,history}`, and the
//! helpers both `generate-fbs` and `schema-sync` need to agree on.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub mod history;

/// `assets/schemas`, resolved from this crate's manifest directory.
#[must_use]
pub fn default_store_dir() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().unwrap_or(manifest).join("schemas")
}

/// A `.fbs` reduced to the part that can change how flatc generates code:
/// comments gone, every run of whitespace collapsed to one space.
///
/// The old Yostar repo pinned each table to the CN commit it was forked from
/// and recorded that as a leading `// https://github.com/MooncellWiki/...`
/// comment, so a comment-sensitive comparison called all 57 files "different"
/// when most were byte-identical CN copies. The sync's bfbs cache is keyed on
/// this form for the same reason.
#[must_use]
pub fn normalize_fbs(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            out.push(' ');
        } else if bytes[i].is_ascii_whitespace() {
            if !out.ends_with(' ') {
                out.push(' ');
            }
            i += 1;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out.trim().to_string()
}

/// The `.fbs` stems in one store directory, sorted.
#[must_use]
pub fn list_stems(dir: &Path) -> BTreeSet<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return BTreeSet::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "fbs"))
        .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(str::to_string))
        .collect()
}

/// Directory-routed tables: every `FlatBuffer` under the prefix is one buffer
/// of the named table, whatever its file is called. These are also the tables
/// the sync SAMPLES rather than verifying whole (see `--levels-sample`).
pub const DIRECTORY_TABLES: &[(&str, &str)] = &[
    ("gamedata/levels/", "prts___levels"),
    ("gamedata/bakemuzzledata/", "bake_muzzle_data"),
];

/// The store table a gamedata buffer belongs to, from its manifest path.
///
/// An exact file-stem match wins (`gamedata/levels/enemydata/enemy_database`
/// is `enemy_database`, not a level); a directory-routed table comes second.
/// `None` for a buffer no stored schema claims.
#[must_use]
pub fn table_for_path(path: &str, tables: &BTreeSet<String>) -> Option<String> {
    let lower = path.to_lowercase();
    let file = lower.rsplit('/').next().unwrap_or(&lower);
    let stem = file.split('.').next().unwrap_or(file);
    if tables.contains(stem) {
        return Some(stem.to_string());
    }
    DIRECTORY_TABLES
        .iter()
        .find(|(prefix, table)| lower.starts_with(prefix) && tables.contains(*table))
        .map(|(_, table)| (*table).to_string())
}

/// Whether `table` is fed by a whole directory and is therefore sampled.
#[must_use]
pub fn is_directory_table(table: &str) -> bool {
    DIRECTORY_TABLES.iter().any(|(_, t)| *t == table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_drops_comments_and_whitespace() {
        let a = "// https://example/commit/abc\ntable A {\n  x: int;  // note\n}\n";
        let b = "table A { x: int; }";
        assert_eq!(normalize_fbs(a), normalize_fbs(b));
        assert_eq!(normalize_fbs("/* c */table  B{}"), "table B{}");
    }

    #[test]
    fn routing_prefers_exact_stem() {
        let tables: BTreeSet<String> = ["enemy_database", "prts___levels", "level_script_table"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        assert_eq!(
            table_for_path("gamedata/levels/enemydata/enemy_database.bytes", &tables).as_deref(),
            Some("enemy_database")
        );
        assert_eq!(
            table_for_path("gamedata/levels/obt/main/level_main_01-01.bytes", &tables).as_deref(),
            Some("prts___levels")
        );
        assert_eq!(
            table_for_path("gamedata/battle/level_script_table.bytes", &tables).as_deref(),
            Some("level_script_table")
        );
        assert_eq!(table_for_path("gamedata/excel/other.bytes", &tables), None);
    }
}
