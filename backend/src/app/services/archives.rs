//! The Archives service: one [`Archive`] per server and game-data load, built
//! under `cpu::run` and cached in a [`ServerCache`].
//!
//! Story appearances ride the story reader's character gallery
//! ([`story::cached_sprites`]): its speaker walk attributes every named line
//! to the sprite lit when it is spoken, and a sprite folder `char_002_amiya_1`
//! names its operator. An operator SPEAKS IN a story when at least one line
//! there is attributed to one of their folders. This undercounts by
//! construction: someone on stage who never speaks, or who speaks with no
//! sprite up, is not counted. When the gallery cannot be built the Archives
//! still answer, with no appearances, and the failure is logged.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Weak};
use std::time::Instant;

use serde::Serialize;

use crate::app::cache::SharedJson;
use crate::app::services::story::{self, ServerCache};
use crate::app::{cpu, error::ApiError, state::AppState};
use crate::core::archives::network::load_network;
use crate::core::archives::profile::Lang;
use crate::core::archives::{
    Archive, ArchiveAppearance, ArchiveOperator, SceneIndex, build_archive,
};
use crate::core::gamedata::tables::load_table;
use crate::core::gamedata::types::handbook_team::HandbookTeamTableFile;
use crate::core::hypergryph::constants::Server;

static ARCHIVE: ServerCache<Archive> = ServerCache::new();

/// Library order and story code per story id, from the story index.
type StoryOrder = HashMap<String, (usize, Option<String>)>;

fn story_order(library: &story::StoryIndexCache) -> StoryOrder {
    library
        .index
        .groups
        .iter()
        .flat_map(|g| &g.stories)
        .enumerate()
        .map(|(i, s)| (s.id.clone(), (i, s.code.clone())))
        .collect()
}

/// Every operator's speaking appearances, in library order, one row per
/// story with the lines of all their folders summed.
fn scene_index(sprites: &story::StorySpriteCache, order: &StoryOrder) -> SceneIndex {
    let mut out: HashMap<String, HashMap<String, (usize, ArchiveAppearance)>> = HashMap::new();
    for detail in sprites.details.values() {
        let Some(char_id) = &detail.sprite.char_id else {
            continue;
        };
        let rows = out.entry(char_id.clone()).or_default();
        for s in detail.stories.iter().filter(|s| s.lines > 0.0) {
            let (pos, code) = order.get(&s.id).cloned().unwrap_or((usize::MAX, None));
            rows.entry(s.id.clone())
                .and_modify(|(_, a)| a.lines += s.lines)
                .or_insert_with(|| {
                    (
                        pos,
                        ArchiveAppearance {
                            story_id: s.id.clone(),
                            name: s.name.clone(),
                            code,
                            tag: s.tag.clone(),
                            group_id: s.group_id.clone(),
                            group_name: s.group_name.clone(),
                            lines: s.lines,
                        },
                    )
                });
        }
    }
    out.into_iter()
        .map(|(char_id, rows)| {
            let mut rows: Vec<(usize, ArchiveAppearance)> = rows.into_values().collect();
            rows.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.story_id.cmp(&b.1.story_id)));
            let list = rows
                .into_iter()
                .map(|(_, mut a)| {
                    a.lines = (a.lines * 100.0).round() / 100.0;
                    a
                })
                .collect();
            (char_id, list)
        })
        .collect()
}

