//! Story pages: the library, one script, a group's art and archive, and the galleries derived from them

pub mod community;
mod files;
mod gallery;
pub mod progress;
mod sprites;

use std::sync::Arc;

use myrtle_engine_story::dto::{StoryArchive, StoryIllustrations, StoryIndex};
use myrtle_engine_story::script::{ScriptError, StoryScript};
use myrtle_engine_story::sprites::SpriteGallery;
use myrtle_gamedata_raw::DirSource;
use myrtle_types::Server;
use tokio::sync::OnceCell;

pub use files::{list_story_tree, modified, touch_if_old, write_atomically};
pub use gallery::{
    StoryArt, ensure_picture, ensure_story_art, ensure_variant, gallery_shared, get_gallery,
    get_story_art, story_art_census, story_art_known, story_art_shared, with_default_fallback,
};
pub use sprites::{
    ensure_head, ensure_variant_thumb, get_story_sprite, get_story_sprites, get_variant_thumb,
    sprites_shared,
};

use crate::snapshots::{Snapshot, Story};
use crate::{AppContext, AppError};

/// Galleries derived from one [`Story`], each built once under its own gate and dropped with it
#[derive(Default)]
pub struct Derived {
    gallery: OnceCell<Arc<myrtle_engine_story::dto::StoryGallery>>,
    art: OnceCell<Arc<StoryArt>>,
    sprites: OnceCell<Arc<SpriteGallery>>,
}

/// `server`'s snapshot with its story library built off the async workers
pub async fn snapshot(ctx: &AppContext, server: Server) -> Result<Arc<Snapshot>, AppError> {
    let snapshot = ctx.snapshots.require(server)?;
    if !snapshot.story_built() {
        let _gate = snapshot.story_gate().await;
        if !snapshot.story_built() {
            let held = Arc::clone(&snapshot);
            ctx.cpu
                .run("story_index", move || held.story().map(|_| ()))
                .await??;
        }
    }
    Ok(snapshot)
}

/// Builds the library after a load, so the first reader does not pay for it
pub async fn warm(ctx: &AppContext, server: Server) {
    let started = std::time::Instant::now();
    match snapshot(ctx, server)
        .await
        .and_then(|s| s.story().map(|_| ()))
    {
        Ok(()) => tracing::info!(
            server = server.as_str(),
            warm_ms = started.elapsed().as_millis(),
            "story library warmed"
        ),
        Err(e) => tracing::warn!(server = server.as_str(), error = %e, "story library warm failed"),
    }
}

/// One derived value of `server`'s story, built on the CPU pool by one caller while the rest wait
async fn derived<T, F>(
    ctx: &AppContext,
    server: Server,
    label: &'static str,
    cell: fn(&Derived) -> &OnceCell<Arc<T>>,
    build: F,
) -> Result<Arc<T>, AppError>
where
    T: Send + Sync + 'static,
    F: FnOnce(&Snapshot, &Story) -> T + Send + 'static,
{
    let snapshot = snapshot(ctx, server).await?;
    let story = snapshot.story()?;
    let held = Arc::clone(&snapshot);
    let built = cell(&story.derived)
        .get_or_try_init(|| async move {
            ctx.cpu
                .run(label, move || {
                    let story = held.story()?;
                    Ok::<_, AppError>(Arc::new(build(&held, story)))
                })
                .await?
        })
        .await?;
    Ok(Arc::clone(built))
}

pub async fn get_story_index(ctx: &AppContext, server: Server) -> Result<StoryIndex, AppError> {
    Ok(snapshot(ctx, server).await?.story()?.library.index.clone())
}

/// One parsed script with its synopsis; `None` for an id the library does not list
pub async fn get_story(
    ctx: &AppContext,
    server: Server,
    story_id: &str,
) -> Result<Option<StoryScript>, AppError> {
    let snapshot = snapshot(ctx, server).await?;
    if !snapshot.story()?.library.lookup.contains_key(story_id) {
        return Ok(None);
    }
    let text = snapshot.story_dirs().map_or_else(
        || ctx.snapshots.server_dir(server),
        |(_, t)| t.to_path_buf(),
    );
    let id = story_id.to_owned();
    ctx.cpu
        .run("story_parse", move || {
            let story = snapshot.story()?;
            story
                .library
                .load_story(&DirSource { root: text }, &story.assets, &id)
                .map_err(|e| match e {
                    ScriptError::Missing { .. } | ScriptError::NotAScript { .. } => {
                        AppError::NotFound(Some(format!("story `{id}`: {e}")))
                    }
                    ScriptError::Io { .. } => AppError::internal(e),
                })
        })
        .await?
}

fn listed(story: &Story, group_id: &str) -> Result<(), AppError> {
    if story.library.illustrations.contains_key(group_id) {
        Ok(())
    } else {
        Err(AppError::NotFound(Some(format!(
            "story group `{group_id}` is not in the library"
        ))))
    }
}

/// A group's (or an operator's) distinct backgrounds, images and sprites with their URLs
pub async fn get_group_illustrations(
    ctx: &AppContext,
    server: Server,
    group_id: &str,
) -> Result<StoryIllustrations, AppError> {
    let snapshot = snapshot(ctx, server).await?;
    listed(snapshot.story()?, group_id)?;
    let id = group_id.to_owned();
    ctx.cpu
        .run("story_illustrations", move || {
            let story = snapshot.story()?;
            story
                .library
                .illustrations(&story.assets, &id)
                .ok_or(AppError::NotFound(None))
        })
        .await?
}

/// The group's "from the archive" screen, empty for a listed group the archive table skips
pub async fn get_group_archive(
    ctx: &AppContext,
    server: Server,
    group_id: &str,
) -> Result<StoryArchive, AppError> {
    let snapshot = snapshot(ctx, server).await?;
    let story = snapshot.story()?;
    listed(story, group_id)?;
    Ok(story
        .library
        .archives
        .get(group_id)
        .cloned()
        .unwrap_or_else(|| StoryArchive {
            group_id: group_id.to_owned(),
            sections: Vec::new(),
        }))
}
