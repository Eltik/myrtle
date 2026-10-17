//! The chunk kinds (`group_id`s) of the corpora, grouped by how ask treats them.

/// Character sources: operator files and Trevor's character dossiers.
pub(crate) const CHARACTER_GROUPS: [&str; 2] = ["archive", "profile"];

/// P4 unit kinds: the typed sources and Trevor's topic summaries, which a lore query leaves out.
pub(crate) const TYPED: [&str; 7] = ["module", "voice", "skin", "is", "enemy", "item", "topic"];

/// Chunk kinds Trevor wrote itself: never a term's evidence, and left out of a `passages` answer, which serves
/// source text only.
pub(crate) const GENERATED_KINDS: [&str; 3] = ["profile", "summary", "topic"];

/// Whether `group_id` is one of the [`GENERATED_KINDS`].
pub(crate) fn is_generated(group_id: &str) -> bool {
    GENERATED_KINDS.contains(&group_id)
}

/// Chunk kinds that hold records and files, which define a term more often than a scene does.
pub(crate) const RECORD_KINDS: [&str; 7] = ["archive", "gametext", "is", "module", "item", "enemy", "skin"];

/// The kinds whose text describes an object (outfit, item, module, enemy) and so names lore it does not explain.
/// Operator files and voice lines keep plain retrieval.
pub(crate) const LORE_KINDS: [&str; 4] = ["skin", "item", "module", "enemy"];

/// The unit kinds the typed fallback tries, in no order: the best-ranked unit of the named operator decides the kind.
pub(crate) const FALLBACK_KINDS: [&str; 3] = ["voice", "module", "art"];
