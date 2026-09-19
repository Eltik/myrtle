use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::routes::resolve_uid;
use crate::app::routes::static_data::json_response;
use crate::app::services::improvements::get_improvements;
use crate::app::state::AppState;

#[derive(Deserialize)]
pub struct ImprovementsParams {
    pub uid: Option<String>,
}

/// Ranked suggestions for what would raise this player's score most.
/// Runs the shared privacy gate: another player's data is readable only when
/// their profile is public, and a player always sees their own.
#[utoipa::path(
    get,
    path = "/user/improvements",
    tag = "player",
    params(
        ("uid" = Option<String>, Query, description = "Player to read. Omitted means the caller's own account, which then requires a token."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "Suggestions, served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", content_type = "application/json"),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_user_improvements(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    headers: HeaderMap,
    Query(params): Query<ImprovementsParams>,
) -> Result<Response, ApiError> {
    // The access gate stays HERE, ahead of the cache read inside
    // `get_improvements`, so a cached body can never be served to a viewer who
    // would not have been allowed to build it.
    let uid = resolve_uid(&state, &auth, params.uid.as_deref()).await?;
    // Admission control lives inside the service now, around the one synchronous
    // pass that actually burns CPU. Taking it here held a permit across five
    // database round-trips too, which is not what it is for and is what made this
    // route shed under trivial concurrency.
    let cached = get_improvements(&state, &uid).await?;
    Ok(json_response(cached, &headers))
}
