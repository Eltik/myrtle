//! The OAuth 2.0 provider: registering applications, the consent decision,
//! and the token, revocation and userinfo endpoints.
//!
//! The rules live in [`crate::core::auth::oauth`] and the statements in
//! [`crate::database::queries::oauth`]; this file is the order they run in.
//!
//! Two error types, by caller. A function the site's own web client reaches
//! with a site token returns [`ApiError`]. A function a third party's server
//! reaches returns [`OAuthError`], the RFC 6749 format its library expects.

use std::sync::LazyLock;

use axum::http::HeaderMap;
use uuid::Uuid;

use crate::app::error::{ApiError, FieldError, OAuthError};
use crate::app::state::AppState;
use crate::core::auth::oauth::{
    self, ACCESS_TTL_SECS, CODE_TTL_SECS, MAX_APPS_PER_USER, MAX_DESCRIPTION_LEN, MAX_NAME_LEN,
    MAX_REDIRECT_URIS, MAX_SESSIONS_PER_GRANT, MAX_STATE_LEN, REFRESH_TTL_SECS, Scope, SecretKind,
};
use crate::database::models::oauth::{
    AuthorizeCheck, AuthorizeDecision, AuthorizeDecisionInput, AuthorizeParams, OAuthApp,
    OAuthAppInput, OAuthAppPublicRow, OAuthAppWithSecret, OAuthAuthorization, OAuthTokenResponse,
    OAuthUserInfo,
};
use crate::database::queries::oauth as db;

/// The avatar a player who never picked one is shown with, as on the site.
const DEFAULT_AVATAR_ID: &str = "char_002_amiya";

/// The public base URL of this API, for the absolute `avatar_url` in
/// `/oauth/userinfo`. `PUBLIC_API_URL` when set; otherwise the request's own
/// `Host`, which is the address the application just reached this server on.
static PUBLIC_API_URL: LazyLock<Option<String>> = LazyLock::new(|| {
    std::env::var("PUBLIC_API_URL")
        .ok()
        .map(|v| v.trim().trim_end_matches('/').to_owned())
        .filter(|v| !v.is_empty())
});

/// See [`PUBLIC_API_URL`].
pub fn public_api_base(headers: &HeaderMap) -> String {
    api_base_from(PUBLIC_API_URL.as_deref(), headers)
}

/// The configured base, else `scheme://host` of the request. The scheme
/// follows `X-Forwarded-Proto`, which only ever shapes the caller's own
/// response.
fn api_base_from(configured: Option<&str>, headers: &HeaderMap) -> String {
    if let Some(base) = configured {
        return base.to_owned();
    }
    let header = |name: &str| headers.get(name).and_then(|v| v.to_str().ok());
    let Some(host) = header("host").filter(|h| !h.is_empty()) else {
        return "https://api.myrtle.moe".to_owned();
    };
    let scheme = match header("x-forwarded-proto").map(str::trim) {
        Some("https") => "https",
        _ => "http",
    };
    format!("{scheme}://{host}")
}

// ---------------------------------------------------------------------------
// Applications
// ---------------------------------------------------------------------------

/// An [`OAuthAppInput`] that passed validation, trimmed.
struct CleanApp {
    name: String,
    description: String,
    redirect_uris: Vec<String>,
}

fn field_error(field: &str, message: impl Into<String>) -> FieldError {
    FieldError {
        field: field.to_owned(),
        message: message.into(),
    }
}

