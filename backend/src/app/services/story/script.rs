//! Loading and parsing ONE story, the `GET /story/{id}` half.
//!
//! Nothing here is cached, the story summary included: the parse is milliseconds (measured 2026-09-21: the
//! longest EN script parses in under 10 ms) and the asset index it resolves
//! through is the shared, already-warmed one. The work runs under
//! [`cpu::run`] because the FIRST call on a cold process still builds that
//! index, one walk over roughly 90k files.

use std::sync::Arc;

use super::cache::cached_index;
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
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let assets_dir = std::path::PathBuf::from(&server_data.assets_dir);
    let live_assets = server_data.asset_index.load_full();
    let id = story_id.to_owned();
    // The parse is milliseconds (measured 2026-09-21: the longest EN script
    // parses in under 10 ms), but the first call also builds the asset index,
    // one walk over ~90k files, so the whole thing runs off the async worker.
    cpu::run("story_parse", move || {
        load_and_parse(&assets_dir, &live_assets, &id, &story_ref)
    })
    .await?
}

/// The synchronous half of [`get_story`], shared with the integration test.
pub fn load_and_parse(
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
    let index = StoryAssetIndex::for_dir(assets_dir, live_assets);
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
