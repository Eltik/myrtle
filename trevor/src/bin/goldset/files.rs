//! The work directory's JSONL files: resumable appends and whole-file rewrites.

use std::collections::HashMap;
use std::io::{BufRead as _, Write as _};
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Serialize, de::DeserializeOwned};
use trevor::goldgen::Generated;

/// Every row of a JSONL file; a missing file has none, blank lines are skipped, and a line that does not parse is an
/// error naming its line.
pub(crate) fn read_jsonl<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    let Ok(f) = std::fs::File::open(path) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for (i, line) in std::io::BufReader::new(f).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        out.push(
            serde_json::from_str(&line)
                .with_context(|| format!("{} line {}", path.display(), i + 1))?,
        );
    }
    Ok(out)
}

/// Append `rows` to `path`, one JSON line each (the resumable stages).
pub(crate) fn append<T: Serialize>(path: &Path, rows: &[T]) -> Result<()> {
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    for r in rows {
        writeln!(f, "{}", serde_json::to_string(r)?)?;
    }
    Ok(())
}

/// Replace `path` with `rows`, written first to the same name with the extension `tmp`.
pub(crate) fn write_jsonl<T: Serialize>(path: &Path, rows: &[T]) -> Result<()> {
    trevor::util::write_jsonl_atomic(path, "tmp", rows)
}

/// `generated.jsonl` of the work directory by generation id.
pub(crate) fn generated_by_id(work: &Path) -> Result<HashMap<String, Generated>> {
    Ok(read_jsonl::<Generated>(&work.join("generated.jsonl"))?.into_iter().map(|g| (g.id.clone(), g)).collect())
}
