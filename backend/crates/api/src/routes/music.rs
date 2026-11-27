use std::net::IpAddr;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{delete, get, patch, post, put};
use myrtle_app::services::music as game;
use myrtle_app::services::music::library::{
    self, MusicLibrary, MusicPlay, MusicPlayInput, MusicPlayUpdate, MusicPlaylist,
    MusicPlaylistAdd, MusicPlaylistCreate, MusicPlaylistPatch,
};
use myrtle_app::services::music::siren;
use myrtle_app::services::music::stats::{
    self, MusicListenerPage, MusicListenerSort, MusicStatInput, MusicStatRecorded, MusicStatWindow,
    MusicTopTracks, StatClient,
};
use myrtle_app::{AppContext, AppError};
use myrtle_db::repos::music_stats::StatKind;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::extract::{self, Caller, ClientIp, MaybeCaller, Paging, ServerSel};
use crate::respond::Cached;
use crate::router::ApiRouter;

const CATALOG_CACHE: &str = "public, max-age=900";
const LYRICS_CACHE: &str = "public, max-age=2592000, immutable";
const STATS_CACHE: &str = "public, max-age=60";
const STREAM_CACHE: &str = "private, max-age=600"; // the signed url expires; no shared cache

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/gallery/music", get(game_index))
        .route("/music/siren", get(siren_catalog))
        .route("/music/siren/albums/{cid}", get(siren_album))
        .route("/music/siren/songs/{cid}/stream", get(siren_stream))
        .route("/music/siren/songs/{cid}/lyrics", get(siren_lyrics))
        .route("/music/library", get(library_get))
        .route("/music/likes/{track_id}", put(like).delete(unlike))
        .route("/music/history", post(history_log))
        .route("/music/history/{id}", put(history_update))
        .route("/music/playlists", post(playlist_create))
        .route(
            "/music/playlists/{id}",
            patch(playlist_patch).delete(playlist_delete),
        )
        .route("/music/playlists/{id}/tracks", post(playlist_add_tracks))
        .route(
            "/music/playlists/{id}/tracks/{track_id}",
            delete(playlist_remove_track),
        )
        .route("/music/stats", get(stats_catalog))
        .route("/music/stats/listen", post(stats_listen))
        .route("/music/stats/download", post(stats_download))
        .route("/music/stats/top-tracks", get(stats_top_tracks))
        .route("/music/stats/top-listeners", get(stats_top_listeners))
        .route(
            "/music/stats/tracks/{track_id}/listeners",
            get(stats_track_listeners),
        )
}

async fn game_index(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
) -> ApiResult<Cached> {
    Ok(Cached::gamedata(game::game_index(&ctx, server).await?))
}

async fn siren_catalog(State(ctx): State<AppContext>) -> ApiResult<Cached> {
    Ok(Cached::new(siren::catalog(&ctx).await?).cache_control(CATALOG_CACHE))
}

async fn siren_album(
    State(ctx): State<AppContext>,
    Path(cid): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let album = siren::album(&ctx, &cid).await?;
    Ok(([(header::CACHE_CONTROL, CATALOG_CACHE)], Json(album)))
}

async fn siren_stream(
    State(ctx): State<AppContext>,
    Path(cid): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let url = siren::stream_url(&ctx, &cid).await?;
    Ok((
        StatusCode::FOUND,
        [
            (header::LOCATION, url),
            (header::CACHE_CONTROL, STREAM_CACHE.to_owned()),
        ],
    ))
}

async fn siren_lyrics(
    State(ctx): State<AppContext>,
    Path(cid): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let text = siren::lyrics(&ctx, &cid).await?;
    Ok((
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::CACHE_CONTROL, LYRICS_CACHE),
        ],
        text,
    ))
}

