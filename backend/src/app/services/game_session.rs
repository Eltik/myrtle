use crate::{
    app::{cache::keys::CacheKey, error::ApiError, state::AppState},
    core::hypergryph::{
        constants::{AuthSession, Server},
        session,
    },
};

/// Load a user's cached game session. Fails if absent or unparseable.
pub async fn load(state: &AppState, user_id: &str) -> Result<AuthSession, ApiError> {
    let json: Option<String> = state
        .cache
        .get(&CacheKey::GameSession { uid: user_id })
        .await;

    let json = json.ok_or(ApiError::BadRequest("no game session - login again".into()))?;

    serde_json::from_str(&json).map_err(|_| ApiError::BadRequest("invalid game session".into()))
}

/// Persist a game session, renewing its TTL.
pub async fn save(state: &AppState, user_id: &str, session: &AuthSession) {
    if let Ok(json) = serde_json::to_string(session) {
        let () = state
            .cache
            .set(&CacheKey::GameSession { uid: user_id }, &json)
            .await;
    }
}

/// Load the session, re-mint a fresh secret from its durable token, persist it,
/// and return it. Establishes a fresh active session for the account, so
/// downstream calls run against a valid secret with a reset sequence counter.
pub async fn ensure_fresh(
    state: &AppState,
    user_id: &str,
    server: Server,
) -> Result<AuthSession, ApiError> {
    let mut session = load(state, user_id).await?;
    session::refresh_secret(&state.http_client, &mut session, server).await?;
    save(state, user_id, &session).await;
    Ok(session)
}