/// The cached Archives for `server`, built on the first call after each
/// game-data load.
pub async fn cached_archive(state: &AppState, server: Server) -> Result<Arc<Archive>, ApiError> {
    let sd = state.require_server_data(server)?;
    let gd = sd.game_data.load_full();
    let build_gd = Arc::clone(&gd);
    let excel_dir = sd.game_data_dir.clone();
    let roots = [sd.assets_dir.clone(), sd.art_dir.clone()];
    let state = state.clone();
    ARCHIVE
        .get_or_build(
            server,
            gd,
            |_| true,
            || async move {
                let scenes_in = match (
                    story::cached_index(&state, server).await,
                    story::cached_sprites(&state, server).await,
                ) {
                    (Ok(library), Ok(sprites)) => Some((library, sprites)),
                    (Err(e), _) | (_, Err(e)) => {
                        tracing::warn!(server = ?server, error = %e, "archives: story gallery unavailable, building without appearances");
                        None
                    }
                };
                let built = cpu::run("archives", move || {
                    let started = Instant::now();
                    let teams = load_table::<HandbookTeamTableFile>(Path::new(&excel_dir), "handbook_team_table")
                        .map(|t| t.teams)
                        .unwrap_or_default();
                    let network = roots.iter().find_map(|root| load_network(Path::new(root)));
                    let scenes = scenes_in
                        .map(|(library, sprites)| scene_index(&sprites, &story_order(&library)))
                        .unwrap_or_default();
                    let lang = Lang::for_server(server);
                    let archive = build_archive(&build_gd, &teams, network.as_ref(), &scenes, lang);
                    tracing::info!(
                        server = ?server,
                        files = archive.demographics.total,
                        groups = archive.network.groups.len(),
                        speakers = scenes.len(),
                        build_ms = started.elapsed().as_millis(),
                        "archives built"
                    );
                    Arc::new(archive)
                })
                .await?;
                Ok(built)
            },
        )
        .await
}

/// The four list payloads of one [`Archive`], serialized once with their
/// `ETag`s. Before, every request cloned the struct and serialized it again:
/// 8.4 ms for the 206,991-byte network on a warm cache (curl, identity).
pub struct ArchiveJson {
    /// The archive these bytes were serialized from; a rebuilt archive (a new
    /// game-data load) misses this and re-serializes.
    archive: Weak<Archive>,
    pub network: SharedJson,
    pub dossiers: SharedJson,
    pub demographics: SharedJson,
    pub hidden: SharedJson,
}

static ARCHIVE_JSON: ServerCache<ArchiveJson> = ServerCache::new();

fn encode(value: &impl Serialize) -> Result<SharedJson, ApiError> {
    serde_json::to_vec(value)
        .map(SharedJson::new)
        .map_err(|e| ApiError::Internal(e.into()))
}

/// The serialized list payloads for `server`, built on the first call after
/// each archive build.
pub async fn cached_archive_json(
    state: &AppState,
    server: Server,
) -> Result<Arc<ArchiveJson>, ApiError> {
    let archive = cached_archive(state, server).await?;
    let sd = state.require_server_data(server)?;
    let gd = sd.game_data.load_full();
    let source = Arc::clone(&archive);
    ARCHIVE_JSON
        .get_or_build(
            server,
            gd,
            |hit| {
                hit.archive
                    .upgrade()
                    .is_some_and(|a| Arc::ptr_eq(&a, &archive))
            },
            || async move {
                cpu::run("archives_json", move || {
                    Ok::<_, ApiError>(Arc::new(ArchiveJson {
                        archive: Arc::downgrade(&source),
                        network: encode(&source.network)?,
                        dossiers: encode(&source.dossiers)?,
                        demographics: encode(&source.demographics)?,
                        hidden: encode(&source.hidden)?,
                    }))
                })
                .await?
            },
        )
        .await
}

pub async fn get_network(state: &AppState, server: Server) -> Result<SharedJson, ApiError> {
    Ok(cached_archive_json(state, server).await?.network.clone())
}

pub async fn get_dossiers(state: &AppState, server: Server) -> Result<SharedJson, ApiError> {
    Ok(cached_archive_json(state, server).await?.dossiers.clone())
}

pub async fn get_demographics(state: &AppState, server: Server) -> Result<SharedJson, ApiError> {
    Ok(cached_archive_json(state, server)
        .await?
        .demographics
        .clone())
}

pub async fn get_hidden_factions(state: &AppState, server: Server) -> Result<SharedJson, ApiError> {
    Ok(cached_archive_json(state, server).await?.hidden.clone())
}

pub async fn get_operator(
    state: &AppState,
    server: Server,
    char_id: &str,
) -> Result<ArchiveOperator, ApiError> {
    cached_archive(state, server)
        .await?
        .operators
        .get(char_id)
        .cloned()
        .ok_or(ApiError::NotFound)
}
