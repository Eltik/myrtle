use std::path::{Component, Path};

use axum::extract::{Path as UrlPath, Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::catalog::release;
use myrtle_app::services::story::{ensure_head, with_default_fallback};
use myrtle_engine_story::thumbs::ThumbFormat;
use myrtle_gamedata::assets::{AssetIndex, AssetKind};
use myrtle_types::Server;
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extract::ServerSel;
use crate::router::ApiRouter;

mod delivery;

use delivery::{chunk_bytes, deliver, not_found, serve_file};

const PUBLIC_ASSET_ROOTS: &[&str] = &["textures", "spine", "audio", "video", "portraits"];

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/avatar/{id}", get(avatar))
        .route("/portrait/{id}", get(portrait))
        .route("/skill-icon/{id}", get(skill_icon))
        .route("/module-icon/{id}", get(module_icon))
        .route("/module-big/{id}", get(module_big))
        .route("/enemy-icon/{id}", get(enemy_icon))
        .route("/item-icon/{id}", get(item_icon))
        .route("/medal-icon/{id}", get(medal_icon))
        .route("/charart/{id}", get(charart))
        .route("/skin-portrait/{id}", get(skin_portrait))
        .route("/banner-image/{pool_id}", get(banner_image))
        .route("/event-image/{act_id}", get(event_image))
        .route("/brand-kv/{kv_id}", get(brand_kv))
        .route("/brand-logo/{brand_id}", get(brand_logo))
        .route("/story-sprite-thumb/{id}", get(story_sprite_thumb))
        .route("/assets/{*path}", get(generic))
}