async fn library_get(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<MusicLibrary>> {
    Ok(Json(library::get(&ctx, &caller).await?))
}

async fn like(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(track_id): Path<String>,
) -> ApiResult<StatusCode> {
    library::like(&ctx, &caller, &track_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn unlike(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(track_id): Path<String>,
) -> ApiResult<StatusCode> {
    library::unlike(&ctx, &caller, &track_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn history_log(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<MusicPlayInput>,
) -> ApiResult<Json<MusicPlay>> {
    Ok(Json(library::log_play(&ctx, &caller, body).await?))
}

async fn history_update(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(id): Path<i64>,
    extract::Json(body): extract::Json<MusicPlayUpdate>,
) -> ApiResult<StatusCode> {
    library::update_play(&ctx, &caller, id, body).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn playlist_create(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<MusicPlaylistCreate>,
) -> ApiResult<Json<MusicPlaylist>> {
    Ok(Json(library::create_playlist(&ctx, &caller, body).await?))
}

async fn playlist_patch(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(id): Path<Uuid>,
    extract::Json(body): extract::Json<MusicPlaylistPatch>,
) -> ApiResult<Json<MusicPlaylist>> {
    Ok(Json(
        library::patch_playlist(&ctx, &caller, id, body).await?,
    ))
}

async fn playlist_delete(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    library::delete_playlist(&ctx, &caller, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn playlist_add_tracks(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(id): Path<Uuid>,
    extract::Json(body): extract::Json<MusicPlaylistAdd>,
) -> ApiResult<Json<MusicPlaylist>> {
    Ok(Json(library::add_tracks(&ctx, &caller, id, body).await?))
}

async fn playlist_remove_track(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path((id, track_id)): Path<(Uuid, String)>,
) -> ApiResult<Json<MusicPlaylist>> {
    Ok(Json(
        library::remove_track(&ctx, &caller, id, &track_id).await?,
    ))
}

async fn stats_catalog(State(ctx): State<AppContext>) -> ApiResult<impl IntoResponse> {
    let catalog = stats::catalog(&ctx).await?;
    Ok(([(header::CACHE_CONTROL, STATS_CACHE)], Json(catalog)))
}

async fn ingest(
    ctx: &AppContext,
    headers: &HeaderMap,
    caller: &MaybeCaller,
    ip: Option<IpAddr>,
    kind: StatKind,
    track_id: &str,
) -> ApiResult<Json<MusicStatRecorded>> {
    if headers.contains_key(header::AUTHORIZATION) && caller.user_id().is_none() {
        return Err(AppError::Unauthorized.into());
    }
    let client = StatClient::of(ctx, caller, ip);
    Ok(Json(
        stats::record(ctx, kind, track_id, client.as_ref()).await?,
    ))
}

async fn stats_listen(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    caller: MaybeCaller,
    ClientIp(ip): ClientIp,
    extract::Json(body): extract::Json<MusicStatInput>,
) -> ApiResult<Json<MusicStatRecorded>> {
    ingest(
        &ctx,
        &headers,
        &caller,
        ip,
        StatKind::Listen,
        &body.track_id,
    )
    .await
}

async fn stats_download(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    caller: MaybeCaller,
    ClientIp(ip): ClientIp,
    extract::Json(body): extract::Json<MusicStatInput>,
) -> ApiResult<Json<MusicStatRecorded>> {
    ingest(
        &ctx,
        &headers,
        &caller,
        ip,
        StatKind::Download,
        &body.track_id,
    )
    .await
}

#[derive(Deserialize)]
struct TopTracksQuery {
    #[serde(default)]
    window: MusicStatWindow,
    limit: Option<u32>,
}

async fn stats_top_tracks(
    State(ctx): State<AppContext>,
    Query(query): Query<TopTracksQuery>,
) -> ApiResult<Json<MusicTopTracks>> {
    Ok(Json(
        stats::top_tracks(&ctx, query.window, query.limit).await?,
    ))
}

#[derive(Deserialize)]
struct ListenersQuery {
    #[serde(default)]
    window: MusicStatWindow,
    #[serde(default)]
    sort: MusicListenerSort,
}

async fn stats_top_listeners(
    State(ctx): State<AppContext>,
    Query(query): Query<ListenersQuery>,
    Paging(page): Paging,
) -> ApiResult<Json<MusicListenerPage>> {
    let listeners = stats::listeners(
        &ctx,
        query.window,
        query.sort,
        None,
        page.limit,
        page.offset,
    )
    .await?;
    Ok(Json(listeners))
}

async fn stats_track_listeners(
    State(ctx): State<AppContext>,
    Path(track_id): Path<String>,
    Query(query): Query<ListenersQuery>,
    Paging(page): Paging,
) -> ApiResult<Json<MusicListenerPage>> {
    let listeners = stats::listeners(
        &ctx,
        query.window,
        query.sort,
        Some(&track_id),
        page.limit,
        page.offset,
    )
    .await?;
    Ok(Json(listeners))
}
