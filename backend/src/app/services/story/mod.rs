//! The story reader's service: the Archives library, one parsed script, one
//! group's art and one group's archive.
//!
//! The index is derived from `story_review_table` (groups and stories),
//! `handbook_info_table` (operator records), `zone_table` + `chapter_table`
//! (main chapter names), `stage_table.Storylines` (the shelves) and the asset
//! index (covers). It is built once per loaded game data and cached.
//!
//! `category` is DERIVED, not guessed: `main` is `EntryType: MAINLINE`,
//! `side` is `ACTIVITY` with `ActType: ACTIVITY_STORY`, `vignette` is
//! `MINI_ACTIVITY` or `ActType: MINI_STORY`, `record` is an `EntryType: NONE`
//! group whose scripts sit under `obt/memory/` (all 364 such groups on EN),
//! `is` and `reclamation` are `obt/rogue*` and `obt/sandbox*` script
//! prefixes (0 groups on EN: those scripts exist on disk but the table does
//! not list them), and `sideContent` is whatever is left (0 on EN).
//!
//! The cut, and what each part owns:
//!
//! - [`dto`]: the wire types, and nothing that derives them.
//! - [`index`]: the build, and its cost in the library's own counts.
//! - [`cache`]: one slot per server and the single flight that guards it.
//! - [`archive`]: the game's own "from the archive" screen for one group.
//! - [`illustrations`]: the distinct names a group's scripts reference.
//! - [`music`]: the theme a group plays over its Archives entry.
//! - [`art`]: every `AssetIndex` lookup that becomes a served URL.
//! - [`script`]: loading and parsing one story.
//!
//! The measured census behind every number quoted in here, and the wire
//! contract itself, are in `docs/story-reader.md`, sections "1. What is true
//! about the data" and "2. Wire contract". Numbers stay in the code only where
//! they document a DECISION the code makes; the narrative history stays in the
//! doc.

mod archive;
mod art;
pub mod cache;
mod dto;
mod illustrations;
mod index;
mod music;
mod script;

pub use archive::{build_archives, get_group_archive};
pub use cache::{ServerCache, cached_index, get_story_index, index_builds, spawn_warm};
pub use dto::*;
pub use illustrations::{
    GroupRefs, NameRefs, SpriteRefs, get_group_illustrations, group_illustrations,
};
pub use index::{StoryIndexCache, StoryRef, build_index};
pub use script::{get_story, load_and_parse};

/// A game table's string as an optional field: trimmed, and absent when it is
/// empty. The archive and music tables write "no value" as an empty string or
/// a single space (101 of the 208 EN `Musics` rows write a single space), so
/// every optional field off those tables goes through this.
pub(super) fn opt(s: &str) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_owned())
}
