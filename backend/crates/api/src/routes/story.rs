use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing::get;
use myrtle_app::services::story::{self, community};
use myrtle_app::{AppContext, AppError};
use myrtle_engine_story::dto::{
    StoryArchive, StoryArtGallery, StoryArtKind, StoryGallery, StoryIllustrations, StoryIndex,
    StorySpriteDetail, StorySpriteIndex,
};
use myrtle_engine_story::gallery::GallerySize;
use myrtle_engine_story::script::StoryScript;

use crate::error::ApiResult;
use crate::extract::ServerSel;
use crate::respond::Cached;
use crate::router::ApiRouter;
use crate::routes::assets::deliver_file;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/story/community", get(community_counts))
        .route("/story/index", get(index))
        .route("/story/group/{group_id}/illustrations", get(illustrations))
        .route("/story/group/{group_id}/archive", get(archive))
        .route("/story/sprites", get(sprites))
        .route("/story/sprites/{base}", get(sprite_detail))
        .route(
            "/story/sprites/{base}/thumb/{variant}",
            get(sprite_variant_thumb),
        )
        .route("/story/gallery", get(gallery))
        .route("/story/gallery/{id}/{size}", get(gallery_picture))
        .route("/story/art-gallery/{kind}", get(art_gallery))
        .route("/story/art-gallery/{kind}/{id}/{size}", get(art_picture))
        .route("/story/{story_id}", get(detail))
}

async fn community_counts(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
) -> ApiResult<Cached> {
    Ok(Cached::new(community::community(&ctx, server).await?))
}

async fn index(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
) -> ApiResult<Json<StoryIndex>> {
    Ok(Json(story::get_story_index(&ctx, server).await?))
}

async fn detail(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(story_id): Path<String>,
) -> ApiResult<Json<StoryScript>> {
    let script = story::get_story(&ctx, server, &story_id).await?;
    Ok(Json(script.ok_or(AppError::NotFound(None))?))
}

async fn illustrations(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(group_id): Path<String>,
) -> ApiResult<Json<StoryIllustrations>> {
    Ok(Json(
        story::get_group_illustrations(&ctx, server, &group_id).await?,
    ))
}

async fn archive(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(group_id): Path<String>,
) -> ApiResult<Json<StoryArchive>> {
    Ok(Json(
        story::get_group_archive(&ctx, server, &group_id).await?,
    ))
}

async fn sprites(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
) -> ApiResult<Json<StorySpriteIndex>> {
    Ok(Json(story::get_story_sprites(&ctx, server).await?))
}

async fn sprite_detail(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(base): Path<String>,
) -> ApiResult<Json<StorySpriteDetail>> {
    Ok(Json(story::get_story_sprite(&ctx, server, &base).await?))
}

async fn gallery(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
) -> ApiResult<Json<StoryGallery>> {
    Ok(Json(story::get_gallery(&ctx, server).await?))
}

async fn art_gallery(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(kind): Path<StoryArtKind>,
) -> ApiResult<Json<StoryArtGallery>> {
    Ok(Json(story::get_story_art(&ctx, server, kind).await?))
}

async fn sprite_variant_thumb(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    Path((base, variant)): Path<(String, String)>,
) -> ApiResult<Response> {
    let (dir, rel) = story::get_variant_thumb(&ctx, server, &base, &variant).await?;
    deliver_file(&ctx, &dir, &rel, &headers).await
}

async fn gallery_picture(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    Path((id, size)): Path<(String, GallerySize)>,
) -> ApiResult<Response> {
    let (dir, rel) = story::ensure_picture(&ctx, server, &id, size).await?;
    deliver_file(&ctx, &dir, &rel, &headers).await
}

async fn art_picture(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    headers: HeaderMap,
    Path((kind, id, size)): Path<(StoryArtKind, String, GallerySize)>,
) -> ApiResult<Response> {
    let (dir, rel) = story::ensure_story_art(&ctx, server, kind, &id, size).await?;
    deliver_file(&ctx, &dir, &rel, &headers).await
}
