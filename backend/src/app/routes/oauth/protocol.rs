//! The endpoints a third party's server calls. They speak RFC 6749: form
//! bodies in, and [`OAuthError`] (`{"error": "..."}`) out instead of this
//! API's usual envelope.

use axum::Form;
use axum::Json;
use axum::extract::State;
use axum::extract::rejection::FormRejection;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::app::error::OAuthError;
use crate::app::services::oauth;
use crate::app::state::AppState;
use crate::database::models::oauth::{OAuthErrorBody, OAuthTokenResponse, OAuthUserInfo};

/// The body of `POST /oauth/token`, form-encoded.
#[derive(Deserialize, ToSchema)]
pub struct OAuthTokenRequest {
    /// `authorization_code` or `refresh_token`.
    #[schema(example = "authorization_code")]
    pub grant_type: Option<String>,
    /// With `authorization_code`: the code from the redirect.
    pub code: Option<String>,
    /// With `authorization_code`: the redirect URI the code was issued for.
    /// Checked when sent.
    pub redirect_uri: Option<String>,
    /// With `authorization_code`: the PKCE verifier. Required when the
    /// authorization request carried a `code_challenge`.
    pub code_verifier: Option<String>,
    /// With `refresh_token`: the refresh token.
    pub refresh_token: Option<String>,
    /// With `refresh_token`: optional, and may only name granted scopes.
    pub scope: Option<String>,
    /// The application's id, when not sent as HTTP Basic credentials.
    pub client_id: Option<String>,
    /// The application's secret, when not sent as HTTP Basic credentials.
    pub client_secret: Option<String>,
}

/// The body of `POST /oauth/revoke`, form-encoded.
#[derive(Deserialize, ToSchema)]
pub struct OAuthRevokeRequest {
    /// The access token or refresh token to revoke.
    pub token: Option<String>,
    /// Accepted and ignored: both kinds of token are looked up either way.
    pub token_type_hint: Option<String>,
    /// The application's id, when not sent as HTTP Basic credentials.
    pub client_id: Option<String>,
    /// The application's secret, when not sent as HTTP Basic credentials.
    pub client_secret: Option<String>,
}

/// A JSON body no cache may keep (RFC 6749 5.1).
fn no_store<T: Serialize>(body: T) -> Response {
    let mut response = Json(body).into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

/// A form body, or the protocol's error for one that did not parse (wrong
/// content type, a repeated field).
fn form_body<T>(form: Result<Form<T>, FormRejection>) -> Result<T, OAuthError> {
    form.map(|Form(body)| body).map_err(|_| {
        OAuthError::invalid_request(
            "the body must be application/x-www-form-urlencoded, each parameter at most once",
        )
    })
}

/// The application's id and secret, from HTTP Basic (RFC 6749 2.3.1, where
/// both halves are form-encoded before being joined) or from the body.
/// Sending both is allowed only when they name the same application.
fn client_credentials(
    headers: &HeaderMap,
    body_id: Option<&str>,
    body_secret: Option<&str>,
) -> Result<(String, String), OAuthError> {
    let basic = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            let (scheme, rest) = v.split_once(' ')?;
            scheme.eq_ignore_ascii_case("basic").then(|| rest.trim())
        });

    if let Some(encoded) = basic {
        let decoded = STANDARD
            .decode(encoded)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .ok_or_else(OAuthError::invalid_client)?;
        let (id, secret) = decoded
            .split_once(':')
            .ok_or_else(OAuthError::invalid_client)?;
        let unescape = |part: &str| {
            urlencoding::decode(part)
                .map(std::borrow::Cow::into_owned)
                .map_err(|_| OAuthError::invalid_client())
        };
        let (id, secret) = (unescape(id)?, unescape(secret)?);
        if body_id.is_some_and(|other| other != id) {
            return Err(OAuthError::invalid_request(
                "client_id in the body does not match the Authorization header",
            ));
        }
        return Ok((id, secret));
    }

    match (body_id, body_secret) {
        (Some(id), Some(secret)) => Ok((id.to_owned(), secret.to_owned())),
        _ => Err(OAuthError::invalid_client()),
    }
}

