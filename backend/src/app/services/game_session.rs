use uuid::Uuid;

use crate::{
    app::{cache::keys::CacheKey, error::ApiError, state::AppState},
    core::hypergryph::{
        constants::{AuthSession, Server},
        session,
    },
    database::queries::{game_credentials, users},
};

/// Shown whenever no session can be established from either the cache or the
/// durable store. Reaching this genuinely does require a new email code.
const NEEDS_LOGIN: &str = "no game session - login again";

/// Load a user's cached game session. Fails if absent or unparseable.
///
/// Cache-only by design: this serves paths that run after [`ensure_fresh`] has
/// already established a live session in the same request. Anything starting
/// cold should call [`ensure_fresh`], which falls back to the durable store.
pub async fn load(state: &AppState, user_id: &str) -> Result<AuthSession, ApiError> {
    let json: Option<String> = state
        .cache
        .get(&CacheKey::GameSession { uid: user_id })
        .await;

    let json = json.ok_or(ApiError::BadRequest(NEEDS_LOGIN.into()))?;

    serde_json::from_str(&json).map_err(|_| ApiError::BadRequest("invalid game session".into()))
}

/// Persist a game session, renewing its TTL.
///
/// Cache-only, and deliberately so. The durable `yostar_uid` / `yostar_token`
/// pair inside the session is fixed at login and never mutates afterwards —
/// `refresh_secret` rewrites the uid, token, secret and seqnum but not those
/// two — so there is nothing here for the database to learn. Writing the
/// durable pair is [`crate::app::services::auth`]'s job, once per login.
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
        .ok_or_else(|| ApiError::BadRequest(NEEDS_LOGIN.into()))?;

    let credential = game_credentials::load(&state.db, &state.config.game_credential_key, user.id)
        .await?
        .ok_or_else(|| ApiError::BadRequest(NEEDS_LOGIN.into()))?;

    Ok(AuthSession {
        uid: uid.into(),
        yostar_uid: credential.yostar_uid.into(),
        yostar_token: credential.yostar_token.into(),
        ..Default::default()
    })
}

/// Establish a live session: take the cached one or rebuild it from the durable
/// store, re-mint a fresh secret from its durable token, persist it, and return
/// it. Downstream calls then run against a valid secret with a reset sequence
/// counter.
///
/// The cache is an optimisation here, not the source of truth. A miss, an
/// eviction, a backend restart, or a cached value that no longer parses all
/// land on the same path — rebuild from `user_game_credentials` — so a user
/// only sees `NEEDS_LOGIN` when we genuinely hold nothing for them.
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

/// Forget a user's game session entirely: the live copy in the cache and the
/// durable credentials behind it.
///
/// This is the user-facing "disconnect" action. Afterwards we cannot reach the
/// account again without a fresh email code, which is the point — it is the
/// self-serve revocation that account deletion used to be the only route to.
/// Their already-synced data is untouched.
pub async fn disconnect(state: &AppState, uid: &str, user_id: Uuid) -> Result<bool, ApiError> {
    state.cache.invalidate(&CacheKey::GameSession { uid }).await;
    state
        .cache
        .invalidate(&CacheKey::PortalSession { uid })
        .await;

    Ok(game_credentials::delete(&state.db, user_id).await?)
}
