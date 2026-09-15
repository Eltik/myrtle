use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::app::extractors::auth::{AuthUser, MaybeAuthUser};
use crate::app::routes::resolve_uid;
use crate::app::{error::ApiError, services, state::AppState};
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
pub async fn get_user(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<GetUserParams>,
) -> Result<Json<UserProfile>, ApiError> {
    let uid = resolve_uid(&state, &auth, Some(&params.uid)).await?;
    let profile = services::user::get_user(&state, &uid).await?;
    Ok(Json(profile))
}

pub async fn get_user_score(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<GetUserParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Return the full user_scores row (all category scores + grade + timestamp)
    // for the Score tab's detailed breakdown.
    let uid = resolve_uid(&state, &auth, Some(&params.uid)).await?;
    let score = get_score_by_uid(&state.db, &uid).await?;
    let body = match score {
        Some(s) => serde_json::to_value(&s).map_err(|e| ApiError::Internal(e.into()))?,
        None => serde_json::Value::Null,
    };
    Ok(Json(body))
}

pub async fn get_user_checkin(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<GetUserParams>,
) -> Result<Json<Option<UserCheckin>>, ApiError> {
    // Daily sign-in state: the month's claim count and per-claim monthly-card
    // flags, the lifetime total, and the active series' progress. `null` when
    // the user has never synced.
    let uid = resolve_uid(&state, &auth, Some(&params.uid)).await?;
    let checkin = get_checkin_by_uid(&state.db, &uid).await?;
    Ok(Json(checkin))
}

#[derive(Deserialize)]
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
pub async fn set_user_role(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
    Json(body): Json<SetRoleRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
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
