use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::app::extractors::auth::{AuthUser, MaybeAuthUser};
use crate::app::routes::{StatusOk, resolve_uid};
use crate::app::{error::ApiError, services, state::AppState};
use crate::database::models::score::UserScore;
use crate::database::models::user::{UserCheckin, UserProfile};
use crate::database::queries::score::get_score_by_uid;
use crate::database::queries::users::{find_by_id, get_checkin_by_uid, update_role};

#[derive(Deserialize)]
pub struct GetUserParams {
    pub uid: String,
}

/// A player's profile. `UserProfile` carries currency, sanity, subscription
/// expiry, last-online time and the account's role, so the `uid` goes through
/// the shared privacy gate: the caller's own profile or a public one, 403
/// otherwise.
#[utoipa::path(
    get,
    path = "/get-user",
    tag = "player",
    params(
        ("uid" = String, Query, description = "Player to read.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "The profile.", body = UserProfile),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_user(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<GetUserParams>,
) -> Result<Json<UserProfile>, ApiError> {
    let uid = resolve_uid(&state, &auth, Some(&params.uid)).await?;
    let profile = services::user::get_user(&state, &uid).await?;
    Ok(Json(profile))
}

/// A player's graded score across every dimension.
/// Runs the shared privacy gate: another player's data is readable only when
/// their profile is public, and a player always sees their own.
#[utoipa::path(
    get,
    path = "/get-user-score",
    tag = "player",
    params(
        ("uid" = String, Query, description = "Player to read.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "The stored score row, or null when the player has never been graded.", body = Option<crate::database::models::score::UserScore>),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_user_score(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<GetUserParams>,
) -> Result<Json<Option<UserScore>>, ApiError> {
    // Typed row, not a `serde_json::Value` round-trip: same bytes on the wire,
    // but the type is the schema and the generated TS.
    let uid = resolve_uid(&state, &auth, Some(&params.uid)).await?;
    let score = get_score_by_uid(&state.db, &uid).await?;
    Ok(Json(score))
}

/// A player's monthly sign-in state.
///
/// Null when the player has no stored check-in row.
/// Runs the shared privacy gate: another player's data is readable only when
/// their profile is public, and a player always sees their own.
#[utoipa::path(
    get,
    path = "/get-user-checkin",
    tag = "player",
    params(
        ("uid" = String, Query, description = "Player to read.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "The sign-in state, or null.", body = Option<UserCheckin>),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_user_checkin(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<GetUserParams>,
) -> Result<Json<Option<UserCheckin>>, ApiError> {
    // `null` until the user has synced once.
    let uid = resolve_uid(&state, &auth, Some(&params.uid)).await?;
    let checkin = get_checkin_by_uid(&state.db, &uid).await?;
    Ok(Json(checkin))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SetRoleRequest {
    pub role: String,
}

/// `PUT /admin/users/{user_id}/role`
///
/// Until now the only way to change a role was the interactive
/// `manage_permissions` CLI, so the admin panel could display a role but never
/// assign one. Granting the Translator role has to be possible from the panel,
/// which is what this route is for.
///
/// Super-admin only, and it refuses to change a super-admin's own row: an
/// accidental self-demotion here would need the CLI to undo.
///
/// Note the role reaches `AuthUser` from the JWT, so the target user keeps
/// their old privileges until their token refreshes. Per-locale translation
/// grants deliberately do not work this way - they are read from the database
/// per request and take effect immediately.
#[utoipa::path(
    put,
    path = "/admin/users/{user_id}/role",
    tag = "admin",
    params(
        ("user_id" = String, Path, description = "Target account id (UUID).")
    ),
    request_body = SetRoleRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The role was set.", body = StatusOk),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn set_user_role(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
    Json(body): Json<SetRoleRequest>,
) -> Result<Json<StatusOk>, ApiError> {
    if !auth.role.is_super_admin() {
        return Err(ApiError::Forbidden);
    }

    let role: crate::core::auth::permissions::GlobalRole =
        body.role.parse().map_err(ApiError::BadRequest)?;

    let target = find_by_id(&state.db, user_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if auth.user_uuid()? == user_id {
        return Err(ApiError::BadRequest(
            "refusing to change your own role; use the manage_permissions CLI".to_owned(),
        ));
    }

    if target.role == role.to_string() {
        return Ok(crate::app::routes::ok_status());
    }

    update_role(&state.db, user_id, &role.to_string()).await?;
    state
        .cache
        .invalidate_by_prefix(&format!("user:{}", target.uid))
        .await;

    tracing::info!(
        actor = %auth.user_id,
        target = %user_id,
        from = %target.role,
        to = %role,
        "role changed"
    );

    Ok(crate::app::routes::ok_status())
}
