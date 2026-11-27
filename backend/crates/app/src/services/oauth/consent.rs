use myrtle_db::Conn;
use myrtle_db::repos::oauth::{self as repo, AppPublicRow};
use myrtle_types::UserId;
use uuid::Uuid;

use super::dto::{
    AuthorizeCheck, AuthorizeDecision, AuthorizeDecisionInput, AuthorizeParams, OAuthAuthorization,
};
use super::protocol::{self, CODE_TTL_SECS, MAX_STATE_LEN, Scope, SecretKind};
use crate::policy::Subject;
use crate::{AppContext, AppError};

struct ValidRequest {
    app: AppPublicRow,
    redirect_uri: String,
    scopes: Vec<Scope>,
    state: Option<String>,
    code_challenge: Option<String>,
}

enum Checked {
    Valid(Box<ValidRequest>),
    Redirect(String),
}

fn player(subject: &Subject) -> Result<UserId, AppError> {
    subject.user_id().ok_or(AppError::Unauthorized)
}

fn invalid_redirect() -> AppError {
    AppError::bad_request("the redirect_uri is not a valid URL")
}

fn error_redirect(
    redirect_uri: &str,
    error: &str,
    description: &str,
    state: Option<&str>,
) -> Result<String, AppError> {
    let mut params = vec![("error", error), ("error_description", description)];
    if let Some(state) = state {
        params.push(("state", state));
    }
    protocol::redirect_with(redirect_uri, &params).ok_or_else(invalid_redirect)
}

async fn validate(conn: &mut Conn, params: &AuthorizeParams) -> Result<Checked, AppError> {
    let client_id = params
        .client_id
        .as_deref()
        .ok_or_else(|| AppError::bad_request("client_id is required"))?;
    let unknown = || AppError::bad_request("unknown client_id");
    let app_id: Uuid = client_id.parse().map_err(|_| unknown())?;
    let app = repo::find_public_app(conn, app_id)
        .await?
        .ok_or_else(unknown)?;

    let redirect_uri = match params.redirect_uri.as_deref() {
        Some(uri) => app
            .redirect_uris
            .iter()
            .find(|registered| registered.as_str() == uri)
            .cloned()
            .ok_or_else(|| {
                AppError::bad_request(
                    "redirect_uri is not one of the redirect URIs registered for this application",
                )
            })?,
        None => match app.redirect_uris.as_slice() {
            [only] => only.clone(),
            _ => {
                return Err(AppError::bad_request(
                    "redirect_uri is required: this application has more than one registered",
                ));
            }
        },
    };

    let state = params.state.as_deref();
    let fail = |error: &str, description: &str, state: Option<&str>| {
        error_redirect(&redirect_uri, error, description, state).map(Checked::Redirect)
    };

    if state.is_some_and(|s| s.len() > MAX_STATE_LEN) {
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
    let scopes = match protocol::parse_scopes(params.scope.as_deref()) {
        Ok(scopes) => scopes,
        Err(name) => return fail("invalid_scope", &format!("unknown scope: {name}"), state),
    };
    let code_challenge = match (
        params.code_challenge.as_deref(),
        params.code_challenge_method.as_deref(),
    ) {
        (None, None) => None,
        (Some(challenge), Some("S256")) if protocol::is_valid_challenge(challenge) => {
            Some(challenge.to_owned())
        }
        (Some(_), Some("S256")) => {
            return fail(
                "invalid_request",
                "code_challenge must be the base64url SHA-256 of the code verifier (43 characters)",
                state,
            );
        }
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

pub async fn check(
    ctx: &AppContext,
    subject: &Subject,
    params: &AuthorizeParams,
) -> Result<AuthorizeCheck, AppError> {
    let player = player(subject)?;
    let mut conn = ctx.db.conn().await?;
    let request = match validate(&mut conn, params).await? {
        Checked::Valid(request) => *request,
        Checked::Redirect(redirect_to) => return Ok(AuthorizeCheck::Redirect { redirect_to }),
    };
    let already_authorized = repo::has_grant(&mut conn, request.app.id, player).await?;
    let redirect_host = protocol::redirect_host(&request.redirect_uri).unwrap_or_default();
    Ok(AuthorizeCheck::Consent {
        scopes: request.scopes.iter().map(ToString::to_string).collect(),
        redirect_uri: request.redirect_uri,
        redirect_host,
        already_authorized,
        app: request.app.into(),
    })
}

pub async fn decide(
    ctx: &AppContext,
    subject: &Subject,
    input: AuthorizeDecisionInput,
) -> Result<AuthorizeDecision, AppError> {
    let player = player(subject)?;
    let mut tx = ctx.db.begin().await?;
    let request = match validate(&mut tx, &input.params).await? {
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

    if repo::find_identity(&mut tx, player).await?.is_none() {
        return Err(AppError::Unauthorized);
    }

    let scope = protocol::join_scopes(&request.scopes);
    let grant_id = repo::upsert_grant(&mut tx, request.app.id, player, &scope).await?;
    let code = protocol::mint(SecretKind::Code);
    repo::insert_code(
        &mut tx,
        &protocol::digest(&code),
        grant_id,
        &request.redirect_uri,
        &scope,
        request.code_challenge.as_deref(),
        CODE_TTL_SECS,
    )
    .await?;
    repo::delete_expired(&mut tx).await?;
    tx.commit().await?;

    let mut params = vec![("code", code.as_str())];
    if let Some(state) = request.state.as_deref() {
        params.push(("state", state));
    }
    let redirect_to =
        protocol::redirect_with(&request.redirect_uri, &params).ok_or_else(invalid_redirect)?;
    tracing::info!(app = %request.app.id, "oauth authorization granted");
    Ok(AuthorizeDecision { redirect_to })
}

pub async fn authorizations(
    ctx: &AppContext,
    subject: &Subject,
) -> Result<Vec<OAuthAuthorization>, AppError> {
    let player = player(subject)?;
    let rows = repo::list_authorizations(&mut *ctx.db.conn().await?, player).await?;
    Ok(rows
        .into_iter()
        .map(|row| OAuthAuthorization {
            scopes: protocol::scope_names(&row.scope),
            authorized_at: row.authorized_at,
            last_used_at: row.last_used_at,
            app: row.app.into(),
        })
        .collect())
}

pub async fn withdraw(ctx: &AppContext, subject: &Subject, app_id: Uuid) -> Result<(), AppError> {
    let player = player(subject)?;
    if !repo::delete_grant(&mut *ctx.db.conn().await?, app_id, player).await? {
        return Err(AppError::NotFound(None));
    }
    tracing::info!(app = %app_id, "oauth authorization revoked by the player");
    Ok(())
}
