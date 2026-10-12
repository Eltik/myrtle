//! Music: the In-Game Music index (every track the game names, where it comes
//! from and where it plays), the Monster Siren Records catalogue, and the
//! account's likes, history and playlists.

use axum::{
    Json,
    extract::{FromRequestParts, Path, Query, State},
    http::{HeaderMap, StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app::extractors::auth::AuthUser;
use crate::app::extractors::pagination::Pagination;
use crate::app::routes::static_data::shared_json_response;
use crate::app::services::music::stats::{self, StatClient};
use crate::app::services::music::{get_music_index, library, siren};
use crate::app::{error::ApiError, state::AppState};
use crate::core::hypergryph::constants::Server;
use crate::database::models::music::{
    MusicLibrary, MusicPlay, MusicPlayInput, MusicPlayUpdate, MusicPlaylist, MusicPlaylistAdd,
    MusicPlaylistCreate, MusicPlaylistPatch,
};
use crate::database::models::music_stats::{
    MusicListenerPage, MusicListenerSort, MusicStatInput, MusicStatKind, MusicStatRecorded,
    MusicStatWindow, MusicStatsCatalog, MusicTopTracks,
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

// ---------------------------------------------------------------------------
// Community stats
// ---------------------------------------------------------------------------

/// The community counters are kept this long by a browser or a proxy.
const STATS_CACHE: &str = "public, max-age=60";

/// Who an ingest call counts against: the account when the caller is signed
/// in, else an HMAC of their address. `None` when neither can be told (no
/// usable address), in which case the call counts nothing.
///
/// A call that presents credentials which fail (an expired or forged bearer)
/// is 401, not anonymous. `MaybeAuthUser` would count it against the address,
/// and the one caller that sends a bearer is the SSR server, whose address is
/// every visitor's: the 401 sends `recordMusicDownloadFn`'s caller back to the
/// browser, which reports from its own address.
pub struct StatCaller(Option<StatClient>);

impl FromRequestParts<AppState> for StatCaller {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if parts.headers.contains_key(header::AUTHORIZATION) {
            let auth = AuthUser::from_request_parts(parts, state).await?;
            return Ok(Self(Some(StatClient::User(auth.user_uuid()?))));
        }
        let client = stats::client_addr(&parts.headers, &parts.extensions)
            .map(|ip| stats::anonymous_client(&state.config.jwt_secret, ip));
        Ok(Self(client))
    }
}

async fn ingest(
    state: &AppState,
    caller: StatCaller,
    kind: MusicStatKind,
    body: &MusicStatInput,
) -> Result<Json<MusicStatRecorded>, ApiError> {
    let counted = match caller.0 {
        Some(client) => stats::record(state, kind, &body.track_id, &client).await?,
        None => {
            // Still answer a bad id as one, so a client bug shows.
            if !stats::is_known_track(state, &body.track_id).await? {
                return Err(ApiError::NotFound);
            }
            false
        }
    };
    Ok(Json(MusicStatRecorded { counted }))
}

/// Count a listen. Open to everyone: a signed-in caller counts by account,
/// anyone else by an HMAC of their address (never stored raw). The same
/// caller counts at most once per track every 30 minutes, so a page may call
/// this on every play that passes its listen rule. A signed-in play logged
/// with `POST /music/history` already counts here; calling both for one
/// play still counts once.
#[utoipa::path(
    post,
    path = "/music/stats/listen",
    operation_id = "music_stats_listen",
    tag = "music",
    request_body = MusicStatInput,
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "Whether this call counted.", body = MusicStatRecorded),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 404, description = "The catalogue lists no such track.", body = crate::app::error::ErrorBody),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn stats_listen(
    State(state): State<AppState>,
    caller: StatCaller,
    Json(body): Json<MusicStatInput>,
) -> Result<Json<MusicStatRecorded>, ApiError> {
    ingest(&state, caller, MusicStatKind::Listen, &body).await
}

/// Count a download. As `POST /music/stats/listen`, with a 24 hour window.
#[utoipa::path(
    post,
    path = "/music/stats/download",
    operation_id = "music_stats_download",
    tag = "music",
    request_body = MusicStatInput,
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "Whether this call counted.", body = MusicStatRecorded),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 404, description = "The catalogue lists no such track.", body = crate::app::error::ErrorBody),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn stats_download(
    State(state): State<AppState>,
    caller: StatCaller,
    Json(body): Json<MusicStatInput>,
) -> Result<Json<MusicStatRecorded>, ApiError> {
    ingest(&state, caller, MusicStatKind::Download, &body).await
}

