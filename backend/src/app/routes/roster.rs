use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::routes::resolve_user_id;
use crate::app::state::AppState;
use crate::database::models::roster::{RosterEntry, SupportUnit};
use crate::database::queries::roster;

#[derive(Deserialize)]
pub struct RosterParams {
    pub uid: Option<String>,
}

/// Every operator a player owns, with level, promotion, skills and modules.
/// Runs the shared privacy gate: another player's data is readable only when
/// their profile is public, and a player always sees their own.
#[utoipa::path(
    get,
    path = "/roster",
    tag = "player",
    params(
        ("uid" = Option<String>, Query, description = "Player to read. Omitted means the caller's own account, which then requires a token.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "The player's owned operators.", body = Vec<RosterEntry>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_roster(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<RosterParams>,
) -> Result<Json<Vec<RosterEntry>>, ApiError> {
    let user_id = resolve_user_id(&state, &auth, params.uid.as_deref()).await?;
    let entries = roster::get_roster(&state.db, user_id).await?;
    Ok(Json(entries))
}

/// One owned operator from a player's roster.
/// Runs the shared privacy gate: another player's data is readable only when
/// their profile is public, and a player always sees their own.
#[utoipa::path(
    get,
    path = "/roster/{operator_id}",
    tag = "player",
    params(
        ("operator_id" = String, Path, description = "Operator id, e.g. `char_002_amiya`."),
        ("uid" = Option<String>, Query, description = "Player to read. Omitted means the caller's own account, which then requires a token.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "The player's copy of that operator.", body = RosterEntry),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_operator(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Path(operator_id): Path<String>,
    Query(params): Query<RosterParams>,
) -> Result<Json<RosterEntry>, ApiError> {
    let user_id = resolve_user_id(&state, &auth, params.uid.as_deref()).await?;
    let entry = roster::get_operator(&state.db, user_id, &operator_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(entry))
}

/// The player's support units, as other players would borrow them.
/// Runs the shared privacy gate: another player's data is readable only when
/// their profile is public, and a player always sees their own.
#[utoipa::path(
    get,
    path = "/get-user-supports",
    tag = "player",
    params(
        ("uid" = Option<String>, Query, description = "Player to read. Omitted means the caller's own account, which then requires a token.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "The configured support slots.", body = Vec<SupportUnit>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_supports(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<RosterParams>,
) -> Result<Json<Vec<SupportUnit>>, ApiError> {
    let user_id = resolve_user_id(&state, &auth, params.uid.as_deref()).await?;
    let entries = roster::get_supports(&state.db, user_id).await?;
    Ok(Json(entries))
}
