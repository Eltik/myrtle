use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::auth::jwt::create_token;
use crate::core::hypergryph::constants::{AuthSession, Server};
use crate::core::hypergryph::session;
use crate::core::hypergryph::yostar;
use crate::database::queries::game_credentials;
use crate::database::queries::users;
use crate::database::queries::users::create_user;
use crate::database::queries::users::find_raw_by_uid;
use serde::Serialize;
use uuid::Uuid;

pub fn parse_server(s: &str) -> Result<Server, ApiError> {
    match s {
        "en" => Ok(Server::EN),
        "jp" => Ok(Server::JP),
        "kr" => Ok(Server::KR),
        "cn" => Ok(Server::CN),
        "bili" => Ok(Server::Bilibili),
        "tw" => Ok(Server::TW),
        _ => Err(ApiError::BadRequest(format!("unknown server: {s}"))),
    }
}

pub async fn send_code(state: &AppState, email: &str, server: Server) -> Result<(), ApiError> {
    if let Err(e) = yostar::send_code(&state.http_client, email, server).await {
        tracing::warn!(
            email = %email,
            server = server.as_str(),
            error = ?e,
            "yostar send_code failed"
        );
        return Err(e.into());
    }
    Ok(())
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub uid: String,
    pub server: String,
}

/// Persist the durable half of a fresh login, so re-syncing survives the cache.
///
/// Every login route ends here. The cached session expires in an hour and dies
/// on any backend restart; the site token lasts seven days. Without this the
/// gap between the two is a forced re-login, which is what users hit when they
/// pressed "Re-sync" the next day.
///
/// Best-effort on purpose: a database that will not take the credential costs
/// the user a re-login later, and failing the login in front of them now would
/// be the worse trade. It is logged either way.
async fn persist_credentials(state: &AppState, user_id: Uuid, uid: &str, session: &AuthSession) {
    if let Err(e) =
        game_credentials::store(&state.db, &state.config.game_credential_key, user_id, session)
            .await
    {
        tracing::warn!(
            uid = %uid,
            error = ?e,
            "failed to persist durable game credentials; resync will need a re-login"
        );
    }
}

pub async fn login(
    state: &AppState,
    email: &str,
    code: &str,
    server: Server,
) -> Result<LoginResponse, ApiError> {
    let result = match session::login(&state.http_client, email, code, server).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                email = %email,
                server = server.as_str(),
                error = ?e,
                "yostar login failed"
            );
            return Err(e.into());
        }
    };
    let session_json =
        serde_json::to_string(&result.session).map_err(|e| ApiError::Internal(e.into()))?;
    let uid = &*result.session.uid;

    state
        .cache
        .set(&CacheKey::GameSession { uid }, &session_json)
        .await;

    if let Some(portal) = &result.portal_session {
        let portal_json =
            serde_json::to_string(portal).map_err(|e| ApiError::Internal(e.into()))?;
        state
            .cache
            .set(&CacheKey::PortalSession { uid }, &portal_json)
            .await;
    }

    let user = match find_raw_by_uid(&state.db, uid, server.index() as i16).await? {
        Some(u) => u,
        None => create_user(&state.db, uid, server.index() as i16).await?,
    };

    persist_credentials(state, user.id, uid, &result.session).await;

    let token = create_token(
        &state.config.jwt_secret,
        &user.id.to_string(),
        uid,
        server.as_str(),
        &user.role,
        7,
    )?;

    Ok(LoginResponse {
        token,
        uid: uid.to_owned(),
        server: server.as_str().to_owned(),
    })
}

pub async fn login_bilibili(
    state: &AppState,
    username: &str,
    password: &str,
) -> Result<LoginResponse, ApiError> {
    let server = Server::Bilibili;

    let auth_session = match session::login_bilibili(&state.http_client, username, password).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(username = %username, error = ?e, "bilibili login failed");
            return Err(e.into());
        }
    };

    let session_json =
        serde_json::to_string(&auth_session).map_err(|e| ApiError::Internal(e.into()))?;
    let uid = &*auth_session.uid;

    state
        .cache
        .set(&CacheKey::GameSession { uid }, &session_json)
        .await;

    let user = match find_raw_by_uid(&state.db, uid, server.index() as i16).await? {
        Some(u) => u,
        None => create_user(&state.db, uid, server.index() as i16).await?,
    };

    persist_credentials(state, user.id, uid, &auth_session).await;

    let token = create_token(
        &state.config.jwt_secret,
        &user.id.to_string(),
        uid,
        server.as_str(),
        &user.role,
        7,
    )?;

    Ok(LoginResponse {
        token,
        uid: uid.to_owned(),
        server: server.as_str().to_owned(),
    })
}