async fn avatar(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> ApiResult<Response> {
    serve_indexed(&ctx, server, &headers, |idx| {
        idx.path(AssetKind::Avatar, &id)
            .or_else(|| idx.path(AssetKind::Avatar, &format!("{id}_2"))) // some operators ship only an elite avatar
            .or_else(|| idx.path(AssetKind::Avatar, &format!("{id}_1")))
            .map(ToOwned::to_owned)
    })
    .await
}

async fn portrait(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> ApiResult<Response> {
    serve_indexed(&ctx, server, &headers, |idx| {
        idx.portrait_path(&id).map(ToOwned::to_owned)
    })
    .await
}

async fn skill_icon(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> ApiResult<Response> {
    serve_indexed(&ctx, server, &headers, |idx| {
        idx.skill_icon_path(&id).map(ToOwned::to_owned)
    })
    .await
}

async fn module_icon(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> ApiResult<Response> {
    serve_indexed(&ctx, server, &headers, |idx| {
        idx.module_icon_path(&id).map(ToOwned::to_owned)
    })
    .await
}

async fn module_big(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> ApiResult<Response> {
    serve_indexed(&ctx, server, &headers, |idx| {
        idx.module_big_path(&id).map(ToOwned::to_owned)
    })
    .await
}

async fn enemy_icon(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> ApiResult<Response> {
    serve_indexed(&ctx, server, &headers, |idx| {
        idx.path(AssetKind::EnemyIcon, &id).map(ToOwned::to_owned)
    })
    .await
}

async fn item_icon(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> ApiResult<Response> {
    serve_indexed(&ctx, server, &headers, |idx| {
        idx.path(AssetKind::ItemIcon, &id).map(ToOwned::to_owned)
    })
    .await
}

async fn medal_icon(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> ApiResult<Response> {
    serve_indexed(&ctx, server, &headers, |idx| {
        idx.path(AssetKind::MedalIcon, &id).map(ToOwned::to_owned)
    })
    .await
}

async fn charart(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> ApiResult<Response> {
    serve_indexed(&ctx, server, &headers, |idx| idx.charart_path(&id)).await
}

async fn skin_portrait(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> ApiResult<Response> {
    serve_indexed(&ctx, server, &headers, |idx| {
        idx.path(AssetKind::SkinPortrait, &id)
            .map(ToOwned::to_owned)
    })
    .await
}

async fn banner_image(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    UrlPath(pool_id): UrlPath<String>,
) -> ApiResult<Response> {
    serve_indexed(&ctx, server, &headers, |idx| {
        idx.gacha_banner_path(&pool_id).map(ToOwned::to_owned)
    })
    .await
}

async fn event_image(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    UrlPath(act_id): UrlPath<String>,
) -> ApiResult<Response> {
    let entry = serve_indexed(&ctx, server, &headers, |idx| {
        idx.path(AssetKind::EventBanner, &act_id)
            .map(ToOwned::to_owned)
    })
    .await;
    if entry.is_ok() {
        return entry;
    }
    if let Ok(path) = release::event_banner(&ctx, server, &act_id).await {
        return serve_file(&path, &headers, "", chunk_bytes(&ctx)).await;
    }
    serve_indexed(&ctx, server, &headers, |idx| {
        idx.event_banner_path(&act_id).map(ToOwned::to_owned)
    })
    .await
}

async fn brand_kv(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    UrlPath(kv_id): UrlPath<String>,
) -> ApiResult<Response> {
    serve_indexed(&ctx, server, &headers, |idx| {
        idx.brand_kv_path(&kv_id).map(ToOwned::to_owned)
    })
    .await
}

async fn brand_logo(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    UrlPath(brand_id): UrlPath<String>,
) -> ApiResult<Response> {
    serve_indexed(&ctx, server, &headers, |idx| {
        idx.brand_logo_path(&brand_id).map(ToOwned::to_owned)
    })
    .await
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ThumbParam {
    #[default]
    Webp,
    Png,
}

#[derive(Debug, Default, Deserialize)]
struct ThumbQuery {
    #[serde(default)]
    format: ThumbParam,
}

async fn story_sprite_thumb(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
    Query(query): Query<ThumbQuery>,
) -> ApiResult<Response> {
    let (format, etag_prefix) = match query.format {
        ThumbParam::Webp => (ThumbFormat::Webp, ""),
        ThumbParam::Png => (ThumbFormat::Png, "png-"),
    };
    for srv in with_default_fallback(&ctx, server) {
        let Some(snapshot) = ctx.snapshots.get(srv) else {
            continue;
        };
        let Some(sprite) = snapshot.model.assets.story_sprites().get(&id) else {
            continue;
        };
        let root = ctx.snapshots.art_dir(srv);
        let rel = ensure_head(
            &ctx,
            &root,
            &id,
            &sprite.body_path(),
            sprite.face_center,
            format,
        )
        .await?;
        return serve_file(&root.join(rel), &headers, etag_prefix, chunk_bytes(&ctx)).await;
    }
    Err(not_found())
}

async fn generic(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    UrlPath(path): UrlPath<String>,
) -> ApiResult<Response> {
    if !ctx.config.assets.allow_all_dirs && !is_public_asset_path(&path) {
        return Err(not_found());
    }
    let dirs = with_default_fallback(&ctx, server)
        .into_iter()
        .filter(|srv| ctx.snapshots.get(*srv).is_some())
        .filter_map(|srv| dir_name(&ctx.snapshots.server_dir(srv)))
        .collect();
    deliver(&ctx, dirs, &path, &headers).await
}

async fn serve_indexed<F>(
    ctx: &AppContext,
    server: Server,
    headers: &HeaderMap,
    resolve: F,
) -> ApiResult<Response>
where
    F: Fn(&AssetIndex) -> Option<String>,
{
    let mut last = not_found();
    for srv in with_default_fallback(ctx, server) {
        let Some(snapshot) = ctx.snapshots.get(srv) else {
            continue;
        };
        let Some(rel) = resolve(&snapshot.model.assets) else {
            continue;
        };
        let Some(dir) = dir_name(&ctx.snapshots.art_dir(srv)) else {
            continue;
        };
        match deliver(ctx, vec![dir], &rel, headers).await {
            Ok(response) => return Ok(response),
            Err(e) => last = e,
        }
    }
    Err(last)
}

pub(crate) async fn deliver_file(
    ctx: &AppContext,
    dir: &Path,
    rel: &str,
    headers: &HeaderMap,
) -> ApiResult<Response> {
    let dir = dir_name(dir).ok_or_else(not_found)?;
    deliver(ctx, vec![dir], rel, headers).await
}

fn is_public_asset_path(requested: &str) -> bool {
    let rel = Path::new(requested.trim_start_matches('/'));
    if !rel
        .components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return false;
    }
    match rel.components().find(|c| !matches!(c, Component::CurDir)) {
        Some(Component::Normal(first)) => first
            .to_str()
            .is_some_and(|f| PUBLIC_ASSET_ROOTS.contains(&f)),
        _ => false,
    }
}

fn dir_name(dir: &Path) -> Option<String> {
    dir.file_name()?.to_str().map(ToOwned::to_owned)
}
