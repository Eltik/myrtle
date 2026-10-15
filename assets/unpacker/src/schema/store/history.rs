//! `assets/schemas/history/<stem>/<label>.fbs`: every distinct version of
//! every table's schema the store has ever seen, one file per version.
//!
//! "Distinct" is judged after [`normalize_fbs`], so two files that differ only
//! in comments or whitespace are one version. A label is
//! `<YYYY-MM-DD>[T<hhmmss>]_<version-or-sha7>`, built so that LEXICAL order is
//! chronological: [`list`] sorts labels descending and that is newest first.
//! The time part only appears where one day holds several versions of one
//! table (the seed from `OpenArknightsFBS` has 67 such files, all from days
//! with two upstream commits).
//!
//! The store is self-contained: nothing here reads a git repository. The seed
//! was imported once (see `assets/schemas/README.md`); since then
//! `schema-sync --generated` adds each newly generated CN schema through
//! [`add`].

use std::path::{Path, PathBuf};

use super::normalize_fbs;

/// One stored version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub label: String,
    pub path: PathBuf,
}

impl Entry {
    /// The store-relative path the lock records as a `source`.
    #[must_use]
    pub fn source(&self, stem: &str) -> String {
        source(stem, &self.label)
    }
}

/// `history/<stem>/<label>.fbs`, the form a lock `source` takes.
#[must_use]
pub fn source(stem: &str, label: &str) -> String {
    format!("history/{stem}/{label}.fbs")
}

/// The directory holding one table's versions.
#[must_use]
pub fn dir(store: &Path, stem: &str) -> PathBuf {
    store.join("history").join(stem)
}

/// The label of a version first seen on `date` (`YYYY-MM-DD`) as `version`.
#[must_use]
pub fn label(date: &str, version: &str) -> String {
    format!("{date}_{version}")
}

/// Labels newest first: lexical descending.
pub fn sort_newest_first(labels: &mut [String]) {
    labels.sort_by(|a, b| b.cmp(a));
}

/// One table's versions, newest first. Empty when the table has none.
#[must_use]
pub fn list(store: &Path, stem: &str) -> Vec<Entry> {
    let d = dir(store, stem);
    let Ok(read) = std::fs::read_dir(&d) else {
        return Vec::new();
    };
    let mut labels: Vec<String> = read
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "fbs"))
        .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(str::to_string))
        .collect();
    sort_newest_first(&mut labels);
    labels
        .into_iter()
        .map(|label| Entry {
            path: d.join(format!("{label}.fbs")),
            label,
        })
        .collect()
}

/// The stored version whose normalised text equals `bytes`, if any.
#[must_use]
pub fn find(store: &Path, stem: &str, bytes: &[u8]) -> Option<Entry> {
    let want = normalize_fbs(&String::from_utf8_lossy(bytes));
    list(store, stem).into_iter().find(|e| {
        std::fs::read(&e.path).is_ok_and(|b| normalize_fbs(&String::from_utf8_lossy(&b)) == want)
    })
}

/// The label `add` gives a new version: `base` when it is free and sorts after
/// every existing label, else `base_2`, `base_3`, ... (each of which sorts
/// after `base`). A `base` that sorts before an existing label (a clock behind
/// the newest stored date) keeps `base` with a numeric suffix; lexical order
/// then cannot be chronological and the caller is warned.
#[must_use]
pub fn next_label(existing: &[String], base: &str) -> (String, bool) {
    let newest = existing.iter().max();
    let free = |l: &str| !existing.iter().any(|e| e == l);
    let mut fallback = None;
    for n in 1..1000 {
        let l = if n == 1 {
            base.to_string()
        } else {
            format!("{base}_{n}")
        };
        if !free(&l) {
            continue;
        }
        if newest.is_none_or(|top| l.as_str() > top.as_str()) {
            return (l, true);
        }
        fallback.get_or_insert(l);
    }
    (fallback.unwrap_or_else(|| format!("{base}_1000")), false)
}