pub async fn send_bilibili_sms(state: &AppState, phone: &str) -> Result<(), ApiError> {
    if let Err(e) = session::send_bilibili_sms(&state.http_client, phone).await {
        tracing::warn!(phone = %phone, error = ?e, "bilibili send_sms_code failed");
        return Err(e.into());
    }
    Ok(())
}

pub async fn login_bilibili_sms(
    state: &AppState,
    phone: &str,
    sms_code: &str,
) -> Result<LoginResponse, ApiError> {
    let server = Server::Bilibili;

    let auth_session = match session::login_bilibili_sms(&state.http_client, phone, sms_code).await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(phone = %phone, error = ?e, "bilibili sms login failed");
            return Err(e.into());
        }
    };

    let session_json =
        serde_json::to_string(&auth_session).map_err(|e| ApiError::Internal(e.into()))?;
    let uid = &*auth_session.uid;

    state
        .cache
        .set(&CacheKey::GameSession { uid }, &session_json)
        .await;

    let user = match find_raw_by_uid(&state.db, uid, server.index() as i16).await? {
        Some(u) => u,
        None => create_user(&state.db, uid, server.index() as i16).await?,
    };

    persist_credentials(state, user.id, uid, &auth_session).await;

    let token = create_token(
        &state.config.jwt_secret,
        &user.id.to_string(),
        uid,
        server.as_str(),
        &user.role,
        7,
    )?;

    Ok(LoginResponse {
        token,
        uid: uid.to_owned(),
        server: server.as_str().to_owned(),
    })
}

pub async fn send_code_cn(state: &AppState, phone: &str) -> Result<(), ApiError> {
    use crate::core::hypergryph::passport;

    if let Err(e) = passport::send_phone_code(&state.http_client, phone).await {
        tracing::warn!(phone = %phone, error = ?e, "hypergryph passport send_phone_code failed");
        return Err(e.into());
    }
    Ok(())
}

/// Experimental CN login via the Hypergryph passport. See
/// `core::hypergryph::passport` module docs: the appCode this depends on is
/// an unverified placeholder, so this is expected to fail until the real
/// game-client appCode is known.
pub async fn login_cn(
    state: &AppState,
    phone: &str,
    password: Option<&str>,
    sms_code: Option<&str>,
) -> Result<LoginResponse, ApiError> {
    use crate::core::hypergryph::passport::PassportCredential;

    let credential = match (password, sms_code) {
        (Some(password), _) => PassportCredential::Password { phone, password },
        (None, Some(code)) => PassportCredential::SmsCode { phone, code },
        (None, None) => {
            return Err(ApiError::BadRequest(
                "provide either password or code".into(),
            ));
        }
    };

    let server = Server::CN;

    let auth_session = match session::login_cn(&state.http_client, credential).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(phone = %phone, error = ?e, "hypergryph CN login failed");
            return Err(e.into());
        }
    };

    let session_json =
        serde_json::to_string(&auth_session).map_err(|e| ApiError::Internal(e.into()))?;
    let uid = &*auth_session.uid;

    state
        .cache
        .set(&CacheKey::GameSession { uid }, &session_json)
        .await;

    let user = match find_raw_by_uid(&state.db, uid, server.index() as i16).await? {
        Some(u) => u,
        None => create_user(&state.db, uid, server.index() as i16).await?,
    };

    persist_credentials(state, user.id, uid, &auth_session).await;

    let token = create_token(
        &state.config.jwt_secret,
        &user.id.to_string(),
        uid,
        server.as_str(),
        &user.role,
        7,
    )?;

    Ok(LoginResponse {
        token,
        uid: uid.to_owned(),
        server: server.as_str().to_owned(),
    })
}

pub async fn update_settings(
    state: &AppState,
    user_id: Uuid,
    public_profile: bool,
    store_gacha: bool,
    share_stats: bool,
) -> Result<(), ApiError> {
    users::update_settings(&state.db, user_id, public_profile, store_gacha, share_stats).await?;
    Ok(())
}
