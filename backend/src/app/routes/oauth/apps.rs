use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::{AuthUser, RoleCheckedUser};
use crate::app::services::oauth;
use crate::app::state::AppState;
use crate::database::models::oauth::{OAuthApp, OAuthAppInput, OAuthAppWithSecret};

/// The OAuth applications the caller has registered.
#[utoipa::path(
    get,
    path = "/oauth/apps",
    operation_id = "oauth_apps_list",
    tag = "oauth",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Oldest first. No secrets: one is readable only in the response that minted it.", body = Vec<OAuthApp>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<OAuthApp>>, ApiError> {
    Ok(Json(oauth::list_apps(&state, auth.user_uuid()?).await?))
}

/// Register an OAuth application.
///
/// The response carries the `client_secret`. It is stored only as a hash, so
/// this is the one time it can be read; a lost secret is replaced with
/// `POST /oauth/apps/{id}/secret`. An account may register ten applications.
#[utoipa::path(
    post,
    path = "/oauth/apps",
    operation_id = "oauth_apps_create",
    tag = "oauth",
    request_body = OAuthAppInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "The application, with its client secret.", body = OAuthAppWithSecret),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 409, response = crate::app::openapi::responses::Conflict),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<OAuthAppInput>,
) -> Result<(StatusCode, Json<OAuthAppWithSecret>), ApiError> {
    let app = oauth::create_app(&state, auth.user_uuid()?, body).await?;
    Ok((StatusCode::CREATED, Json(app)))
}

/// Replace an application's name, description and redirect URIs.
///
/// A redirect URI that is removed stops being accepted by the consent page at
/// once. Sign-ins already made are unaffected.
#[utoipa::path(
    put,
    path = "/oauth/apps/{id}",
    operation_id = "oauth_apps_update",
    tag = "oauth",
    params(("id" = Uuid, Path, description = "The application's id (its `client_id`).")),
    request_body = OAuthAppInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "The application as stored.", body = OAuthApp),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<OAuthAppInput>,
) -> Result<Json<OAuthApp>, ApiError> {
    let app = oauth::update_app(&state, auth.user_uuid()?, id, body).await?;
    Ok(Json(app))
}

/// Delete an application.
///
/// Every player's authorization of it goes too, and every token it holds
/// stops working. Not undoable. Someone else's application answers 404, the
/// same as one that does not exist.
#[utoipa::path(
    delete,
    path = "/oauth/apps/{id}",
    operation_id = "oauth_apps_delete",
    tag = "oauth",
    params(("id" = Uuid, Path, description = "The application's id (its `client_id`).")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "The application is gone."),
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
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    oauth::delete_app(&state, Some(auth.user_uuid()?), id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Take down any OAuth application, whoever registered it.
///
/// Super-admin only. For an application that impersonates another or abuses
/// the consent screen: its id is the `client_id` in the authorization link it
/// sends players to. Every player is signed out of it and its tokens stop
/// working. Not undoable.
#[utoipa::path(
    delete,
    path = "/admin/oauth/apps/{id}",
    operation_id = "admin_oauth_apps_delete",
    tag = "admin",
    params(("id" = Uuid, Path, description = "The application's id (its `client_id`).")),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 204, description = "The application is gone."),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn admin_delete(
    State(state): State<AppState>,
    auth: RoleCheckedUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !auth.role.is_super_admin() {
        return Err(ApiError::Forbidden);
    }
    oauth::delete_app(&state, None, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Replace an application's client secret.
///
/// The old secret stops working at once and the response carries the new one,
/// readable this one time. Tokens already issued keep working: the secret
/// authenticates the application, not the sign-ins it has made.
#[utoipa::path(
    post,
    path = "/oauth/apps/{id}/secret",
    operation_id = "oauth_apps_rotate_secret",
    tag = "oauth",
    params(("id" = Uuid, Path, description = "The application's id (its `client_id`).")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "The application, with its new client secret.", body = OAuthAppWithSecret),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn rotate_secret(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<OAuthAppWithSecret>, ApiError> {
    let app = oauth::rotate_secret(&state, auth.user_uuid()?, id).await?;
    Ok(Json(app))
}