/// Stores `bytes` as a new version of `stem` labelled `base` (see
/// [`next_label`]) unless its normalised text is already stored. Returns the
/// label written, `None` when the version was already there.
pub fn add(store: &Path, stem: &str, base: &str, bytes: &[u8]) -> anyhow::Result<Option<String>> {
    if find(store, stem, bytes).is_some() {
        return Ok(None);
    }
    let existing: Vec<String> = list(store, stem).into_iter().map(|e| e.label).collect();
    let (label, ordered) = next_label(&existing, base);
    if !ordered {
        eprintln!(
            "warning: history/{stem}: {label} does not sort after the newest stored label; \
             lexical order is no longer chronological for this table"
        );
    }
    let d = dir(store, stem);
    std::fs::create_dir_all(&d)?;
    std::fs::write(d.join(format!("{label}.fbs")), bytes)?;
    Ok(Some(label))
}

/// Removes every version whose normalised text an OLDER version already has
/// (the first label to carry a schema is the one kept). Returns the labels
/// removed.
pub fn dedupe(store: &Path, stem: &str) -> anyhow::Result<Vec<String>> {
    let mut entries = list(store, stem);
    entries.reverse();
    let mut seen = std::collections::HashSet::new();
    let mut removed = Vec::new();
    for e in entries {
        let text = normalize_fbs(&String::from_utf8_lossy(&std::fs::read(&e.path)?));
        if !seen.insert(text) {
            std::fs::remove_file(&e.path)?;
            removed.push(e.label);
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| (*x).to_string()).collect()
    }

    #[test]
    fn labels_sort_newest_first() {
        let mut l = s(&[
            "2023-06-08T102625_2.0.11",
            "2026-09-04_2.7.71",
            "2025-04-14_ab97499",
            "2026-10-09_2.7.81",
            "2023-06-08T102953_2.0.11",
            "2023-07-06_2.0.40",
        ]);
        sort_newest_first(&mut l);
        assert_eq!(
            l,
            s(&[
                "2026-10-09_2.7.81",
                "2026-09-04_2.7.71",
                "2025-04-14_ab97499",
                "2023-07-06_2.0.40",
                "2023-06-08T102953_2.0.11",
                "2023-06-08T102625_2.0.11",
            ])
        );
    }

    #[test]
    fn next_label_stays_newest() {
        let existing = s(&["2026-09-04_2.7.71", "2026-10-09_2.7.81"]);
        assert_eq!(
            next_label(&existing, "2026-10-15_2.7.81"),
            ("2026-10-15_2.7.81".to_string(), true)
        );
        // Same day, same version, different text: a suffix that sorts after.
        let existing = s(&["2026-10-15_2.7.81"]);
        let (l, ok) = next_label(&existing, "2026-10-15_2.7.81");
        assert_eq!((l.as_str(), ok), ("2026-10-15_2.7.81_2", true));
        assert!(l.as_str() > "2026-10-15_2.7.81");
        // A clock behind the newest label cannot sort last.
        let existing = s(&["2027-01-01_2.8.01"]);
        assert!(!next_label(&existing, "2026-10-15_2.7.81").1);
    }

    #[test]
    fn add_list_find_dedupe() {
        let store = std::env::temp_dir().join(format!("history-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&store);
        let a = b"table A { x: int; }\nroot_type A;\n";
        let a_commented = b"// header\ntable A {\n  x: int;\n}\nroot_type A;\n";
        let b = b"table A { x: int; y: int; }\nroot_type A;\n";
        assert_eq!(
            add(&store, "t", "2026-01-01_1.0", a).unwrap().as_deref(),
            Some("2026-01-01_1.0")
        );
        // Same schema after normalisation: not stored again.
        assert_eq!(
            add(&store, "t", "2026-02-01_1.1", a_commented).unwrap(),
            None
        );
        assert_eq!(
            add(&store, "t", "2026-03-01_1.2", b).unwrap().as_deref(),
            Some("2026-03-01_1.2")
        );
        let labels: Vec<String> = list(&store, "t").into_iter().map(|e| e.label).collect();
        assert_eq!(labels, s(&["2026-03-01_1.2", "2026-01-01_1.0"]));
        assert_eq!(
            find(&store, "t", a_commented).unwrap().label,
            "2026-01-01_1.0"
        );
        // A duplicate written behind add's back: the newer copy goes.
        std::fs::write(dir(&store, "t").join("2026-04-01_1.3.fbs"), a_commented).unwrap();
        assert_eq!(dedupe(&store, "t").unwrap(), s(&["2026-04-01_1.3"]));
        assert_eq!(list(&store, "t").len(), 2);
        std::fs::remove_dir_all(&store).unwrap();
    }
}