/// Every problem with `input` at once, as a 422, so the form can mark each
/// field instead of surfacing them one save at a time.
fn clean_app_input(input: OAuthAppInput) -> Result<CleanApp, ApiError> {
    let mut errors = Vec::new();

    let name = input.name.trim().to_owned();
    let name_len = name.chars().count();
    if name_len == 0 {
        errors.push(field_error("name", "is required"));
    } else if name_len > MAX_NAME_LEN {
        errors.push(field_error(
            "name",
            format!("is longer than {MAX_NAME_LEN} characters"),
        ));
    } else if name.chars().any(char::is_control) {
        errors.push(field_error("name", "must not contain control characters"));
    }

    let description = input.description.unwrap_or_default().trim().to_owned();
    if description.chars().count() > MAX_DESCRIPTION_LEN {
        errors.push(field_error(
            "description",
            format!("is longer than {MAX_DESCRIPTION_LEN} characters"),
        ));
    } else if description.chars().any(char::is_control) {
        errors.push(field_error(
            "description",
            "must not contain line breaks or control characters",
        ));
    }

    // Kept in the order given, each once. Not trimmed: a URI is used exactly
    // as registered, so stray whitespace is an error to show, not to hide.
    let mut redirect_uris: Vec<String> = Vec::new();
    for uri in input.redirect_uris {
        if let Err(reason) = oauth::validate_redirect_uri(&uri) {
            errors.push(field_error("redirect_uris", format!("{uri:?} {reason}")));
        } else if !redirect_uris.contains(&uri) {
            redirect_uris.push(uri);
        }
    }
    if redirect_uris.is_empty() && errors.iter().all(|e| e.field != "redirect_uris") {
        errors.push(field_error(
            "redirect_uris",
            "at least one redirect URI is required",
        ));
    } else if redirect_uris.len() > MAX_REDIRECT_URIS {
        errors.push(field_error(
            "redirect_uris",
            format!("at most {MAX_REDIRECT_URIS} redirect URIs are allowed"),
        ));
    }

    if errors.is_empty() {
        Ok(CleanApp {
            name,
            description,
            redirect_uris,
        })
    } else {
        Err(ApiError::ValidationFailed(errors))
    }
}

pub async fn list_apps(state: &AppState, owner_id: Uuid) -> Result<Vec<OAuthApp>, ApiError> {
    let mut conn = state.db.acquire().await?;
    Ok(db::list_apps(&mut conn, owner_id).await?)
}

