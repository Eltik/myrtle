//! Loading and parsing one story (`GET /story/{id}`). Uncached: the longest EN
//! script parses in under 10 ms (2026-09-21). Runs under [`cpu::run`] because the
//! first call on a cold process builds the asset index (~90k files).

use std::sync::Arc;

use super::cache::{StoryTrees, cached_index};
use super::index::StoryRef;
use crate::app::{cpu, error::ApiError, state::AppState};
use crate::core::gamedata::assets::AssetIndex;
use crate::core::hypergryph::constants::Server;
use crate::core::story::{self, ScriptError, StoryAssetIndex, StoryScript};

/// Load, parse and resolve one story. `None` when the id is unknown; a 404
/// with a message when the id is known but no script file backs it.
pub async fn get_story(
    state: &AppState,
    server: Server,
    story_id: &str,
) -> Result<Option<StoryScript>, ApiError> {
    let cache = cached_index(state, server).await?;
    let Some(story_ref) = cache.lookup.get(story_id).cloned() else {
        return Ok(None);
    };
    let server_data = state.require_server_data(server)?;
    let trees = StoryTrees::of(&server_data);
    let id = story_id.to_owned();
    cpu::run("story_parse", move || {
        load_and_parse(
            &trees.art_dir,
            &trees.assets_dir,
            &trees.live,
            &id,
            &story_ref,
        )
    })
    .await?
}

/// The synchronous half of [`get_story`], shared with the integration test.
/// The script and its synopsis are read under `assets_dir`, the art they name
/// resolves under `art_dir` (the same tree except on a text-only server).
pub fn load_and_parse(
    art_dir: &std::path::Path,
    assets_dir: &std::path::Path,
    live_assets: &Arc<AssetIndex>,
    story_id: &str,
    story_ref: &StoryRef,
) -> Result<Option<StoryScript>, ApiError> {
    let script = match story::load_script(assets_dir, &story_ref.story_txt) {
        Ok(s) => s,
        Err(e @ (ScriptError::Missing { .. } | ScriptError::NotAScript { .. })) => {
            return Err(ApiError::NotFoundMessage(format!(
                "story `{story_id}`: {e}"
            )));
        }
        Err(e) => return Err(ApiError::Internal(anyhow::anyhow!(e))),
    };
    let index = StoryAssetIndex::for_dirs(art_dir, assets_dir, live_assets);
    let mut parsed = story::parse_story(
        story_id,
        &story_ref.name,
        &story_ref.group_id,
        &script,
        &index,
    );
    // The summary is read the way the script is, uncached: one file of at
    // most 490 bytes on the EN tree, beside a parse that already reads more.
    parsed.synopsis = story_ref
        .story_info
        .as_deref()
        .and_then(|info| story::load_synopsis(assets_dir, info));
    Ok(Some(parsed))
}
