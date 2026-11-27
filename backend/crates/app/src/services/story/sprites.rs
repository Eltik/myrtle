use std::path::{Path, PathBuf};
use std::sync::Arc;

use myrtle_engine_story::assets::served_rel;
use myrtle_engine_story::dto::{StorySpriteDetail, StorySpriteIndex, StorySpriteVariant};
use myrtle_engine_story::sprites::{SpriteGallery, build_sprite_index};
use myrtle_engine_story::thumbs::{
    FORMAT, HEAD_PX, THUMB_H, ThumbFormat, compose, encode, head_rel_path, render_head,
    variant_rel_path,
};
use myrtle_gamedata_raw::DirSource;
use myrtle_types::Server;

use super::files::{
    HEAD_RENDERS, THUMB_RENDERS, fresh, render_permit, touch_if_old, write_atomically,
};
use super::gallery::art_dir;
use super::{derived, snapshot};
use crate::{AppContext, AppError};

pub async fn sprites_shared(
    ctx: &AppContext,
    server: Server,
) -> Result<Arc<SpriteGallery>, AppError> {
    let fallback = ctx.snapshots.server_dir(server);
    derived(
        ctx,
        server,
        "story_sprites",
        |d| &d.sprites,
        move |snapshot, story| {
            let (art, text) = snapshot.story_dirs().map_or_else(
                || (fallback.clone(), fallback.clone()),
                |(a, t)| (a.to_path_buf(), t.to_path_buf()),
            );
            let file_len = |url: &str| {
                std::fs::metadata(art.join(served_rel(url)))
                    .ok()
                    .map(|m| m.len())
            };
            let gallery = build_sprite_index(
                &snapshot.model,
                &story.library,
                &story.assets,
                &DirSource { root: text },
                &DirSource { root: art.clone() },
                &file_len,
            );
            tracing::info!(
                folders = gallery.index.sprites.len(),
                scripts = gallery.census.scripts,
                "story sprite gallery built"
            );
            gallery
        },
    )
    .await
}

pub async fn get_story_sprites(
    ctx: &AppContext,
    server: Server,
) -> Result<StorySpriteIndex, AppError> {
    Ok(sprites_shared(ctx, server).await?.index.clone())
}

pub async fn get_story_sprite(
    ctx: &AppContext,
    server: Server,
    base: &str,
) -> Result<StorySpriteDetail, AppError> {
    sprites_shared(ctx, server)
        .await?
        .details
        .get(&base.to_ascii_lowercase())
        .cloned()
        .ok_or_else(|| AppError::NotFound(Some(format!("no story sprite folder `{base}`"))))
}

pub async fn get_variant_thumb(
    ctx: &AppContext,
    server: Server,
    base: &str,
    key: &str,
) -> Result<(PathBuf, String), AppError> {
    let gallery = sprites_shared(ctx, server).await?;
    let detail = gallery
        .details
        .get(&base.to_ascii_lowercase())
        .ok_or(AppError::NotFound(None))?;
    let variant = detail
        .variants
        .iter()
        .find(|v| v.key == key)
        .ok_or(AppError::NotFound(None))?;
    let root = art_dir(ctx, &*snapshot(ctx, server).await?, server);
    let rel = ensure_variant_thumb(ctx, &root, &detail.sprite.base, variant).await?;
    Ok((root, rel))
}

pub async fn ensure_variant_thumb(
    ctx: &AppContext,
    root: &Path,
    base: &str,
    variant: &StorySpriteVariant,
) -> Result<String, AppError> {
    let body = root.join(served_rel(&variant.sprite.body_url));
    let face = variant
        .sprite
        .face_url
        .as_deref()
        .map(|u| root.join(served_rel(u)));
    let rel = variant_rel_path(base, &variant.key, FORMAT);
    let target = root.join(&rel);
    let sources: Vec<&Path> = std::iter::once(body.as_path())
        .chain(face.as_deref())
        .collect();
    if fresh(&target, &sources) {
        touch_if_old(&target);
        return Ok(rel);
    }
    let _permit = render_permit(&THUMB_RENDERS).await?;
    if fresh(&target, &sources) {
        return Ok(rel);
    }
    let sprite = variant.sprite.clone();
    ctx.cpu
        .run("story_sprite_variant_thumb", move || {
            let body_png = std::fs::read(&body).map_err(|e| e.to_string())?;
            let face_png = face
                .as_deref()
                .map(std::fs::read)
                .transpose()
                .map_err(|e| e.to_string())?;
            let img = compose(&body_png, face_png.as_deref(), &sprite, THUMB_H)
                .map_err(|e| e.to_string())?;
            let bytes = encode(&img, FORMAT).map_err(|e| e.to_string())?;
            write_atomically(&target, &bytes).map_err(|e| e.to_string())
        })
        .await?
        .map_err(|e| {
            tracing::warn!(base, key = %variant.key, error = %e, "sprite variant thumb failed");
            AppError::NotFound(None)
        })?;
    Ok(rel)
}

pub async fn ensure_head(
    ctx: &AppContext,
    root: &Path,
    id: &str,
    body_rel: &str,
    face: Option<(f32, f32)>,
    format: ThumbFormat,
) -> Result<String, AppError> {
    if id.is_empty() || id.contains(['/', '\\', '\0']) || id.starts_with('.') {
        return Err(AppError::NotFound(None));
    }
    let source = root.join(body_rel.trim_start_matches('/'));
    let rel = head_rel_path(id, format);
    let target = root.join(&rel);
    if fresh(&target, &[&source]) {
        return Ok(rel);
    }
    let _permit = render_permit(&HEAD_RENDERS).await?;
    if fresh(&target, &[&source]) {
        return Ok(rel);
    }
    ctx.cpu
        .offload("story_sprite_thumb", move || {
            let png = std::fs::read(&source).map_err(|e| e.to_string())?;
            let bytes = render_head(&png, face, HEAD_PX, format).map_err(|e| e.to_string())?;
            write_atomically(&target, &bytes).map_err(|e| e.to_string())
        })
        .await?
        .map_err(|e| {
            tracing::warn!(id, error = %e, "story sprite thumbnail failed");
            AppError::NotFound(None)
        })?;
    Ok(rel)
}
