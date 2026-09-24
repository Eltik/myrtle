//! `GET`/`PUT /user/story-progress`, and `POST /user/story-progress/import`:
//! the reader's progress on the account, and the game's verdict re-derived
//! from what the database holds.
//!
//! Both sit behind the same bearer token every other `/user/...` route uses and
//! answer 401 signed out. The document travels whole in each direction; the
//! merge that decides what a round trip produces runs in the browser
//! (`frontend/src/lib/story/sync.ts`), because only the browser holds both
//! sides.
use axum::{Json, extract::State};

use crate::app::extractors::auth::AuthUser;
use crate::app::services::story_progress::{
    self as svc, StoryProgressDocument, StoryProgressResponse,
};
use crate::app::{error::ApiError, state::AppState};

/// The caller's stored reading progress, or null when nothing is stored yet.
#[utoipa::path(
    get,
    path = "/user/story-progress",
    tag = "player",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "The stored document and when it was written, both null when the account has never synced.", body = StoryProgressResponse),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_story_progress(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<StoryProgressResponse>, ApiError> {
    Ok(Json(svc::get(&state, auth.user_uuid()?).await?))
}

/// Replace the caller's stored reading progress with the document sent.
#[utoipa::path(
    put,
    path = "/user/story-progress",
    tag = "player",
    request_body = StoryProgressDocument,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "The document as stored, and when it was written.", body = StoryProgressResponse),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn put_story_progress(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<StoryProgressDocument>,
) -> Result<Json<StoryProgressResponse>, ApiError> {
    Ok(Json(svc::put(&state, auth.user_uuid()?, body).await?))
}

/// Re-derive which stories the GAME has read from the account's stored game
/// data, without refreshing it, and answer as `GET` would.
///
/// The "Sync now" button calls this: it reaches the database, never the game
/// server, so it is safe to press while the game is open.
#[utoipa::path(
    post,
    path = "/user/story-progress/import",
    tag = "player",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "The stored document and the re-derived game verdict.", body = StoryProgressResponse),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn import_story_progress(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<StoryProgressResponse>, ApiError> {
    let server = crate::app::services::auth::parse_server(&auth.server)?;
    Ok(Json(
        svc::reverdict(&state, auth.user_uuid()?, server).await?,
    ))
}
