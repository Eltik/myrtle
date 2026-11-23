pub mod library;
pub mod siren;
pub mod stats;

use std::collections::HashSet;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use myrtle_engine_music::{
    ClipTree, MusicIndex, MusicSources, MusicTables, StoryScript, bgm_event, build_music_index,
    level_ids,
};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::story::StoryVariables;
use myrtle_gamedata_raw::{DirSource, TableSource};
use myrtle_types::Server;

use crate::cache::{CachedJson, key_hash};
use crate::snapshots::Snapshot;
use crate::{AppContext, AppError};

const INDEX_TTL: Duration = crate::services::catalog::static_data::TTL;

struct View {
    snapshot: Arc<Snapshot>,
    trees: Vec<(Option<String>, PathBuf)>,
    key: String,
    ids_key: String,
}

fn view(ctx: &AppContext, server: Server) -> Result<View, AppError> {
    let server = server.data_server();
    let snapshot = ctx.snapshots.require(server)?;
    let loaded: Vec<(Server, Arc<Snapshot>)> = ctx
        .snapshots
        .loaded()
        .into_iter()
        .filter_map(|s| {
            let held = if s == server {
                Arc::clone(&snapshot)
            } else {
                ctx.snapshots.get(s)?
            };
            Some((s, held))
        })
        .collect();
    let (key, ids_key) = index_keys(server, &loaded);
    let trees = std::iter::once((None, server))
        .chain(
            loaded
                .iter()
                .filter(|(s, _)| *s != server)
                .map(|(s, _)| (Some(s.as_str().to_owned()), *s)),
        )
        .map(|(label, s)| (label, ctx.snapshots.server_dir(s)))
        .collect();
    Ok(View {
        snapshot,
        trees,
        key,
        ids_key,
    })
}

#[must_use]
pub fn index_keys(server: Server, loaded: &[(Server, Arc<Snapshot>)]) -> (String, String) {
    let mut versions = String::new();
    for (s, snapshot) in loaded {
        let v = &snapshot.model.meta.game_version;
        let _ = write!(versions, "{}={}/{};", s.as_str(), v.client, v.res);
    }
    let hash = key_hash(versions.as_bytes());
    let server = server.data_server().as_str();
    (
        format!("static:{server}:music:{hash}"),
        format!("static:{server}:music_ids:{hash}"),
    )
}

pub async fn game_index(ctx: &AppContext, server: Server) -> Result<CachedJson, AppError> {
    index_of(ctx, server, view(ctx, server)?).await
}

async fn index_of(ctx: &AppContext, server: Server, view: View) -> Result<CachedJson, AppError> {
    let server = server.data_server();
    let View {
        snapshot,
        trees,
        key,
        ids_key,
    } = view;
    let build_ctx = ctx.clone();
    ctx.cache
        .json_detached(&key, INDEX_TTL, move || async move {
            let (json, ids) = build_ctx
                .cpu
                .run("music_index", move || {
                    let index =
                        build_music_index(&snapshot.model, &sources(&snapshot.model, &trees));
                    tracing::info!(
                        server = server.as_str(),
                        tracks = index.totals.tracks,
                        with_audio = index.totals.with_audio,
                        unavailable = index.totals.unavailable,
                        "music index built"
                    );
                    let ids: Vec<String> = index.tracks.iter().map(|t| t.id.clone()).collect();
                    serde_json::to_string(&index).map(|json| (json, ids))
                })
                .await?
                .map_err(AppError::internal)?;
            build_ctx.cache.set_json(&ids_key, &ids, INDEX_TTL).await;
            Ok(json)
        })
        .await
}

pub async fn game_track_ids(ctx: &AppContext, server: Server) -> Result<HashSet<String>, AppError> {
    let view = view(ctx, server)?;
    let ids_key = view.ids_key.clone();
    if let Some(ids) = ctx.cache.get_json::<HashSet<String>>(&ids_key).await {
        return Ok(ids);
    }
    let index = index_of(ctx, server, view).await?;
    if let Some(ids) = ctx.cache.get_json::<HashSet<String>>(&ids_key).await {
        return Ok(ids);
    }
    let ids: HashSet<String> = ctx
        .cpu
        .offload("music_track_ids", move || {
            serde_json::from_str::<MusicIndex>(&index.body)
                .map(|index| index.tracks.into_iter().map(|t| t.id).collect())
        })
        .await?
        .map_err(AppError::internal)?;
    ctx.cache.set_json(&ids_key, &ids, INDEX_TTL).await;
    Ok(ids)
}

fn sources(model: &GameModel, trees: &[(Option<String>, PathBuf)]) -> MusicSources {
    let dirs: Vec<DirSource> = trees
        .iter()
        .map(|(_, root)| DirSource { root: root.clone() })
        .collect();
    let Some(own) = dirs.first() else {
        return MusicSources::default();
    };
    let read = |dir: &DirSource, rel: &str| dir.read_asset(rel).ok().flatten();
    let (tables, warnings) =
        MusicTables::parse(|name| read(own, &format!("gamedata/excel/{name}.json")));
    for warning in warnings {
        tracing::warn!(warning, "music tables");
    }
    let vars = read(own, "gamedata/story/story_variables.json.json")
        .and_then(|b| StoryVariables::parse(&b).ok())
        .unwrap_or_default();
    let scripts = own
        .list_gamedata("story")
        .unwrap_or_default()
        .into_iter()
        .filter(|p| p.ends_with(".txt.txt"))
        .filter_map(|p| {
            let bytes = read(own, &format!("gamedata/{p}"))?;
            Some(StoryScript::read(
                p.strip_prefix("story/").unwrap_or(&p),
                &bytes,
                &vars,
            ))
        })
        .collect();
    let level_bgm = level_ids(model)
        .into_iter()
        .filter_map(|id| {
            let rel = format!("gamedata/levels/{id}.json");
            let event = dirs.iter().find_map(|d| bgm_event(&read(d, &rel)?))?;
            Some((id, event))
        })
        .collect();
    let clip_trees = trees
        .iter()
        .zip(&dirs)
        .map(|((server, _), dir)| ClipTree {
            server: server.clone(),
            paths: dir
                .list_assets()
                .inspect_err(|e| tracing::warn!(root = %dir.root.display(), error = %e, "music clips not listed"))
                .unwrap_or_default()
                .into_iter()
                .filter(|p| p.starts_with("audio/"))
                .collect(),
        })
        .collect();
    MusicSources {
        tables,
        scripts,
        level_bgm,
        clip_trees,
    }
}
