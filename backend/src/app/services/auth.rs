use crate::app::cache::keys::CacheKey;
use crate::app::cache::store;
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::auth::jwt;
use crate::core::hypergryph::constants::Server;
use crate::core::hypergryph::session;
use crate::core::hypergryph::yostar;
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
    yostar::send_code(&state.http_client, email, server).await?;
    Ok(())
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub uid: String,
    pub server: String,
}

pub async fn login(
    state: &AppState,
    email: &str,
    code: &str,
    server: Server,
) -> Result<LoginResponse, ApiError> {
    let result = session::login(&state.http_client, email, code, server).await?;
    let session_json =
        serde_json::to_string(&result.session).map_err(|e| ApiError::Internal(e.into()))?;
    let uid = &*result.session.uid;

    store::set(
        &mut state.redis.clone(),
        &CacheKey::GameSession { uid },
        &session_json,
    )
    .await;

    if let Some(portal) = &result.portal_session {
        let portal_json =
            serde_json::to_string(portal).map_err(|e| ApiError::Internal(e.into()))?;
        store::set(
            &mut state.redis.clone(),
            &CacheKey::PortalSession { uid },
            &portal_json,
        )
        .await;
    }

    let user = match find_raw_by_uid(&state.db, uid, server.index() as i16).await? {
        Some(u) => u,
        None => create_user(&state.db, uid, server.index() as i16).await?,
    };

    let token = jwt::create_token(
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
    crate::database::queries::users::update_settings(
        &state.db,
        user_id,
        public_profile,
        store_gacha,
        share_stats,
    )
    .await?;
    Ok(())
}