async fn authenticate(
    state: &AppState,
    headers: &HeaderMap,
    body_id: Option<&str>,
    body_secret: Option<&str>,
) -> Result<Uuid, OAuthError> {
    let (id, secret) = client_credentials(headers, body_id, body_secret)?;
    oauth::authenticate_client(state, &id, &secret).await
}

fn required<'a>(value: Option<&'a str>, name: &str) -> Result<&'a str, OAuthError> {
    value
        .filter(|v| !v.is_empty())
        .ok_or_else(|| OAuthError::invalid_request(format!("{name} is required")))
}

/// Exchange an authorization code, or a refresh token, for tokens.
///
/// The token endpoint of RFC 6749. The body is
/// `application/x-www-form-urlencoded`. The application authenticates with
/// its `client_id` and `client_secret`, as HTTP Basic credentials or as body
/// fields.
///
/// - `grant_type=authorization_code` with `code` (and `code_verifier` when
///   the authorization request used PKCE). A code works once and expires
///   after five minutes; presenting one a second time also revokes what the
///   first exchange issued.
/// - `grant_type=refresh_token` with `refresh_token`. The refresh token is
///   single use: the response carries a new one.
///
/// The access token lasts seven days, the refresh token ninety from its last
/// use. Errors use the RFC 6749 body (`error`, `error_description`), not this
/// API's usual envelope.
#[utoipa::path(
    post,
    path = "/oauth/token",
    operation_id = "oauth_token",
    tag = "oauth",
    request_body(content = OAuthTokenRequest, content_type = "application/x-www-form-urlencoded"),
    security(("oauth_client" = []), ()),
    responses(
        (status = 200, description = "A new access token and refresh token. `Cache-Control: no-store`.", body = OAuthTokenResponse),
        (status = 400, description = "`invalid_request`, `invalid_grant`, `invalid_scope` or `unsupported_grant_type`.", body = OAuthErrorBody),
        (status = 401, description = "`invalid_client`: unknown `client_id` or wrong `client_secret`.", body = OAuthErrorBody),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, description = "`server_error`.", body = OAuthErrorBody),
        (status = 503, description = "`temporarily_unavailable`: safe to retry.", body = OAuthErrorBody)
    )
)]
pub async fn token(
    State(state): State<AppState>,
    headers: HeaderMap,
    form: Result<Form<OAuthTokenRequest>, FormRejection>,
) -> Result<Response, OAuthError> {
    let body = form_body(form)?;
    let app_id = authenticate(
        &state,
        &headers,
        body.client_id.as_deref(),
        body.client_secret.as_deref(),
    )
    .await?;

    let tokens = match body.grant_type.as_deref() {
        Some("authorization_code") => {
            let code = required(body.code.as_deref(), "code")?;
            oauth::exchange_code(
                &state,
                app_id,
                code,
                body.redirect_uri.as_deref(),
                body.code_verifier.as_deref(),
            )
            .await?
        }
        Some("refresh_token") => {
            let refresh_token = required(body.refresh_token.as_deref(), "refresh_token")?;
            oauth::refresh_tokens(&state, app_id, refresh_token, body.scope.as_deref()).await?
        }
        Some(_) => return Err(OAuthError::unsupported_grant_type()),
        None => return Err(OAuthError::invalid_request("grant_type is required")),
    };
    Ok(no_store(tokens))
}

/// Revoke an access token or a refresh token.
///
/// RFC 7009. Either token ends the whole sign-in it belongs to: its access
/// token and its refresh token both stop working. The application
/// authenticates as it does at the token endpoint, and can only revoke
/// tokens it was issued. A token that is unknown or already dead is a 200
/// too: the caller wanted it gone and it is.
#[utoipa::path(
    post,
    path = "/oauth/revoke",
    operation_id = "oauth_revoke",
    tag = "oauth",
    request_body(content = OAuthRevokeRequest, content_type = "application/x-www-form-urlencoded"),
    security(("oauth_client" = []), ()),
    responses(
        (status = 200, description = "The token no longer works. Empty body."),
        (status = 400, description = "`invalid_request`: no `token`.", body = OAuthErrorBody),
        (status = 401, description = "`invalid_client`: unknown `client_id` or wrong `client_secret`.", body = OAuthErrorBody),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, description = "`server_error`.", body = OAuthErrorBody),
        (status = 503, description = "`temporarily_unavailable`: safe to retry.", body = OAuthErrorBody)
    )
)]
pub async fn revoke(
    State(state): State<AppState>,
    headers: HeaderMap,
    form: Result<Form<OAuthRevokeRequest>, FormRejection>,
) -> Result<Response, OAuthError> {
    let body = form_body(form)?;
    let app_id = authenticate(
        &state,
        &headers,
        body.client_id.as_deref(),
        body.client_secret.as_deref(),
    )
    .await?;
    let token = required(body.token.as_deref(), "token")?;
    oauth::revoke_token(&state, app_id, token).await?;
    Ok((
        StatusCode::OK,
        [(header::CACHE_CONTROL, HeaderValue::from_static("no-store"))],
    )
        .into_response())
}

