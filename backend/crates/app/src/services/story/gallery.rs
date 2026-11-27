use std::path::PathBuf;
use std::sync::Arc;

use myrtle_engine_story::assets::served_rel;
use myrtle_engine_story::dto::{StoryArtGallery, StoryArtKind, StoryGallery};
use myrtle_engine_story::gallery::{GallerySize, build_gallery, cache_rel_path, render_jpeg};
use myrtle_engine_story::story_art::{self, StoryArtCensus, build_story_art, resolve_art};
use myrtle_types::Server;

use super::files::{PICTURE_RENDERS, fresh, render_permit, touch_if_old, write_atomically};
use super::{derived, snapshot};
use crate::snapshots::Snapshot;
use crate::{AppContext, AppError};

pub struct StoryArt {
    galleries: [(StoryArtGallery, StoryArtCensus); 2],
}

impl StoryArt {
    const fn slot(kind: StoryArtKind) -> usize {
        match kind {
            StoryArtKind::Cg => 0,
            StoryArtKind::Scene => 1,
        }
    }

    #[must_use]
    pub const fn gallery(&self, kind: StoryArtKind) -> &StoryArtGallery {
        &self.galleries[Self::slot(kind)].0
    }

    #[must_use]
    pub const fn census(&self, kind: StoryArtKind) -> &StoryArtCensus {
        &self.galleries[Self::slot(kind)].1
    }

    #[must_use]
    pub fn lists(&self, kind: StoryArtKind, id: &str) -> bool {
        self.gallery(kind)
            .groups
            .iter()
            .any(|g| g.pictures.iter().any(|p| p.id == id))
    }
}

pub(super) fn art_dir(ctx: &AppContext, snapshot: &Snapshot, server: Server) -> PathBuf {
    snapshot.story_dirs().map_or_else(
        || ctx.snapshots.server_dir(server),
        |(a, _)| a.to_path_buf(),
    )
}

pub async fn gallery_shared(
    ctx: &AppContext,
    server: Server,
) -> Result<Arc<StoryGallery>, AppError> {
    derived(
        ctx,
        server,
        "story_gallery",
        |d| &d.gallery,
        |snapshot, story| build_gallery(&snapshot.model, &story.assets),
    )
    .await
}

pub async fn get_gallery(ctx: &AppContext, server: Server) -> Result<StoryGallery, AppError> {
    Ok(StoryGallery::clone(&*gallery_shared(ctx, server).await?))
}

pub async fn story_art_shared(ctx: &AppContext, server: Server) -> Result<Arc<StoryArt>, AppError> {
    let min_bpp = ctx.config.story.art_min_bytes_per_px;
    let fallback = ctx.snapshots.server_dir(server);
    derived(
        ctx,
        server,
        "story_art_gallery",
        |d| &d.art,
        move |snapshot, story| {
            let art = snapshot
                .story_dirs()
                .map_or(fallback, |(a, _)| a.to_path_buf());
            let file_len = |url: &str| {
                std::fs::metadata(art.join(served_rel(url)))
                    .ok()
                    .map(|m| m.len())
            };
            StoryArt {
                galleries: build_story_art(&story.library, &story.assets, &file_len, min_bpp),
            }
        },
    )
    .await
}

pub async fn get_story_art(
    ctx: &AppContext,
    server: Server,
    kind: StoryArtKind,
) -> Result<StoryArtGallery, AppError> {
    Ok(story_art_shared(ctx, server).await?.gallery(kind).clone())
}

pub async fn story_art_census(
    ctx: &AppContext,
    server: Server,
    kind: StoryArtKind,
) -> Result<StoryArtCensus, AppError> {
    Ok(story_art_shared(ctx, server).await?.census(kind).clone())
}

pub async fn story_art_known(ctx: &AppContext, kind: StoryArtKind, id: &str) -> bool {
    for server in ctx.snapshots.loaded() {
        if let Ok(art) = story_art_shared(ctx, server).await
            && art.lists(kind, id)
        {
            return true;
        }
    }
    false
}

#[must_use]
pub fn with_default_fallback(ctx: &AppContext, server: Server) -> Vec<Server> {
    let mut order = vec![server];
    let default = ctx.snapshots.default_server();
    if default != server {
        order.push(default);
    }
    if ctx.config.switches.asset_cn_fallback.enabled
        && !order
            .iter()
            .any(|s| matches!(s, Server::CN | Server::Bilibili))
    {
        order.push(Server::CN);
    }
    order
}

pub async fn ensure_picture(
    ctx: &AppContext,
    server: Server,
    id: &str,
    size: GallerySize,
) -> Result<(PathBuf, String), AppError> {
    for srv in with_default_fallback(ctx, server) {
        let Ok(gallery) = gallery_shared(ctx, srv).await else {
            continue;
        };
        let Some(url) = gallery
            .groups
            .iter()
            .flat_map(|g| &g.pictures)
            .find(|p| p.id == id)
            .and_then(|p| p.url.clone())
        else {
            continue;
        };
        let Ok(snapshot) = snapshot(ctx, srv).await else {
            continue;
        };
        let root = art_dir(ctx, &snapshot, srv);
        return ensure_variant(ctx, root, &url, cache_rel_path(id, size), size).await;
    }
    Err(AppError::NotFound(None))
}

pub async fn ensure_story_art(
    ctx: &AppContext,
    server: Server,
    kind: StoryArtKind,
    id: &str,
    size: GallerySize,
) -> Result<(PathBuf, String), AppError> {
    for srv in with_default_fallback(ctx, server) {
        let Ok(art) = story_art_shared(ctx, srv).await else {
            continue;
        };
        if !art.lists(kind, id) {
            continue;
        }
        let Ok(snapshot) = snapshot(ctx, srv).await else {
            continue;
        };
        let Some((url, _, _)) = snapshot
            .story()
            .ok()
            .and_then(|s| resolve_art(&s.assets, kind, id))
        else {
            continue;
        };
        let root = art_dir(ctx, &snapshot, srv);
        let rel = story_art::cache_rel_path(kind, id, size);
        return ensure_variant(ctx, root, &url, rel, size).await;
    }
    Err(AppError::NotFound(None))
}

pub async fn ensure_variant(
    ctx: &AppContext,
    root: PathBuf,
    url: &str,
    rel: String,
    size: GallerySize,
) -> Result<(PathBuf, String), AppError> {
    let original = served_rel(url);
    let source = root.join(&original);
    let target = root.join(&rel);
    if fresh(&target, &[&source]) {
        touch_if_old(&target);
        return Ok((root, rel));
    }
    let _permit = render_permit(&PICTURE_RENDERS).await?;
    if fresh(&target, &[&source]) {
        return Ok((root, rel));
    }
    let width = size.width();
    let rendered = ctx
        .cpu
        .run("story_gallery_variant", move || {
            let png = std::fs::read(&source).map_err(|e| e.to_string())?;
            let jpeg = render_jpeg(&png, width).map_err(|e| e.to_string())?;
            write_atomically(&target, &jpeg).map_err(|e| e.to_string())
        })
        .await?;
    match rendered {
        Ok(()) => Ok((root, rel)),
        Err(e) => {
            tracing::warn!(rel, error = %e, "gallery variant failed");
            Ok((root, original))
        }
    }
}