/// All-time listens and downloads for every track that has any, keyed by
/// track id: one call covers the whole catalogue.
#[utoipa::path(
    get,
    path = "/music/stats",
    operation_id = "music_stats",
    tag = "music",
    responses(
        (status = 200, description = "Every counted track. Cached for a minute.", body = MusicStatsCatalog),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn stats_catalog(State(state): State<AppState>) -> Result<Response, ApiError> {
    let catalog = stats::catalog(&state).await?;
    Ok(([(header::CACHE_CONTROL, STATS_CACHE)], Json(catalog)).into_response())
}

#[derive(Deserialize)]
pub struct TopTracksParams {
    #[serde(default)]
    pub window: MusicStatWindow,
    pub limit: Option<u32>,
}

/// The most listened tracks over a window, everyone's listens counted.
#[utoipa::path(
    get,
    path = "/music/stats/top-tracks",
    operation_id = "music_stats_top_tracks",
    tag = "music",
    params(
        ("window" = Option<MusicStatWindow>, Query, description = "`week` (default, the last 7 days), `month` (the last 30) or `all`."),
        ("limit" = Option<u32>, Query, description = "How many tracks. Defaults to 50, capped at 100.")
    ),
    responses(
        (status = 200, description = "Tracks by listens, most first. Ties share a rank.", body = MusicTopTracks),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn stats_top_tracks(
    State(state): State<AppState>,
    Query(params): Query<TopTracksParams>,
) -> Result<Json<MusicTopTracks>, ApiError> {
    let limit = params.limit.unwrap_or(stats::TOP_TRACKS_DEFAULT);
    Ok(Json(stats::top_tracks(&state, params.window, limit).await?))
}

#[derive(Deserialize)]
pub struct ListenersParams {
    #[serde(default)]
    pub window: MusicStatWindow,
    #[serde(default)]
    pub sort: MusicListenerSort,
    #[serde(flatten)]
    pub pagination: Pagination,
}

/// One page of listeners for either listener route.
async fn listener_page(
    state: &AppState,
    params: &ListenersParams,
    track_id: Option<&str>,
) -> Result<Json<MusicListenerPage>, ApiError> {
    Ok(Json(
        stats::listeners(
            state,
            params.window,
            params.sort,
            track_id,
            params.pagination.limit(),
            params.pagination.offset(),
        )
        .await?,
    ))
}

/// Signed-in listeners ranked by counted listens or by time heard over a
/// window. Only public profiles are ranked, as on every user leaderboard;
/// ranks close over the hidden ones.
#[utoipa::path(
    get,
    path = "/music/stats/top-listeners",
    operation_id = "music_stats_top_listeners",
    tag = "music",
    params(
        ("window" = Option<MusicStatWindow>, Query, description = "`week` (default, the last 7 days), `month` (the last 30) or `all`."),
        ("sort" = Option<MusicListenerSort>, Query, description = "`listens` (default) or `time` (seconds heard)."),
        ("limit" = Option<u32>, Query, description = "Page size. Defaults to 20, capped at 100."),
        ("offset" = Option<u32>, Query, description = "Rows to skip. Defaults to 0.")
    ),
    responses(
        (status = 200, description = "A page of listeners and how many there are.", body = MusicListenerPage),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn stats_top_listeners(
    State(state): State<AppState>,
    Query(params): Query<ListenersParams>,
) -> Result<Json<MusicListenerPage>, ApiError> {
    listener_page(&state, &params, None).await
}

/// One track's signed-in listeners, ranked as `/music/stats/top-listeners`.
#[utoipa::path(
    get,
    path = "/music/stats/tracks/{track_id}/listeners",
    operation_id = "music_stats_track_listeners",
    tag = "music",
    params(
        ("track_id" = String, Path, description = "`siren:<cid>` or `game:<track id>`."),
        ("window" = Option<MusicStatWindow>, Query, description = "`week` (default, the last 7 days), `month` (the last 30) or `all`."),
        ("sort" = Option<MusicListenerSort>, Query, description = "`listens` (default) or `time` (seconds heard)."),
        ("limit" = Option<u32>, Query, description = "Page size. Defaults to 20, capped at 100."),
        ("offset" = Option<u32>, Query, description = "Rows to skip. Defaults to 0.")
    ),
    responses(
        (status = 200, description = "A page of the track's listeners and how many there are.", body = MusicListenerPage),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn stats_track_listeners(
    State(state): State<AppState>,
    Path(track_id): Path<String>,
    Query(params): Query<ListenersParams>,
) -> Result<Json<MusicListenerPage>, ApiError> {
    listener_page(&state, &params, Some(&track_id)).await
}

/// The Monster Siren catalogue, the account's library and the community
/// stats. The in-game index (`music`, `music_srv`) is registered by the
/// caller next to the other game-data routes.
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
        .routes(routes!(stats_listen))
        .routes(routes!(stats_download))
        .routes(routes!(stats_catalog))
        .routes(routes!(stats_top_tracks))
        .routes(routes!(stats_top_listeners))
        .routes(routes!(stats_track_listeners))
}
