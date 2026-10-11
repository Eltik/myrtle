//! Music: the In-Game Music index (every track the game names, where it comes
//! from and where it plays), the Monster Siren Records catalogue, and the
//! account's likes, history and playlists.

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app::extractors::auth::AuthUser;
use crate::app::routes::static_data::shared_json_response;
use crate::app::services::music::{get_music_index, library, siren};
use crate::app::{error::ApiError, state::AppState};
use crate::core::hypergryph::constants::Server;
use crate::database::models::music::{
    MusicLibrary, MusicPlay, MusicPlayInput, MusicPlayUpdate, MusicPlaylist, MusicPlaylistAdd,
    MusicPlaylistCreate, MusicPlaylistPatch,
};

/// The index with its `ETag` and `Cache-Control: public, max-age=300`; a
/// matching `If-None-Match` gets a 304 and no body.
async fn respond(
    state: &AppState,
    server: Server,
    headers: &HeaderMap,
) -> Result<Response, ApiError> {
    let json = get_music_index(state, server).await?;
    Ok(shared_json_response(&json, headers))
}

/// Every piece of music the game names on the default server: battle themes,
/// lobby and menu music, story scene music and map music, each with its
/// intro and loop clips and every stage, story scene, season, event and
/// screen it plays in. Built once per game data load and asset tree.
#[utoipa::path(
    get,
    path = "/gallery/music",
    operation_id = "gallery_music",
    tag = "gamedata",
    params(
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "Every track. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", body = crate::app::services::music::MusicIndex),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn music(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    respond(&state, state.default_server, &headers).await
}

/// `GET /{server}/gallery/music` - the same from `{server}`.
#[utoipa::path(
    get,
    path = "/{server}/gallery/music",
    operation_id = "gallery_music_srv",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "Every track. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", body = crate::app::services::music::MusicIndex),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn music_srv(
    State(state): State<AppState>,
    Path(server): Path<Server>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    respond(&state, server, &headers).await
}

// ---------------------------------------------------------------------------
// Monster Siren Records
// ---------------------------------------------------------------------------

/// The catalogue is kept this long by a browser or a proxy; a sync lands at
/// most this late.
const CATALOG_CACHE: &str = "public, max-age=900";
/// Lyrics never change once published.
const LYRICS_CACHE: &str = "public, max-age=2592000, immutable";

/// The Monster Siren Records catalogue: every release and every track, as the
/// label lists them, synced from the label's public API on a clock (default
/// every 6 h). Covers are the label's CDN URLs and load directly; audio and
/// lyrics go through `/music/siren/songs/{cid}/stream` and `/lyrics`.
#[utoipa::path(
    get,
    path = "/music/siren",
    operation_id = "music_siren_catalog",
    tag = "music",
    responses(
        (status = 200, description = "Every release, newest first, and every track in album order.", body = siren::SirenCatalog),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, description = "The first sync has not finished yet; it was started by this call.", body = crate::app::error::ErrorBody)
    )
)]
pub async fn siren_catalog(State(state): State<AppState>) -> Result<Response, ApiError> {
    let json = siren::catalog(&state).await?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, CATALOG_CACHE),
        ],
        json,
    )
        .into_response())
}

