//! `GET /story/index`, `GET /story/{story_id}`,
//! `GET /story/group/{group_id}/illustrations`,
//! `GET /story/group/{group_id}/archive` and `GET /story/community`: the
//! Archives library, one parsed script, one group's art, one group's archive
//! and what the community has read. See `docs/story-reader.md` for the wire
//! contract.

use axum::{
    Json,
    extract::{Path, State},
};

use crate::app::services::story::{
    StoryArchive, StoryIllustrations, StoryIndex, get_group_archive, get_group_illustrations,
    get_story, get_story_index,
};
use crate::app::services::story_community::{self, StoryCommunity};
use crate::app::{error::ApiError, state::AppState};
use crate::core::hypergryph::constants::Server;
use crate::core::story::StoryScript;

/// The Archives library on the default server: every story group with its
/// stories, and every operator's records. `hasScript` says whether
/// `GET /story/{id}` will answer with a script.
#[utoipa::path(
    get,
    path = "/story/index",
    operation_id = "story_index",
    tag = "gamedata",
    responses(
        (status = 200, description = "The story library.", body = StoryIndex),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn index(State(state): State<AppState>) -> Result<Json<StoryIndex>, ApiError> {
    Ok(Json(get_story_index(&state, state.default_server).await?))
}

/// `GET /{server}/story/index` - the story library from `{server}`. The
/// `/{server}` form reads that server's game data; the bare form reads the
/// default server.
#[utoipa::path(
    get,
    path = "/{server}/story/index",
    operation_id = "story_index_srv",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`.")
    ),
    responses(
        (status = 200, description = "The story library.", body = StoryIndex),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn index_srv(
    State(state): State<AppState>,
    Path(server): Path<Server>,
) -> Result<Json<StoryIndex>, ApiError> {
    Ok(Json(get_story_index(&state, server).await?))
}

/// One story on the default server: its script as a generic command stream
/// and every asset it references, resolved to `/api/assets` paths. 404 when
/// the id is unknown, and 404 with a message when the id is in the library
/// but no script file backs it.
#[utoipa::path(
    get,
    path = "/story/{story_id}",
    operation_id = "story_detail",
    tag = "gamedata",
    params(
        ("story_id" = String, Path, description = "Story id from the library (`1stact_level_a001_01_beg`).")
    ),
    responses(
        (status = 200, description = "The parsed script.", body = StoryScript),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn detail(
    State(state): State<AppState>,
    Path(story_id): Path<String>,
) -> Result<Json<StoryScript>, ApiError> {
    get_story(&state, state.default_server, &story_id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

/// `GET /{server}/story/{story_id}` - one story from `{server}`. The
/// `/{server}` form reads that server's game data; the bare form reads the
/// default server.
#[utoipa::path(
    get,
    path = "/{server}/story/{story_id}",
    operation_id = "story_detail_srv",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("story_id" = String, Path, description = "Story id from the library (`1stact_level_a001_01_beg`).")
    ),
    responses(
        (status = 200, description = "The parsed script.", body = StoryScript),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn detail_srv(
    State(state): State<AppState>,
    Path((server, story_id)): Path<(Server, String)>,
) -> Result<Json<StoryScript>, ApiError> {
    get_story(&state, server, &story_id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

/// Every background, CG and character sprite a group's stories reference, on
/// the default server. `group_id` is a `StoryGroup.id` (`main_0`, `act11side`,
/// `story_kroos_set_1`) or an operator `charId` (`char_124_kroos`) for that
/// operator's records: one route rather than two, because an operator's
/// records are split over one or two story sets and have no single group id,
/// while the two id spaces cannot collide (every operator id starts with
/// `char_`, no group id does). Answered from the warmed index, so no script
/// is read. A name with no file on disk is listed with `url: null`.
#[utoipa::path(
    get,
    path = "/story/group/{group_id}/illustrations",
    operation_id = "story_group_illustrations",
    tag = "gamedata",
    params(
        ("group_id" = String, Path, description = "Story group id or an operator `charId`.")
    ),
    responses(
        (status = 200, description = "The group's illustrations.", body = StoryIllustrations),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn illustrations(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
) -> Result<Json<StoryIllustrations>, ApiError> {
    get_group_illustrations(&state, state.default_server, &group_id)
        .await
        .map(Json)
}

/// `GET /{server}/story/group/{group_id}/illustrations` - the same from
/// `{server}`. The `/{server}` form reads that server's game data; the bare
/// form reads the default server.
#[utoipa::path(
    get,
    path = "/{server}/story/group/{group_id}/illustrations",
    operation_id = "story_group_illustrations_srv",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("group_id" = String, Path, description = "Story group id or an operator `charId`.")
    ),
    responses(
        (status = 200, description = "The group's illustrations.", body = StoryIllustrations),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn illustrations_srv(
    State(state): State<AppState>,
    Path((server, group_id)): Path<(Server, String)>,
) -> Result<Json<StoryIllustrations>, ApiError> {
    get_group_illustrations(&state, server, &group_id)
        .await
        .map(Json)
}

/// The game's own "from the archive" screen for one group: its logs,
/// landmarks, news, files, gallery, soundtrack and recordings, in the order
/// the game lists them, with every picture and clip resolved to an
/// `/api/assets` path. Only the sections the group HAS are listed, so a group
/// the archive table does not carry answers 200 with an empty `sections`
/// rather than a 404; an id the library does not list at all is a 404, the
/// same as the illustrations route. Answered from the warmed index, so no
/// table is re-read.
#[utoipa::path(
    get,
    path = "/story/group/{group_id}/archive",
    operation_id = "story_group_archive",
    tag = "gamedata",
    params(
        ("group_id" = String, Path, description = "Story group id or an operator `charId`.")
    ),
    responses(
        (status = 200, description = "The group's archive.", body = StoryArchive),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn archive(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
) -> Result<Json<StoryArchive>, ApiError> {
    get_group_archive(&state, state.default_server, &group_id)
        .await
        .map(Json)
}

/// `GET /{server}/story/group/{group_id}/archive` - the same from `{server}`.
/// The `/{server}` form reads that server's game data; the bare form reads
/// the default server.
#[utoipa::path(
    get,
    path = "/{server}/story/group/{group_id}/archive",
    operation_id = "story_group_archive_srv",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("group_id" = String, Path, description = "Story group id or an operator `charId`.")
    ),
    responses(
        (status = 200, description = "The group's archive.", body = StoryArchive),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn archive_srv(
    State(state): State<AppState>,
    Path((server, group_id)): Path<(Server, String)>,
) -> Result<Json<StoryArchive>, ApiError> {
    get_group_archive(&state, server, &group_id).await.map(Json)
}

/// What the community has READ, on the default server: how many accounts have
/// opened each story, and how far into each group they get.
///
/// Anonymous throughout: every field is a count over accounts and nothing
/// names one. An account is counted only while it has turned neither
/// `publicProfile` nor `shareStats` off. The aggregate is computed off the
/// request path and recomputed at most every six hours, so `computedAt` is how
/// old the numbers are rather than when they were asked for.
#[utoipa::path(
    get,
    path = "/story/community",
    operation_id = "story_community",
    tag = "gamedata",
    responses(
        (status = 200, description = "The community reading aggregate.", body = StoryCommunity),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn community(State(state): State<AppState>) -> Result<Json<StoryCommunity>, ApiError> {
    Ok(Json(
        (*story_community::cached(&state, state.default_server).await?).clone(),
    ))
}

/// `GET /{server}/story/community` - the same from `{server}`. The `/{server}`
/// form reads that server's game data and its accounts; the bare form reads
/// the default server.
#[utoipa::path(
    get,
    path = "/{server}/story/community",
    operation_id = "story_community_srv",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`.")
    ),
    responses(
        (status = 200, description = "The community reading aggregate.", body = StoryCommunity),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn community_srv(
    State(state): State<AppState>,
    Path(server): Path<Server>,
) -> Result<Json<StoryCommunity>, ApiError> {
    Ok(Json(
        (*story_community::cached(&state, server).await?).clone(),
    ))
}
