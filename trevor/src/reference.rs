//! Eval references from the Arknights wiki (2026-10-04, Ian: "use the Wiki as a *reference*").
//!
//! Nothing Trevor serves or builds its answers from may come from the wiki: the wiki is an answer key that
//! Trevor's own deductions from the game data are measured against (`scripts/design_infer.py score`,
//! `scripts/deaths.py eval`, `scripts/reading_guide.py score`). The references live under [`DIR`], and this
//! module is the only code in `src/` that names that directory (a test in `tools.rs` checks it). Its one reader,
//! [`wiki_design_basis`], exists for `ask --wiki-legacy`, the kill switch that restores the wiki-based answers of
//! the morning of 2026-10-04 for measurement; the default never calls it.

use std::path::Path;

use serde_json::Value;

/// The eval-reference directory, relative to the crate root.
pub const DIR: &str = "eval/reference";

/// The wiki's design-basis rows (`scripts/design_basis.py`), for `ask --wiki-legacy` only.
#[must_use]
pub fn wiki_design_basis(root: &Path) -> Option<Vec<Value>> {
    let rows: Vec<Value> = crate::util::read_jsonl_opt(&root.join(DIR).join("design_basis.jsonl"))?;
    (!rows.is_empty()).then_some(rows)
}
