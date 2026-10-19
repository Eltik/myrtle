//! The In-Game Music gallery: every piece of music the game names, where it
//! comes from and where it plays, built once per game data load.
//!
//! - [`dto`]: the wire types.
//! - [`build`]: the derivation, one source per place kind.
//! - [`siren`]: the Monster Siren Records catalogue, synced from the label.
//! - [`library`]: the account's likes, listening history and playlists.

mod build;
mod dto;
pub mod library;
pub mod siren;
pub mod stats;

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Weak};

pub use build::{MusicSources, build_music_index};
pub use dto::*;

use crate::app::cache::SharedJson;
use crate::app::cpu;
use crate::app::error::ApiError;
use crate::app::services::story::ServerCache;
use crate::app::state::AppState;
use crate::core::gamedata::assets::AssetIndex;
use crate::core::hypergryph::constants::Server;

/// The index plus the asset index it resolved clips against, so an asset hot
/// reload rebuilds it as a game-data reload does.
struct Cached {
    /// The index serialized once, with its `ETag`: every request answers with
    /// these bytes, and a revalidation that matches gets a 304.
    json: SharedJson,
    /// Every track id in the index, for checking a `game:` id without
    /// parsing the JSON back.
    ids: Arc<HashSet<String>>,
    assets: Weak<AssetIndex>,
}

static MUSIC: ServerCache<Cached> = ServerCache::new();

/// Drop the music index cached for `server`; see [`ServerCache::clear`].
pub(crate) fn clear_cache(server: Server) -> bool {
    MUSIC.clear(server)
}

/// The trees `server`'s index reads: its own first, then every other loaded
/// server's, so a clip or a level the asked server no longer ships is found
/// where it still is.
fn sources(state: &AppState, server: Server) -> Option<MusicSources> {
    let own = state.try_server_data(server)?;
    let mut level_dirs = vec![PathBuf::from(&own.assets_dir).join("gamedata/levels")];
    let mut clip_trees = vec![(None, PathBuf::from(&own.art_dir))];
    for other in state.pickable_servers() {
        if other == server {
            continue;
        }
        let Some(sd) = state.try_server_data(other) else {
            continue;
        };
        let levels = PathBuf::from(&sd.assets_dir).join("gamedata/levels");
        if !level_dirs.contains(&levels) {
            level_dirs.push(levels);
        }
        let art = PathBuf::from(&sd.art_dir);
        if !clip_trees.iter().any(|(_, dir)| *dir == art) {
            clip_trees.push((Some(other.as_str().to_owned()), art));
        }
    }
    Some(MusicSources {
        assets_dir: PathBuf::from(&own.assets_dir),
        level_dirs,
        clip_trees,
    })
}

/// `server`'s music index as JSON bytes, built on first request after each
/// load.
///
/// # Errors
/// `404` for a server that is not loaded, `500` when the index does not
/// serialize, `503` when the CPU pool sheds.
pub async fn get_music_index(state: &AppState, server: Server) -> Result<SharedJson, ApiError> {
    Ok(cached(state, server).await?.json.clone())
}

/// Every track id in `server`'s index (the part after `game:`), from the
/// same build as [`get_music_index`].
///
/// # Errors
/// As [`get_music_index`].
pub async fn game_track_ids(
    state: &AppState,
    server: Server,
) -> Result<Arc<HashSet<String>>, ApiError> {
    Ok(Arc::clone(&cached(state, server).await?.ids))
}

async fn cached(state: &AppState, server: Server) -> Result<Arc<Cached>, ApiError> {
    let sd = state.require_server_data(server)?;
    let gd = sd.game_data.load_full();
    let live = sd.asset_index.load_full();
    let src = sources(state, server).ok_or(ApiError::NotFound)?;
    let build_gd = Arc::clone(&gd);
    let build_live = Arc::clone(&live);
    let cached = MUSIC
        .get_or_build(
            server,
            gd,
            |hit| hit.assets.upgrade().is_some_and(|a| Arc::ptr_eq(&a, &live)),
            || async move {
                cpu::run("music_index", move || {
                    let started = std::time::Instant::now();
                    let index = build_music_index(&build_gd, &src);
                    tracing::info!(
                        server = ?server,
                        tracks = index.totals.tracks,
                        titled = index.totals.titled,
                        with_audio = index.totals.with_audio,
                        unavailable = index.totals.unavailable,
                        build_ms = started.elapsed().as_millis(),
                        "music index built"
                    );
                    let json =
                        serde_json::to_vec(&index).map_err(|e| ApiError::Internal(e.into()))?;
                    let ids = index.tracks.iter().map(|t| t.id.clone()).collect();
                    Ok::<_, ApiError>(Arc::new(Cached {
                        json: SharedJson::new(json),
                        ids: Arc::new(ids),
                        assets: Arc::downgrade(&build_live),
                    }))
                })
                .await?
            },
        )
        .await?;
    Ok(cached)
}
