use uuid::Uuid;

use crate::{
    app::{cache::keys::CacheKey, error::ApiError, state::AppState},
    core::hypergryph::{
        constants::{AuthSession, Server},
        session,
    },
    database::queries::{game_credentials, users},
};

/// Cached game session; fails if absent or unparseable.
///
/// Cache-only: for paths that run after [`ensure_fresh`] in the same request.
/// Cold starts call [`ensure_fresh`], which falls back to the durable store.
pub async fn load(state: &AppState, user_id: &str) -> Result<AuthSession, ApiError> {
    let json: Option<String> = state
        .cache
        .get(&CacheKey::GameSession { uid: user_id })
        .await;

    let json = json.ok_or(ApiError::GameLoginRequired)?;

    serde_json::from_str(&json).map_err(|_| ApiError::BadRequest("invalid game session".into()))
}

/// Caches a game session, renewing its TTL.
///
/// Cache-only on purpose: the durable `yostar_uid` / `yostar_token` pair is fixed
/// at login (`refresh_secret` never rewrites it), so the DB has nothing to learn.
/// [`crate::app::services::auth`] writes the durable pair once per login.
pub async fn save(state: &AppState, user_id: &str, session: &AuthSession) {
    if let Ok(json) = serde_json::to_string(session) {
        let () = state
            .cache
            .set(&CacheKey::GameSession { uid: user_id }, &json)
            .await;
    }
}

/// Rebuild a session from the durable credentials in `user_game_credentials`.
///
/// The result carries only the durable pair; [`session::refresh_secret`] fills
/// in the live `uid`, `token` and `secret`.
async fn restore(state: &AppState, uid: &str, server: Server) -> Result<AuthSession, ApiError> {
    let user = users::find_raw_by_uid(&state.db, uid, server.index() as i16)
        .await?
        .ok_or(ApiError::GameLoginRequired)?;

    let credential = game_credentials::load(&state.db, &state.config.game_credential_key, user.id)
        .await?
        .ok_or(ApiError::GameLoginRequired)?;

    Ok(AuthSession {
        uid: uid.into(),
        yostar_uid: credential.yostar_uid.into(),
        yostar_token: credential.yostar_token.into(),
        ..Default::default()
    })
}

/// Live session: cached or rebuilt from the durable store, with a freshly minted
/// secret and a reset sequence counter, persisted.
///
/// The cache is an optimisation, not the source of truth. A miss, eviction,
/// restart or unparseable entry all rebuild from `user_game_credentials`, so
/// [`ApiError::GameLoginRequired`] means we genuinely hold nothing.
pub async fn ensure_fresh(
    state: &AppState,
    user_id: &str,
    server: Server,
) -> Result<AuthSession, ApiError> {
    let mut session = match load(state, user_id).await {
        Ok(session) if !session.yostar_token.is_empty() => session,
        _ => restore(state, user_id, server).await?,
    };

    session::refresh_secret(&state.http_client, &mut session, server).await?;
    save(state, user_id, &session).await;
    Ok(session)
}

/// Forgets the cached session and the durable credentials behind it (the
/// user-facing "disconnect"). A fresh email code is needed afterwards; synced data
/// is untouched. Before this, account deletion was the only self-serve revocation.
pub async fn disconnect(state: &AppState, uid: &str, user_id: Uuid) -> Result<bool, ApiError> {
    state.cache.invalidate(&CacheKey::GameSession { uid }).await;
    state
        .cache
        .invalidate(&CacheKey::PortalSession { uid })
        .await;

    Ok(game_credentials::delete(&state.db, user_id).await?)
}
