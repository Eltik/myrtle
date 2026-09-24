//! What a group's scripts DRAW: the distinct backgrounds, CGs and sprite
//! folders they reference, and the stories that reference each.
//!
//! The cache holds NAMES, never URLs. Every name is resolved per request
//! through the same [`StoryAssetIndex`] `GET /story/{id}` resolves through, so
//! a re-extract that moves a file cannot leave a stale URL behind.
//!
//! Scale: the builder runs once per story at index build, 2,254 calls over
//! 1,887 distinct scripts on EN, and the per-request half is a walk over one
//! group's rows. Both are linear in the names a group writes; nothing here is
//! quadratic in the library.

use std::collections::HashMap;

use super::cache::cached_index;
use super::dto::{IllustrationItem, SpriteItem, StoryIllustrations};
use super::index::StoryIndexCache;
use crate::app::{cpu, error::ApiError, state::AppState};
use crate::core::hypergryph::constants::Server;
use crate::core::story::{ScriptFacts, StoryAssetIndex};

/// One name and the group's stories that write it, cached unresolved.
#[derive(Debug, Clone, Default)]
pub struct NameRefs {
    pub name: String,
    pub story_ids: Vec<String>,
}

/// One sprite folder, the stories that write it, and the distinct face keys.
#[derive(Debug, Clone, Default)]
pub struct SpriteRefs {
    pub base: String,
    pub story_ids: Vec<String>,
    pub faces: Vec<String>,
}

/// A group's distinct references, held in the cache as NAMES. The URLs are
/// resolved per request through the same [`StoryAssetIndex`] the reader uses,
/// so a re-extract that moves a file cannot leave a stale URL in the cache.
#[derive(Debug, Clone, Default)]
pub struct GroupRefs {
    pub backgrounds: Vec<NameRefs>,
    pub images: Vec<NameRefs>,
    pub sprites: Vec<SpriteRefs>,
}

impl GroupRefs {
    pub(super) fn illustration_count(&self) -> u32 {
        u32::try_from(self.backgrounds.len() + self.images.len()).unwrap_or(u32::MAX)
    }

    pub(super) fn sprite_count(&self) -> u32 {
        u32::try_from(self.sprites.len()).unwrap_or(u32::MAX)
    }
}

/// Accumulates one group's references over its stories, in the order the
/// group lists them. A story is added once, so the last story id on a row is
/// the dedupe test.
#[derive(Debug, Default)]
pub(super) struct GroupRefsBuilder {
    refs: GroupRefs,
    background_at: HashMap<String, usize>,
    image_at: HashMap<String, usize>,
    sprite_at: HashMap<String, usize>,
}

fn push_name(list: &mut Vec<NameRefs>, at: &mut HashMap<String, usize>, name: &str, story: &str) {
    let i = at.get(name).copied().unwrap_or_else(|| {
        list.push(NameRefs {
            name: name.to_owned(),
            story_ids: Vec::new(),
        });
        at.insert(name.to_owned(), list.len() - 1);
        list.len() - 1
    });
    let ids = &mut list[i].story_ids;
    if ids.last().map(String::as_str) != Some(story) {
        ids.push(story.to_owned());
    }
}

impl GroupRefsBuilder {
    /// Fold one story's facts in, in the order the group lists its stories.
    pub(super) fn add(&mut self, story_id: &str, facts: &ScriptFacts) {
        for name in &facts.backgrounds {
            push_name(
                &mut self.refs.backgrounds,
                &mut self.background_at,
                name,
                story_id,
            );
        }
        for name in &facts.images {
            push_name(&mut self.refs.images, &mut self.image_at, name, story_id);
        }
        for sprite in &facts.sprites {
            let i = self
                .sprite_at
                .get(&sprite.base)
                .copied()
                .unwrap_or_else(|| {
                    self.refs.sprites.push(SpriteRefs {
                        base: sprite.base.clone(),
                        ..SpriteRefs::default()
                    });
                    self.sprite_at
                        .insert(sprite.base.clone(), self.refs.sprites.len() - 1);
                    self.refs.sprites.len() - 1
                });
            let row = &mut self.refs.sprites[i];
            if row.story_ids.last().map(String::as_str) != Some(story_id) {
                row.story_ids.push(story_id.to_owned());
            }
            for face in &sprite.faces {
                if !row.faces.contains(face) {
                    row.faces.push(face.clone());
                }
            }
        }
    }

    /// The accumulated references, the scratch indices dropped.
    pub(super) fn finish(self) -> GroupRefs {
        self.refs
    }
}

/// Turn one group's cached names into the wire shape, resolving each through
/// the same index `GET /story/{id}` resolves through. `None` when the id is
/// neither a group nor an operator with records.
#[must_use]
pub fn group_illustrations(
    cache: &StoryIndexCache,
    assets: &StoryAssetIndex,
    group_id: &str,
) -> Option<StoryIllustrations> {
    let refs = cache.illustrations.get(group_id)?;
    Some(StoryIllustrations {
        group_id: group_id.to_owned(),
        backgrounds: refs
            .backgrounds
            .iter()
            .map(|r| IllustrationItem {
                name: r.name.clone(),
                url: assets.resolve_background(&r.name).map(|(u, _)| u),
                story_ids: r.story_ids.clone(),
            })
            .collect(),
        images: refs
            .images
            .iter()
            .map(|r| IllustrationItem {
                name: r.name.clone(),
                url: assets.resolve_image(&r.name).map(|(u, _)| u),
                story_ids: r.story_ids.clone(),
            })
            .collect(),
        sprites: refs
            .sprites
            .iter()
            .map(|r| SpriteItem {
                base: r.base.clone(),
                // A bare base resolves to face 1 of body `$1`, the rule the
                // client reads for a missing index.
                body_url: assets.resolve_character(&r.base).map(|c| c.body_url),
                story_ids: r.story_ids.clone(),
                faces: u32::try_from(r.faces.len()).unwrap_or(u32::MAX),
            })
            .collect(),
    })
}

/// Every illustration a group references, off the cached facts: no script is
/// read and none is re-parsed. `group_id` is a `StoryGroup.id` or an operator
/// `charId`, because an operator's records are split over one or two story
/// sets and so have no single group id of their own.
pub async fn get_group_illustrations(
    state: &AppState,
    server: Server,
    group_id: &str,
) -> Result<StoryIllustrations, ApiError> {
    let cache = cached_index(state, server).await?;
    if !cache.illustrations.contains_key(group_id) {
        return Err(ApiError::NotFoundMessage(format!(
            "story group `{group_id}` is not in the library"
        )));
    }
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let assets_dir = std::path::PathBuf::from(&server_data.assets_dir);
    let live_assets = server_data.asset_index.load_full();
    let id = group_id.to_owned();
    cpu::run("story_illustrations", move || {
        let assets = StoryAssetIndex::for_dir(&assets_dir, &live_assets);
        group_illustrations(&cache, &assets, &id).ok_or(ApiError::NotFound)
    })
    .await?
}
