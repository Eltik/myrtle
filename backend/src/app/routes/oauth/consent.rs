use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::services::oauth;
use crate::app::state::AppState;
use crate::database::models::oauth::{
    AuthorizeCheck, AuthorizeDecision, AuthorizeDecisionInput, AuthorizeParams, OAuthAuthorization,
};

/// Check an authorization request before showing the player a consent screen.
///
/// This is the backend half of the authorization endpoint. A third party
/// sends the player's browser to `https://myrtle.moe/oauth/authorize` with
/// these same query parameters; that page calls here with the player's site
/// token. It is not called by the third party.
///
/// An unknown `client_id`, or a `redirect_uri` the application did not
/// register, is a 400: there is no trustworthy place to send the browser, so
/// the player is shown the error. Every other problem is reported to the
/// application, as a `redirect` result whose URL carries the `error`.
#[utoipa::path(
    get,
    path = "/oauth/authorize",
    operation_id = "oauth_authorize_check",
    tag = "oauth",
    params(
        ("response_type" = Option<String>, Query, description = "Must be `code`."),
        ("client_id" = Option<String>, Query, description = "The application's id."),
        ("redirect_uri" = Option<String>, Query, description = "One of the application's registered redirect URIs, exactly. May be left out when it registered only one."),
        ("scope" = Option<String>, Query, description = "Space-separated scopes. Defaults to `identify`, the only one."),
        ("state" = Option<String>, Query, description = "The application's own opaque value, returned to it unchanged. Up to 1024 bytes."),
        ("code_challenge" = Option<String>, Query, description = "PKCE: the unpadded base64url SHA-256 of the application's code verifier."),
        ("code_challenge_method" = Option<String>, Query, description = "PKCE: must be `S256` when a challenge is sent.")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "What the consent page should do: ask the player, or send the browser back with an error.", body = AuthorizeCheck),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn check(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<AuthorizeParams>,
) -> Result<Json<AuthorizeCheck>, ApiError> {
    let result = oauth::check_authorization(&state, auth.user_uuid()?, &params).await?;
    Ok(Json(result))
}

/// Record the player's answer to an authorization request.
///
/// Approving stores their consent and mints a single-use authorization code,
/// good for five minutes; `redirect_to` is the application's redirect URI
/// carrying that `code`. Declining returns the same URI carrying
/// `error=access_denied`. The request's `state` rides along either way. The
/// request is validated again here, exactly as `GET /oauth/authorize` does.
#[utoipa::path(
    post,
    path = "/oauth/authorize",
    operation_id = "oauth_authorize_decide",
    tag = "oauth",
    request_body = AuthorizeDecisionInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Where to send the player's browser.", body = AuthorizeDecision),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn decide(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<AuthorizeDecisionInput>,
) -> Result<Json<AuthorizeDecision>, ApiError> {
    let decision = oauth::decide_authorization(&state, auth.user_uuid()?, body).await?;
    Ok(Json(decision))
}

/// The applications the caller has authorized.
#[utoipa::path(
    get,
    path = "/oauth/authorizations",
    operation_id = "oauth_authorizations_list",
    tag = "oauth",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Most recently used first.", body = Vec<OAuthAuthorization>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list_authorizations(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<OAuthAuthorization>>, ApiError> {
    let list = oauth::list_authorizations(&state, auth.user_uuid()?).await?;
    Ok(Json(list))
}

/// Withdraw the caller's authorization of an application.
///
/// Every token the application holds for the caller stops working at once.
/// The application can ask again, and the player will see the consent screen.
#[utoipa::path(
    delete,
    path = "/oauth/authorizations/{app_id}",
    operation_id = "oauth_authorizations_revoke",
    tag = "oauth",
    params(("app_id" = Uuid, Path, description = "The application's id.")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "The application no longer has access."),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn revoke_authorization(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(app_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    oauth::revoke_authorization(&state, auth.user_uuid()?, app_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
