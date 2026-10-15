use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::routes::tier_lists::ServerQuery;
use crate::app::services::squad;
use crate::app::state::AppState;
use crate::database::models::squad::{SavedSquad, SavedSquadInput, SquadsResponse};

/// The caller's saved squads, one entry per slot, and the in-game squads from
/// their last roster sync.
#[utoipa::path(
    get,
    path = "/squads",
    operation_id = "squads_get",
    tag = "squads",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "`saved` has one entry per slot, `null` for an empty one; `synced` is empty until the account syncs.", body = SquadsResponse),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_all(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<SquadsResponse>, ApiError> {
    let squads = squad::get(&state, auth.user_uuid()?).await?;
    Ok(Json(squads))
}

/// Save a squad into a slot, replacing whatever was there. A blank name is
/// stored as "Squad N".
#[utoipa::path(
    put,
    path = "/squads/{slot}",
    operation_id = "squad_save",
    tag = "squads",
    params(
        ("slot" = i32, Path, description = "1-based slot, 1 to 4."),
        ("server" = Option<String>, Query, description = "Game server whose operators the squad is checked against: `en`, `jp`, `kr`, `cn`, `tw` or `bili`. Defaults to the deployment's default server.")
    ),
    request_body = SavedSquadInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "The squad as stored, `members` padded to twelve seats.", body = SavedSquad),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn save(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slot): Path<i32>,
    Query(query): Query<ServerQuery>,
    Json(body): Json<SavedSquadInput>,
) -> Result<Json<SavedSquad>, ApiError> {
    let server = query.or_default(&state);
    let saved = squad::save(&state, auth.user_uuid()?, slot, body, server).await?;
    Ok(Json(saved))
}

/// Empty a slot.
#[utoipa::path(
    delete,
    path = "/squads/{slot}",
    operation_id = "squad_delete",
    tag = "squads",
    params(
        ("slot" = i32, Path, description = "1-based slot, 1 to 4.")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "The slot is empty."),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slot): Path<i32>,
) -> Result<StatusCode, ApiError> {
    squad::delete(&state, auth.user_uuid()?, slot).await?;
    Ok(StatusCode::NO_CONTENT)
}
