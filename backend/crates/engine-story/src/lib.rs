pub mod archive;
pub mod assets;
pub mod dto;
pub mod gallery;
pub mod index;
pub mod parser;
pub mod script;
pub mod speakers;
pub mod sprites;
pub mod story_art;
pub mod thumbs;
pub mod verdict;

pub use assets::StoryAssetIndex;
pub use index::{StoryLibrary, StoryRef, build_index};
pub use parser::StoryCommand;

pub(crate) fn opt(s: &str) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_owned())
}