/// Register an application for `owner_id` and mint its first secret.
pub async fn create_app(
    state: &AppState,
    owner_id: Uuid,
    input: OAuthAppInput,
) -> Result<OAuthAppWithSecret, ApiError> {
    let clean = clean_app_input(input)?;
    let mut tx = state.db.begin().await?;

    // Lock the account row: two creates racing each other then count one
    // after the other, so the cap holds. No row means the token outlived its
    // account, which is 401 everywhere.
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE id = $1 FOR UPDATE")
        .bind(owner_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    if db::count_apps(&mut tx, owner_id).await? >= MAX_APPS_PER_USER {
        return Err(ApiError::Conflict(format!(
            "an account can register at most {MAX_APPS_PER_USER} applications"
        )));
    }

    let client_secret = oauth::mint(SecretKind::ClientSecret);
    let app = db::insert_app(
        &mut tx,
        owner_id,
        &clean.name,
        &clean.description,
        &clean.redirect_uris,
        &oauth::digest(&client_secret),
    )
    .await?;
    tx.commit().await?;

    tracing::info!(app = %app.id, "oauth application registered");
    Ok(OAuthAppWithSecret { app, client_secret })
}

pub async fn update_app(
    state: &AppState,
    owner_id: Uuid,
    app_id: Uuid,
    input: OAuthAppInput,
) -> Result<OAuthApp, ApiError> {
    let clean = clean_app_input(input)?;
    let mut conn = state.db.acquire().await?;
    db::update_app(
        &mut conn,
        app_id,
        owner_id,
        &clean.name,
        &clean.description,
        &clean.redirect_uris,
    )
    .await?
    .ok_or(ApiError::NotFound)
}

/// Replace an application's secret. The old one stops authenticating at once;
/// sign-ins already made keep working.
pub async fn rotate_secret(
    state: &AppState,
    owner_id: Uuid,
    app_id: Uuid,
) -> Result<OAuthAppWithSecret, ApiError> {
    let client_secret = oauth::mint(SecretKind::ClientSecret);
    let mut conn = state.db.acquire().await?;
    let app = db::replace_secret(&mut conn, app_id, owner_id, &oauth::digest(&client_secret))
        .await?
        .ok_or(ApiError::NotFound)?;
    tracing::info!(app = %app.id, "oauth client secret rotated");
    Ok(OAuthAppWithSecret { app, client_secret })
}

/// Delete an application, signing every player out of it. `owner_id` limits
/// the delete to that account's applications; `None` is the super admin
/// taking down anyone's.
pub async fn delete_app(
    state: &AppState,
    owner_id: Option<Uuid>,
    app_id: Uuid,
) -> Result<(), ApiError> {
    let mut conn = state.db.acquire().await?;
    if !db::delete_app(&mut conn, app_id, owner_id).await? {
        return Err(ApiError::NotFound);
    }
    tracing::info!(app = %app_id, by_admin = owner_id.is_none(), "oauth application deleted");
    Ok(())
}

// ---------------------------------------------------------------------------
// Consent
// ---------------------------------------------------------------------------

/// An authorization request that may be shown to the player.
struct ValidRequest {
    app: OAuthAppPublicRow,
    redirect_uri: String,
    scopes: Vec<Scope>,
    state: Option<String>,
    code_challenge: Option<String>,
}

/// What became of an authorization request: valid, or wrong in a way the
/// application is told about by redirect. (Wrong in a way it cannot be told
/// about is the `Err` of [`validate_request`].)
enum Checked {
    Valid(Box<ValidRequest>),
    Redirect(String),
}

/// The redirect that reports `error` to the application (RFC 6749 4.1.2.1).
fn error_redirect(
    redirect_uri: &str,
    error: &str,
    description: &str,
    state: Option<&str>,
) -> Result<String, ApiError> {
    let mut params = vec![("error", error), ("error_description", description)];
    if let Some(state) = state {
        params.push(("state", state));
    }
    oauth::redirect_with(redirect_uri, &params)
        .ok_or_else(|| ApiError::BadRequest("the redirect_uri is not a valid URL".to_owned()))
}

/// Check an authorization request, in the order RFC 6749 4.1.2.1 sets.
///
/// First the application and the redirect URI. Until both are known good
/// there is nowhere trustworthy to send the browser, so a failure is a 400
/// for the player to read: redirecting to an unregistered URI would make this
/// page an open redirect. After that every failure belongs to the application
/// and goes back to it as a redirect.
async fn validate_request(
    conn: &mut sqlx::PgConnection,
    params: &AuthorizeParams,
) -> Result<Checked, ApiError> {
    let client_id = params
        .client_id
        .as_deref()
        .ok_or_else(|| ApiError::BadRequest("client_id is required".to_owned()))?;
    let unknown = || ApiError::BadRequest("unknown client_id".to_owned());
    let app_id: Uuid = client_id.parse().map_err(|_| unknown())?;
    let app = db::find_public_app(conn, app_id)
        .await?
        .ok_or_else(unknown)?;

    let redirect_uri = match params.redirect_uri.as_deref() {
        Some(uri) => app
            .redirect_uris
            .iter()
            .find(|registered| registered.as_str() == uri)
            .cloned()
            .ok_or_else(|| {
                ApiError::BadRequest(
                    "redirect_uri is not one of the redirect URIs registered for this application"
                        .to_owned(),
                )
            })?,
        None => match app.redirect_uris.as_slice() {
            [only] => only.clone(),
            _ => {
                return Err(ApiError::BadRequest(
                    "redirect_uri is required: this application has more than one registered"
                        .to_owned(),
                ));
            }
        },
    };

    let state = params.state.as_deref();
    let fail = |error: &str, description: &str, state: Option<&str>| {
        error_redirect(&redirect_uri, error, description, state).map(Checked::Redirect)
    };

    if state.is_some_and(|s| s.len() > MAX_STATE_LEN) {
        // Too long to echo back, so the error travels without it.
        return fail("invalid_request", "state is too long", None);
    }
    match params.response_type.as_deref() {
        Some("code") => {}
        Some(_) => {
            return fail(
                "unsupported_response_type",
                "response_type must be code",
                state,
            );
        }
        None => return fail("invalid_request", "response_type is required", state),
    }
    let scopes = match oauth::parse_scopes(params.scope.as_deref()) {
        Ok(scopes) => scopes,
        Err(name) => return fail("invalid_scope", &format!("unknown scope: {name}"), state),
    };
    let code_challenge = match (
        params.code_challenge.as_deref(),
        params.code_challenge_method.as_deref(),
    ) {
        (None, None) => None,
        (Some(challenge), Some("S256")) if oauth::is_valid_challenge(challenge) => {
            Some(challenge.to_owned())
        }
        (Some(_), Some("S256")) => {
            return fail(
                "invalid_request",
                "code_challenge must be the base64url SHA-256 of the code verifier (43 characters)",
                state,
            );
        }
        // An absent method means `plain` in RFC 7636, which is refused too.
        (Some(_), _) => {
            return fail(
                "invalid_request",
                "code_challenge_method must be S256",
                state,
            );
        }
        (None, Some(_)) => {
            return fail(
                "invalid_request",
                "code_challenge_method was sent without a code_challenge",
                state,
            );
        }
    };

    Ok(Checked::Valid(Box::new(ValidRequest {
        app,
        redirect_uri,
        scopes,
        state: params.state.clone(),
        code_challenge,
    })))
}

/// What the consent page should show for an authorization request.
pub async fn check_authorization(
    state: &AppState,
    user_id: Uuid,
    params: &AuthorizeParams,
) -> Result<AuthorizeCheck, ApiError> {
    let mut conn = state.db.acquire().await?;
    let request = match validate_request(&mut conn, params).await? {
        Checked::Valid(request) => *request,
        Checked::Redirect(redirect_to) => return Ok(AuthorizeCheck::Redirect { redirect_to }),
    };
    let already_authorized = db::has_grant(&mut conn, request.app.id, user_id).await?;
    let redirect_host = oauth::redirect_host(&request.redirect_uri).unwrap_or_default();
    Ok(AuthorizeCheck::Consent {
        scopes: request.scopes.iter().map(ToString::to_string).collect(),
        redirect_uri: request.redirect_uri,
        redirect_host,
        already_authorized,
        app: request.app.into_public(),
    })
}

/// Act on the player's answer and say where the browser goes next.
///
/// The request is validated again from scratch: the body is the browser's,
/// and nothing the consent page was told earlier is trusted to still hold (an
/// application can be edited or deleted while the page is open).
pub async fn decide_authorization(
    state: &AppState,
    user_id: Uuid,
    input: AuthorizeDecisionInput,
) -> Result<AuthorizeDecision, ApiError> {
    let mut tx = state.db.begin().await?;
    let request = match validate_request(&mut tx, &input.params).await? {
        Checked::Valid(request) => *request,
        Checked::Redirect(redirect_to) => return Ok(AuthorizeDecision { redirect_to }),
    };

    if !input.approve {
        let redirect_to = error_redirect(
            &request.redirect_uri,
            "access_denied",
            "the player declined the request",
            request.state.as_deref(),
        )?;
        return Ok(AuthorizeDecision { redirect_to });
    }

    // A token that outlived its account must not leave a grant behind.
    if db::find_identity(&mut tx, user_id).await?.is_none() {
        return Err(ApiError::Unauthorized);
    }

    let scope = oauth::join_scopes(&request.scopes);
    let grant_id = db::upsert_grant(&mut tx, request.app.id, user_id, &scope).await?;
    let code = oauth::mint(SecretKind::Code);
    db::insert_code(
        &mut tx,
        &oauth::digest(&code),
        grant_id,
        &request.redirect_uri,
        &scope,
        request.code_challenge.as_deref(),
        CODE_TTL_SECS,
    )
    .await?;
    // Housekeeping rides the consent, the one write path every flow goes
    // through: no job to schedule, and the tables cannot outgrow their use.
    db::delete_expired(&mut tx).await?;
    tx.commit().await?;

    let mut params = vec![("code", code.as_str())];
    if let Some(state) = request.state.as_deref() {
        params.push(("state", state));
    }
    let redirect_to = oauth::redirect_with(&request.redirect_uri, &params)
        .ok_or_else(|| ApiError::BadRequest("the redirect_uri is not a valid URL".to_owned()))?;
    tracing::info!(app = %request.app.id, "oauth authorization granted");
    Ok(AuthorizeDecision { redirect_to })
}

pub async fn list_authorizations(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<OAuthAuthorization>, ApiError> {
    let mut conn = state.db.acquire().await?;
    let rows = db::list_authorizations(&mut conn, user_id).await?;
    Ok(rows
        .into_iter()
        .map(|row| OAuthAuthorization {
            scopes: oauth::stored_scopes(&row.scope)
                .iter()
                .map(ToString::to_string)
                .collect(),
            authorized_at: row.authorized_at,
            last_used_at: row.last_used_at,
            app: row.app.into_public(),
        })
        .collect())
}

/// Withdraw the caller's consent to an application; its tokens die with it.
pub async fn revoke_authorization(
    state: &AppState,
    user_id: Uuid,
    app_id: Uuid,
) -> Result<(), ApiError> {
    let mut conn = state.db.acquire().await?;
    if !db::delete_grant(&mut conn, app_id, user_id).await? {
        return Err(ApiError::NotFound);
    }
    tracing::info!(app = %app_id, "oauth authorization revoked by the player");
    Ok(())
}

// ---------------------------------------------------------------------------
// Protocol endpoints
// ---------------------------------------------------------------------------

/// Authenticate an application by id and secret (RFC 6749 2.3.1). An unknown
/// id and a wrong secret get the same answer.
pub async fn authenticate_client(
    state: &AppState,
    client_id: &str,
    client_secret: &str,
) -> Result<Uuid, OAuthError> {
    let app_id: Uuid = client_id
        .parse()
        .map_err(|_| OAuthError::invalid_client())?;
    let mut conn = state.db.acquire().await?;
    let stored = db::find_secret_hash(&mut conn, app_id)
        .await?
        .ok_or_else(OAuthError::invalid_client)?;
    if !oauth::matches_digest(client_secret, &stored) {
        return Err(OAuthError::invalid_client());
    }
    Ok(app_id)
}

/// Mint and store a session's token pair.
async fn issue_tokens(
    conn: &mut sqlx::PgConnection,
    grant_id: Uuid,
    scope: &str,
) -> Result<OAuthTokenResponse, OAuthError> {
    let access_token = oauth::mint(SecretKind::AccessToken);
    let refresh_token = oauth::mint(SecretKind::RefreshToken);
    db::insert_token(
        conn,
        grant_id,
        &oauth::digest(&access_token),
        &oauth::digest(&refresh_token),
        scope,
        ACCESS_TTL_SECS,
        REFRESH_TTL_SECS,
        MAX_SESSIONS_PER_GRANT,
    )
    .await?;
    Ok(OAuthTokenResponse {
        access_token,
        token_type: "Bearer",
        expires_in: ACCESS_TTL_SECS,
        refresh_token,
        scope: scope.to_owned(),
    })
}

/// `grant_type=authorization_code`: trade a code for tokens.
///
/// The code is spent by the first statement and stays spent whatever the
/// checks after it decide: a code that failed its redirect URI or its PKCE
/// verifier is not offered a second guess.
pub async fn exchange_code(
    state: &AppState,
    app_id: Uuid,
    code: &str,
    redirect_uri: Option<&str>,
    code_verifier: Option<&str>,
) -> Result<OAuthTokenResponse, OAuthError> {
    let invalid =
        || OAuthError::invalid_grant("the authorization code is invalid, expired or already used");
    let code_hash = oauth::digest(code);
    let mut conn = state.db.acquire().await?;

    let Some(spent) = db::consume_code(&mut conn, &code_hash, app_id).await? else {
        // A code presented twice means it leaked, or the application retried
        // an exchange whose answer it lost. Either way what the first
        // exchange issued can no longer be trusted (RFC 6749 4.1.2).
        if let Some(grant_id) = db::find_spent_code_grant(&mut conn, &code_hash, app_id).await? {
            let revoked = db::delete_grant_tokens(&mut conn, grant_id).await?;
            tracing::warn!(app = %app_id, revoked, "oauth authorization code replayed; sessions revoked");
        }
        return Err(invalid());
    };

    if redirect_uri.is_some_and(|uri| uri != spent.redirect_uri) {
        return Err(OAuthError::invalid_grant(
            "redirect_uri does not match the one the code was issued for",
        ));
    }
    match (spent.code_challenge.as_deref(), code_verifier) {
        (Some(challenge), Some(verifier)) if oauth::verify_pkce(verifier, challenge) => {}
        (Some(_), Some(_)) => {
            return Err(OAuthError::invalid_grant(
                "code_verifier does not match the code_challenge",
            ));
        }
        (Some(_), None) => {
            return Err(OAuthError::invalid_request(
                "code_verifier is required: the authorization request carried a code_challenge",
            ));
        }
        (None, Some(_)) => {
            return Err(OAuthError::invalid_grant(
                "code_verifier was sent, but the authorization request carried no code_challenge",
            ));
        }
        (None, None) => {}
    }

    let mut tx = sqlx::Acquire::begin(&mut *conn).await?;
    let tokens = issue_tokens(&mut tx, spent.grant_id, &spent.scope).await?;
    tx.commit().await?;
    Ok(tokens)
}

/// `grant_type=refresh_token`: trade a refresh token for a new pair. The
/// refresh token is single use; the response carries its replacement.
pub async fn refresh_tokens(
    state: &AppState,
    app_id: Uuid,
    refresh_token: &str,
    scope: Option<&str>,
) -> Result<OAuthTokenResponse, OAuthError> {
    // A refresh may narrow the scope but never widen it (RFC 6749 6). With one
    // scope there is nothing to narrow to, so naming a real scope changes
    // nothing and naming an unknown one is the error.
    if let Err(name) = oauth::parse_scopes(scope) {
        return Err(OAuthError::invalid_scope(format!("unknown scope: {name}")));
    }

    let access_token = oauth::mint(SecretKind::AccessToken);
    let new_refresh_token = oauth::mint(SecretKind::RefreshToken);
    let mut conn = state.db.acquire().await?;
    let scope = db::rotate_token(
        &mut conn,
        &oauth::digest(refresh_token),
        app_id,
        &oauth::digest(&access_token),
        &oauth::digest(&new_refresh_token),
        ACCESS_TTL_SECS,
        REFRESH_TTL_SECS,
    )
    .await?
    .ok_or_else(|| {
        OAuthError::invalid_grant("the refresh token is invalid, expired, revoked or already used")
    })?;

    Ok(OAuthTokenResponse {
        access_token,
        token_type: "Bearer",
        expires_in: ACCESS_TTL_SECS,
        refresh_token: new_refresh_token,
        scope,
    })
}

/// RFC 7009: end the session a token belongs to. An unknown token is not an
/// error: the application wanted it dead and it is.
pub async fn revoke_token(state: &AppState, app_id: Uuid, token: &str) -> Result<(), OAuthError> {
    let mut conn = state.db.acquire().await?;
    db::revoke_token(&mut conn, &oauth::digest(token), app_id).await?;
    Ok(())
}

/// The identity an access token speaks for.
pub async fn userinfo(
    state: &AppState,
    access_token: &str,
    api_base: &str,
) -> Result<OAuthUserInfo, OAuthError> {
    let mut conn = state.db.acquire().await?;
    let owner = db::find_access_token(&mut conn, &oauth::digest(access_token))
        .await?
        .ok_or_else(OAuthError::invalid_token)?;
    if !oauth::stored_scopes(&owner.scope).contains(&Scope::Identify) {
        return Err(OAuthError::insufficient_scope(Scope::Identify.as_str()));
    }
    // The grant cascades from the account, so a live token implies a row; a
    // delete landing between the two reads is the one way to miss it.
    let user = db::find_identity(&mut conn, owner.user_id)
        .await?
        .ok_or_else(OAuthError::invalid_token)?;

    let avatar = user.avatar_id.as_deref().unwrap_or(DEFAULT_AVATAR_ID);
    let avatar_url = format!("{api_base}/api/avatar/{}", oauth::avatar_stem(avatar));
    let username = user
        .nickname
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| user.uid.clone());
    Ok(OAuthUserInfo {
        id: user.id,
        username,
        tag: user.nick_number.filter(|t| !t.is_empty()),
        uid: user.uid,
        server: user.server.to_lowercase(),
        avatar_id: user.avatar_id,
        avatar_url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(name: &str, description: Option<&str>, uris: &[&str]) -> OAuthAppInput {
        OAuthAppInput {
            name: name.to_owned(),
            description: description.map(str::to_owned),
            redirect_uris: uris.iter().map(|u| (*u).to_owned()).collect(),
        }
    }

    fn fields(result: Result<CleanApp, ApiError>) -> Vec<String> {
        match result {
            Err(ApiError::ValidationFailed(errors)) => {
                errors.into_iter().map(|e| e.field).collect()
            }
            Err(other) => panic!("expected a validation failure, got {other:?}"),
            Ok(_) => Vec::new(),
        }
    }

    #[test]
    fn a_valid_application_is_trimmed_and_its_uris_deduplicated() {
        let clean = clean_app_input(input(
            "  miowawa.de ",
            Some(" connections "),
            &[
                "https://miowawa.de/cb",
                "http://localhost:4000/cb",
                "https://miowawa.de/cb",
            ],
        ))
        .expect("valid");
        assert_eq!(clean.name, "miowawa.de");
        assert_eq!(clean.description, "connections");
        assert_eq!(
            clean.redirect_uris,
            ["https://miowawa.de/cb", "http://localhost:4000/cb"]
        );
    }

    #[test]
    fn every_bad_field_is_reported_in_one_response() {
        let long_name = "n".repeat(MAX_NAME_LEN + 1);
        let long_description = "d".repeat(MAX_DESCRIPTION_LEN + 1);
        assert_eq!(
            fields(clean_app_input(input(
                &long_name,
                Some(&long_description),
                &["http://example.com/cb"]
            ))),
            ["name", "description", "redirect_uris"]
        );
        assert_eq!(
            fields(clean_app_input(input("   ", None, &[]))),
            ["name", "redirect_uris"]
        );
        assert_eq!(
            fields(clean_app_input(input(
                "a\nb",
                Some("x\ny"),
                &["https://a.example/cb"]
            ))),
            ["name", "description"]
        );
    }

    #[test]
    fn an_application_takes_at_most_ten_redirect_uris() {
        let uris: Vec<String> = (0..=MAX_REDIRECT_URIS)
            .map(|i| format!("https://example.com/cb/{i}"))
            .collect();
        let refs: Vec<&str> = uris.iter().map(String::as_str).collect();
        assert_eq!(
            fields(clean_app_input(input("app", None, &refs))),
            ["redirect_uris"]
        );
        assert!(clean_app_input(input("app", None, &refs[..MAX_REDIRECT_URIS])).is_ok());
    }

    #[test]
    fn the_api_base_is_the_configured_one_else_the_request_host() {
        let mut headers = HeaderMap::new();
        assert_eq!(api_base_from(None, &headers), "https://api.myrtle.moe");
        headers.insert("host", "localhost:3060".parse().expect("header"));
        assert_eq!(api_base_from(None, &headers), "http://localhost:3060");
        headers.insert("x-forwarded-proto", "https".parse().expect("header"));
        assert_eq!(api_base_from(None, &headers), "https://localhost:3060");
        assert_eq!(
            api_base_from(Some("https://api.example.com"), &headers),
            "https://api.example.com"
        );
    }
}
