//! Guards against the quietest failure mode this backend has.
//!
//! `load_table_or_warn` catches a deserialization error, logs one line, and
//! substitutes `T::default()`. The service then serves an empty table as if it
//! were real data: no error, a passing health check, a passing smoke test. In
//! September 2026 six CN items carrying `ClassifyType: "MEMENTO"` — a value the
//! `ItemClass` enum did not name — failed the whole `item_table`, so all 1,556
//! CN items lost their names and icons. It was found by a user noticing a broken
//! image, not by anything in CI.
//!
//! Hypergryph ships new enum values on CN first, so CN is the canary. Point
//! `GAME_DATA_DIR`/`ASSETS_DIR` at a CN extract in a second CI job to get the
//! warning before it reaches production.

mod common;

/// No table may fall back to its default.
///
/// A failure here names the table and the serde error, which is normally enough
/// to spot the culprit — most often an enum missing a variant. The fix is
/// usually `#[serde(other)] Unknown` on that enum rather than adding the one new
/// variant, since the next unnamed value would break it again.
#[test]
fn no_table_falls_back_to_default() {
    let gd = common::load_game_data();
    assert!(
        gd.table_warnings.is_empty(),
        "{} table(s) failed to load and were replaced by empty defaults:\n  {}",
        gd.table_warnings.len(),
        gd.table_warnings.join("\n  ")
    );
}

/// A table can also deserialize "successfully" into nothing. These four carry the
/// site: an empty one is a visible outage, so assert they have real content
/// rather than merely having parsed.
#[test]
fn core_tables_are_populated() {
    let gd = common::load_game_data();
    assert!(
        !gd.operators.is_empty(),
        "character_table produced no operators"
    );
    assert!(
        !gd.materials.items.is_empty(),
        "item_table produced no items"
    );
    assert!(!gd.skills.is_empty(), "skill_table produced no skills");
    assert!(
        !gd.voices.char_words.is_empty(),
        "charword_table produced no voice lines"
    );
}
