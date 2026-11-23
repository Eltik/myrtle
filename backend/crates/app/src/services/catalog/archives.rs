use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use myrtle_engine_archives::build::file_ids;
use myrtle_engine_archives::dto::ArchiveAppearance;
use myrtle_engine_archives::profile::Lang;
use myrtle_engine_archives::{SceneIndex, build_archive};
use myrtle_engine_story::dto::StoryIndex;
use myrtle_engine_story::sprites::SpriteGallery;
use myrtle_gamedata_raw::tables::handbook_network::RawNetwork;
use myrtle_gamedata_raw::{DirSource, TableSource};
use myrtle_types::Server;

use super::static_data::{TTL, key, to_json};
use crate::cache::CachedJson;
use crate::snapshots::Snapshot;
use crate::{AppContext, AppError};

const ENTRY_TTL: Duration = TTL.saturating_add(Duration::from_mins(5));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveList {
    Network,
    Dossiers,
    Demographics,
    HiddenFactions,
}

impl ArchiveList {
    const fn part(self) -> &'static str {
        match self {
            Self::Network => "network",
            Self::Dossiers => "dossiers",
            Self::Demographics => "demographics",
            Self::HiddenFactions => "hidden",
        }
    }
}

fn part_key(snapshot: &Snapshot, server: Server, part: &str) -> String {
    key(snapshot, server, &format!("archives_{part}"), "0")
}

fn operator_key(snapshot: &Snapshot, server: Server, char_id: &str) -> String {
    key(
        snapshot,
        server,
        &format!("archive_operator:{char_id}"),
        "0",
    )
}

fn load_network(root: &Path) -> Option<RawNetwork> {
    let source = DirSource {
        root: root.to_path_buf(),
    };
    let force_map = source
        .read_gamedata("art/handbook_force_map_table.json.json")
        .ok()??;
    let positions = source
        .read_gamedata("art/handbookpos_table.json.json")
        .ok()??;
    RawNetwork::from_slices(&force_map, &positions)
        .inspect_err(
            |(table, e)| tracing::warn!(table, error = %e, "archives network did not parse"),
        )
        .ok()
}