/// The player an access token speaks for.
///
/// Send the access token as `Authorization: Bearer`. Needs the `identify`
/// scope. `id` is the stable key for a connected account; `username` is the
/// player's in-game nickname, which is not unique and can change.
///
/// A site token from `/login` is not accepted here, and an access token is
/// not accepted anywhere else in this API.
#[utoipa::path(
    get,
    path = "/oauth/userinfo",
    operation_id = "oauth_userinfo",
    tag = "oauth",
    security(("oauth_access_token" = [])),
    responses(
        (status = 200, description = "The player's identity. `Cache-Control: no-store`.", body = OAuthUserInfo),
        (status = 401, description = "`invalid_token`: the access token is missing, expired or revoked.", body = OAuthErrorBody),
        (status = 403, description = "`insufficient_scope`: the token was not granted `identify`.", body = OAuthErrorBody),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, description = "`server_error`.", body = OAuthErrorBody),
        (status = 503, description = "`temporarily_unavailable`: safe to retry.", body = OAuthErrorBody)
    )
)]
pub async fn userinfo(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, OAuthError> {
    let access_token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            let (scheme, rest) = v.split_once(' ')?;
            scheme.eq_ignore_ascii_case("bearer").then(|| rest.trim())
        })
        .filter(|token| !token.is_empty())
        .ok_or_else(OAuthError::invalid_token)?;
    let info: OAuthUserInfo =
        oauth::userinfo(&state, access_token, &oauth::public_api_base(&headers)).await?;
    Ok(no_store(info))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basic(raw: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {}", STANDARD.encode(raw))).expect("header"),
        );
        headers
    }

    fn code(result: Result<(String, String), OAuthError>) -> &'static str {
        result.expect_err("should be refused").code()
    }

    #[test]
    fn credentials_come_from_basic_or_from_the_body() {
        let pair = ("app".to_owned(), "secret".to_owned());
        assert_eq!(
            client_credentials(&basic("app:secret"), None, None).ok(),
            Some(pair.clone())
        );
        assert_eq!(
            client_credentials(&HeaderMap::new(), Some("app"), Some("secret")).ok(),
            Some(pair.clone())
        );
        // Basic wins, and a matching client_id in the body is harmless.
        assert_eq!(
            client_credentials(&basic("app:secret"), Some("app"), Some("other")).ok(),
            Some(pair)
        );
    }

    #[test]
    fn basic_credentials_are_form_decoded_and_split_at_the_first_colon() {
        assert_eq!(
            client_credentials(&basic("a%3Ab:s%20e:c"), None, None).ok(),
            Some(("a:b".to_owned(), "s e:c".to_owned()))
        );
    }

    #[test]
    fn missing_or_mangled_credentials_are_invalid_client() {
        assert_eq!(
            code(client_credentials(&HeaderMap::new(), None, None)),
            "invalid_client"
        );
        assert_eq!(
            code(client_credentials(&HeaderMap::new(), Some("app"), None)),
            "invalid_client"
        );
        assert_eq!(
            code(client_credentials(&basic("no-colon"), None, None)),
            "invalid_client"
        );
        let mut garbage = HeaderMap::new();
        garbage.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Basic !!!not-base64!!!"),
        );
        assert_eq!(
            code(client_credentials(&garbage, None, None)),
            "invalid_client"
        );
    }

    #[test]
    fn two_different_client_ids_in_one_request_are_refused() {
        assert_eq!(
            code(client_credentials(
                &basic("app:secret"),
                Some("other"),
                None
            )),
            "invalid_request"
        );
    }
}