/// One release in full: the label's blurb, the wide banner and its tracks in
/// album order.
#[utoipa::path(
    get,
    path = "/music/siren/albums/{cid}",
    operation_id = "music_siren_album",
    tag = "music",
    params(("cid" = String, Path, description = "The label's album id, as listed by `/music/siren`.")),
    responses(
        (status = 200, description = "The release.", body = siren::SirenAlbumDetail),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn siren_album(
    State(state): State<AppState>,
    Path(cid): Path<String>,
) -> Result<Response, ApiError> {
    let album = siren::album(&state, &cid).await?;
    Ok(([(header::CACHE_CONTROL, CATALOG_CACHE)], Json(album)).into_response())
}

/// Redirects to the track's audio on the label's CDN. The label signs each
/// stream URL for about three hours; this backend holds the signed URL and
/// asks for a new one shortly before it expires, so a player that re-requests
/// this path on each play always lands on a live URL. The CDN answers byte
/// ranges (`206`) and allows any origin, so a `<audio>` element with
/// `crossorigin="anonymous"` can feed Web Audio.
#[utoipa::path(
    get,
    path = "/music/siren/songs/{cid}/stream",
    operation_id = "music_siren_stream",
    tag = "music",
    params(("cid" = String, Path, description = "The label's song id.")),
    responses(
        (status = 302, description = "`Location` is the signed CDN URL (WAV, about 60 MB a track)."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn siren_stream(
    State(state): State<AppState>,
    Path(cid): Path<String>,
) -> Result<Response, ApiError> {
    let url = siren::stream_url(&state, &cid).await?;
    Ok((
        StatusCode::FOUND,
        [
            (header::LOCATION, url),
            // The signature is personal to the moment; a shared cache must
            // not hand an expired redirect to the next listener.
            (header::CACHE_CONTROL, "private, max-age=600".to_owned()),
        ],
    )
        .into_response())
}

/// The track's lyrics as an LRC file (`[mm:ss.xx]` timestamps, CRLF lines),
/// fetched from the label once and kept.
#[utoipa::path(
    get,
    path = "/music/siren/songs/{cid}/lyrics",
    operation_id = "music_siren_lyrics",
    tag = "music",
    params(("cid" = String, Path, description = "The label's song id.")),
    responses(
        (status = 200, description = "The LRC file.", content_type = "text/plain", body = String),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn siren_lyrics(
    State(state): State<AppState>,
    Path(cid): Path<String>,
) -> Result<Response, ApiError> {
    let text = siren::lyrics(&state, &cid).await?;
    Ok((
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::CACHE_CONTROL, LYRICS_CACHE),
        ],
        text.to_string(),
    )
        .into_response())
}

// ---------------------------------------------------------------------------
// The account's library
// ---------------------------------------------------------------------------

/// Everything the account keeps about music: liked tracks, recent plays with
/// per-track totals, and playlists. Track ids are `siren:<cid>` for a label
/// release and `game:<track id>` for an in-game track.
#[utoipa::path(
    get,
    path = "/music/library",
    operation_id = "music_library_get",
    tag = "music",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "The library. Empty lists for a fresh account.", body = MusicLibrary),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn library_get(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<MusicLibrary>, ApiError> {
    Ok(Json(library::get(&state, auth.user_uuid()?).await?))
}

/// Like a track. Liking it again is a no-op.
#[utoipa::path(
    put,
    path = "/music/likes/{track_id}",
    operation_id = "music_like",
    tag = "music",
    params(("track_id" = String, Path, description = "`siren:<cid>` or `game:<track id>`.")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Liked."),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn like(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(track_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    library::like(&state, auth.user_uuid()?, &track_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Remove a like.
#[utoipa::path(
    delete,
    path = "/music/likes/{track_id}",
    operation_id = "music_unlike",
    tag = "music",
    params(("track_id" = String, Path, description = "`siren:<cid>` or `game:<track id>`.")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "No longer liked."),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn unlike(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(track_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    library::unlike(&state, auth.user_uuid()?, &track_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Log a play. The page calls this once the listener has heard enough of the
/// track to count, then raises the time with `PUT /music/history/{id}`.
#[utoipa::path(
    post,
    path = "/music/history",
    operation_id = "music_history_log",
    tag = "music",
    request_body = MusicPlayInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "The play, with the id later updates name.", body = MusicPlay),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn history_log(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<MusicPlayInput>,
) -> Result<Json<MusicPlay>, ApiError> {
    Ok(Json(
        library::log_play(&state, auth.user_uuid()?, body).await?,
    ))
}

/// Raise a play's listened time. The stored time only grows.
#[utoipa::path(
    put,
    path = "/music/history/{id}",
    operation_id = "music_history_update",
    tag = "music",
    params(("id" = i64, Path, description = "The play id from `POST /music/history`.")),
    request_body = MusicPlayUpdate,
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Updated."),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn history_update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<MusicPlayUpdate>,
) -> Result<StatusCode, ApiError> {
    library::update_play(&state, auth.user_uuid()?, id, body).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Create a playlist, optionally with its first tracks.
#[utoipa::path(
    post,
    path = "/music/playlists",
    operation_id = "music_playlist_create",
    tag = "music",
    request_body = MusicPlaylistCreate,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "The playlist.", body = MusicPlaylist),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 409, response = crate::app::openapi::responses::Conflict),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn playlist_create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<MusicPlaylistCreate>,
) -> Result<Json<MusicPlaylist>, ApiError> {
    Ok(Json(
        library::create_playlist(&state, auth.user_uuid()?, body).await?,
    ))
}

/// Rename a playlist, set its full track order, or both.
#[utoipa::path(
    patch,
    path = "/music/playlists/{id}",
    operation_id = "music_playlist_patch",
    tag = "music",
    params(("id" = uuid::Uuid, Path, description = "The playlist id.")),
    request_body = MusicPlaylistPatch,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "The playlist as stored.", body = MusicPlaylist),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn playlist_patch(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<uuid::Uuid>,
    Json(body): Json<MusicPlaylistPatch>,
) -> Result<Json<MusicPlaylist>, ApiError> {
    Ok(Json(
        library::patch_playlist(&state, auth.user_uuid()?, id, body).await?,
    ))
}

/// Delete a playlist.
#[utoipa::path(
    delete,
    path = "/music/playlists/{id}",
    operation_id = "music_playlist_delete",
    tag = "music",
    params(("id" = uuid::Uuid, Path, description = "The playlist id.")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Deleted."),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn playlist_delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ApiError> {
    library::delete_playlist(&state, auth.user_uuid()?, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Append tracks to a playlist, skipping any already in it.
#[utoipa::path(
    post,
    path = "/music/playlists/{id}/tracks",
    operation_id = "music_playlist_add_tracks",
    tag = "music",
    params(("id" = uuid::Uuid, Path, description = "The playlist id.")),
    request_body = MusicPlaylistAdd,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "The playlist as stored.", body = MusicPlaylist),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn playlist_add_tracks(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<uuid::Uuid>,
    Json(body): Json<MusicPlaylistAdd>,
) -> Result<Json<MusicPlaylist>, ApiError> {
    Ok(Json(
        library::add_tracks(&state, auth.user_uuid()?, id, body).await?,
    ))
}

/// Remove one track from a playlist.
#[utoipa::path(
    delete,
    path = "/music/playlists/{id}/tracks/{track_id}",
    operation_id = "music_playlist_remove_track",
    tag = "music",
    params(
        ("id" = uuid::Uuid, Path, description = "The playlist id."),
        ("track_id" = String, Path, description = "`siren:<cid>` or `game:<track id>`.")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "The playlist as stored.", body = MusicPlaylist),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn playlist_remove_track(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, track_id)): Path<(uuid::Uuid, String)>,
) -> Result<Json<MusicPlaylist>, ApiError> {
    Ok(Json(
        library::remove_track(&state, auth.user_uuid()?, id, &track_id).await?,
    ))
}

/// The Monster Siren catalogue and the account's library. The in-game index
/// (`music`, `music_srv`) is registered by the caller next to the other
/// game-data routes.
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(siren_catalog))
        .routes(routes!(siren_album))
        .routes(routes!(siren_stream))
        .routes(routes!(siren_lyrics))
        .routes(routes!(library_get))
        .routes(routes!(like, unlike))
        .routes(routes!(history_log))
        .routes(routes!(history_update))
        .routes(routes!(playlist_create))
        .routes(routes!(playlist_patch, playlist_delete))
        .routes(routes!(playlist_add_tracks))
        .routes(routes!(playlist_remove_track))
}