#[must_use]
pub fn scene_index(sprites: &SpriteGallery, library: &StoryIndex) -> SceneIndex {
    let order: HashMap<&str, (usize, Option<&String>)> = library
        .groups
        .iter()
        .flat_map(|g| &g.stories)
        .enumerate()
        .map(|(i, s)| (s.id.as_str(), (i, s.code.as_ref())))
        .collect();
    let mut out: HashMap<String, HashMap<&str, (usize, ArchiveAppearance)>> = HashMap::new();
    for detail in sprites.details.values() {
        let Some(char_id) = &detail.sprite.char_id else {
            continue;
        };
        let rows = out.entry(char_id.clone()).or_default();
        for s in detail.stories.iter().filter(|s| s.lines > 0.0) {
            let (pos, code) = order
                .get(s.id.as_str())
                .copied()
                .unwrap_or((usize::MAX, None));
            rows.entry(&s.id)
                .and_modify(|(_, a)| a.lines += s.lines)
                .or_insert_with(|| {
                    let appearance = ArchiveAppearance {
                        story_id: s.id.clone(),
                        name: s.name.clone(),
                        code: code.cloned(),
                        tag: s.tag.clone(),
                        group_id: s.group_id.clone(),
                        group_name: s.group_name.clone(),
                        lines: s.lines,
                    };
                    (pos, appearance)
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

fn build_entries(
    snapshot: &Snapshot,
    server: Server,
    root: &Path,
    scenes: &SceneIndex,
) -> Result<Vec<(String, CachedJson)>, AppError> {
    let gd = &snapshot.model;
    let network = load_network(root);
    let archive = build_archive(
        gd,
        &gd.factions,
        network.as_ref(),
        scenes,
        Lang::for_server(server),
    );
    let lists = [
        (ArchiveList::Network, to_json(&archive.network)?),
        (ArchiveList::Dossiers, to_json(&archive.dossiers)?),
        (ArchiveList::Demographics, to_json(&archive.demographics)?),
        (ArchiveList::HiddenFactions, to_json(&archive.hidden)?),
    ];
    let mut entries: Vec<(String, CachedJson)> = lists
        .into_iter()
        .map(|(list, body)| {
            (
                part_key(snapshot, server, list.part()),
                CachedJson::new(body),
            )
        })
        .collect();
    for (char_id, operator) in &archive.operators {
        entries.push((
            operator_key(snapshot, server, char_id),
            CachedJson::new(to_json(operator)?),
        ));
    }
    Ok(entries)
}

async fn ensure_built(
    ctx: &AppContext,
    snapshot: &Arc<Snapshot>,
    server: Server,
) -> Result<(), AppError> {
    let snapshot = Arc::clone(snapshot);
    let root = ctx.snapshots.server_dir(server);
    let (cache, cpu, owned) = (ctx.cache.clone(), ctx.cpu.clone(), ctx.clone());
    let marker = part_key(&snapshot, server, "built");
    ctx.cache
        .json_detached(&marker, TTL, move || async move {
            let sprites = match crate::services::story::sprites_shared(&owned, server).await {
                Ok(sprites) => Some(sprites),
                Err(e) => {
                    tracing::warn!(server = server.as_str(), error = %e, "archives: story gallery unavailable, building without appearances");
                    None
                }
            };
            let entries = cpu
                .run("archives", move || {
                    let scenes = match (&sprites, snapshot.story()) {
                        (Some(sprites), Ok(story)) => scene_index(sprites, &story.library.index),
                        _ => SceneIndex::new(),
                    };
                    build_entries(&snapshot, server, &root, &scenes)
                        .map(|entries| (entries, scenes.len()))
                })
                .await??;
            let (entries, speakers) = entries;
            tracing::info!(
                server = server.as_str(),
                entries = entries.len(),
                speakers,
                "archives built"
            );
            for (key, body) in &entries {
                cache.set_json(key, body, ENTRY_TTL).await;
            }
            Ok("true".to_owned())
        })
        .await?;
    Ok(())
}

async fn entry(
    ctx: &AppContext,
    snapshot: &Arc<Snapshot>,
    server: Server,
    key: &str,
    expected: impl FnOnce() -> bool,
) -> Result<Option<CachedJson>, AppError> {
    ensure_built(ctx, snapshot, server).await?;
    if let Some(hit) = ctx.cache.get_json(key).await {
        return Ok(Some(hit));
    }
    if !expected() {
        return Ok(None);
    }
    tracing::warn!(
        server = server.as_str(),
        key,
        "archives entry missing, rebuilding"
    );
    ctx.cache
        .invalidate(&part_key(snapshot, server, "built"))
        .await;
    ensure_built(ctx, snapshot, server).await?;
    Ok(ctx.cache.get_json(key).await)
}

pub async fn list(
    ctx: &AppContext,
    server: Server,
    list: ArchiveList,
) -> Result<CachedJson, AppError> {
    let snapshot = ctx.snapshots.require(server)?;
    let key = part_key(&snapshot, server, list.part());
    entry(ctx, &snapshot, server, &key, || true)
        .await?
        .ok_or(AppError::Unavailable)
}

pub async fn operator(
    ctx: &AppContext,
    server: Server,
    char_id: &str,
) -> Result<CachedJson, AppError> {
    let snapshot = ctx.snapshots.require(server)?;
    let has_file = || file_ids(&snapshot.model).iter().any(|id| id == char_id); // an unknown id must not rebuild
    let key = operator_key(&snapshot, server, char_id);
    entry(ctx, &snapshot, server, &key, has_file)
        .await?
        .ok_or(AppError::NotFound(None))
}
