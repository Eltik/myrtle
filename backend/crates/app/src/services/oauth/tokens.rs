use myrtle_db::Conn;
use myrtle_db::repos::oauth as repo;
use myrtle_types::UserId;
use uuid::Uuid;

use super::dto::{OAuthTokenResponse, OAuthUserInfo};
use super::error::ProtocolError;
use super::protocol::{
    self, ACCESS_TTL_SECS, MAX_SESSIONS_PER_GRANT, REFRESH_TTL_SECS, Scope, SecretKind,
};
use crate::AppContext;

const DEFAULT_AVATAR_ID: &str = "char_002_amiya";

pub async fn authenticate(
    ctx: &AppContext,
    authorization: Option<&str>,
    body_id: Option<&str>,
    body_secret: Option<&str>,
) -> Result<Uuid, ProtocolError> {
    let (client_id, client_secret) =
        protocol::client_credentials(authorization, body_id, body_secret)?;
    let app_id: Uuid = client_id
        .parse()
        .map_err(|_| ProtocolError::invalid_client())?;
    let stored = repo::find_secret_hash(&mut *ctx.db.conn().await?, app_id)
        .await?
        .ok_or_else(ProtocolError::invalid_client)?;
    if !protocol::matches_digest(&client_secret, &stored) {
        return Err(ProtocolError::invalid_client());
    }
    Ok(app_id)
}

const fn token_response(
    access_token: String,
    refresh_token: String,
    scope: String,
) -> OAuthTokenResponse {
    OAuthTokenResponse {
        access_token,
        token_type: "Bearer",
        expires_in: ACCESS_TTL_SECS,
        refresh_token,
        scope,
    }
}

async fn issue(
    conn: &mut Conn,
    grant_id: Uuid,
    scope: String,
) -> Result<OAuthTokenResponse, ProtocolError> {
    let access_token = protocol::mint(SecretKind::AccessToken);
    let refresh_token = protocol::mint(SecretKind::RefreshToken);
    repo::insert_token(
        conn,
        grant_id,
        &protocol::digest(&access_token),
        &protocol::digest(&refresh_token),
        &scope,
        ACCESS_TTL_SECS,
        REFRESH_TTL_SECS,
        MAX_SESSIONS_PER_GRANT,
    )
    .await?;
    Ok(token_response(access_token, refresh_token, scope))
}

pub async fn exchange_code(
    ctx: &AppContext,
    app_id: Uuid,
    code: &str,
    redirect_uri: Option<&str>,
    code_verifier: Option<&str>,
) -> Result<OAuthTokenResponse, ProtocolError> {
    let code_hash = protocol::digest(code);
    let mut conn = ctx.db.conn().await?;

    let Some(spent) = repo::consume_code(&mut conn, &code_hash, app_id).await? else {
        if let Some(grant_id) = repo::find_spent_code_grant(&mut conn, &code_hash, app_id).await? {
            let revoked = repo::delete_grant_tokens(&mut conn, grant_id).await?;
            tracing::warn!(app = %app_id, revoked, "oauth authorization code replayed; sessions revoked");
        }
        return Err(ProtocolError::invalid_grant(
            "the authorization code is invalid, expired or already used",
        ));
    };

    if redirect_uri.is_some_and(|uri| uri != spent.redirect_uri) {
        return Err(ProtocolError::invalid_grant(
            "redirect_uri does not match the one the code was issued for",
        ));
    }
    match (spent.code_challenge.as_deref(), code_verifier) {
        (Some(challenge), Some(verifier)) if protocol::verify_pkce(verifier, challenge) => {}
        (Some(_), Some(_)) => {
            return Err(ProtocolError::invalid_grant(
                "code_verifier does not match the code_challenge",
            ));
        }
        (Some(_), None) => {
            return Err(ProtocolError::invalid_request(
                "code_verifier is required: the authorization request carried a code_challenge",
            ));
        }
        (None, Some(_)) => {
            return Err(ProtocolError::invalid_grant(
                "code_verifier was sent, but the authorization request carried no code_challenge",
            ));
        }
        (None, None) => {}
    }

    issue(&mut conn, spent.grant_id, spent.scope).await
}

pub async fn refresh(
    ctx: &AppContext,
    app_id: Uuid,
    refresh_token: &str,
    scope: Option<&str>,
) -> Result<OAuthTokenResponse, ProtocolError> {
    if let Err(name) = protocol::parse_scopes(scope) {
        return Err(ProtocolError::invalid_scope(format!(
            "unknown scope: {name}"
        )));
    }

    let access_token = protocol::mint(SecretKind::AccessToken);
    let new_refresh_token = protocol::mint(SecretKind::RefreshToken);
    let scope = repo::rotate_token(
        &mut *ctx.db.conn().await?,
        &protocol::digest(refresh_token),
        app_id,
        &protocol::digest(&access_token),
        &protocol::digest(&new_refresh_token),
        ACCESS_TTL_SECS,
        REFRESH_TTL_SECS,
    )
    .await?
    .ok_or_else(|| {
        ProtocolError::invalid_grant(
            "the refresh token is invalid, expired, revoked or already used",
        )
    })?;
    Ok(token_response(access_token, new_refresh_token, scope))
}

pub async fn revoke(ctx: &AppContext, app_id: Uuid, token: &str) -> Result<(), ProtocolError> {
    repo::revoke_token(&mut *ctx.db.conn().await?, &protocol::digest(token), app_id).await?;
    Ok(())
}

pub async fn userinfo(
    ctx: &AppContext,
    access_token: &str,
    api_base: &str,
) -> Result<OAuthUserInfo, ProtocolError> {
    let mut conn = ctx.db.conn().await?;
    let owner = repo::find_access_token(&mut conn, &protocol::digest(access_token))
        .await?
        .ok_or_else(ProtocolError::invalid_token)?;
    if !protocol::stored_scopes(&owner.scope).contains(&Scope::Identify) {
        return Err(ProtocolError::insufficient_scope(Scope::Identify));
    }
    let user = repo::find_identity(&mut conn, UserId(owner.user_id))
        .await?
        .ok_or_else(ProtocolError::invalid_token)?;

    let avatar = user.avatar_id.as_deref().unwrap_or(DEFAULT_AVATAR_ID);
    let avatar_url = format!("{api_base}/api/avatar/{}", protocol::avatar_stem(avatar));
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
