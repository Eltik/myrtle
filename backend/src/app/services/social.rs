use crate::{
    app::{cache::keys::CacheKey, error::ApiError, state::AppState},
    core::hypergryph::{
        constants::{AuthSession, Server},
        fetch::auth_request,
    },
};

async fn load_session(state: &AppState, user_id: &str) -> Result<AuthSession, ApiError> {
    let session_json: Option<String> = state
        .cache
        .get(&CacheKey::GameSession { uid: user_id })
        .await;

    let session_json =
        session_json.ok_or(ApiError::BadRequest("no game session - login again".into()))?;

    serde_json::from_str(&session_json)
        .map_err(|_| ApiError::BadRequest("invalid game session".into()))
}

async fn save_session(state: &AppState, user_id: &str, session: &AuthSession) {
    if let Ok(json) = serde_json::to_string(session) {
        let () = state
            .cache
            .set(&CacheKey::GameSession { uid: user_id }, &json)
            .await;
    }
}

async fn social_sort_list(
    state: &AppState,
    user_id: &str,
    server: Server,
    list_type: u8,
    sort_keys: &[&str],
    param: serde_json::Value,
) -> Result<Vec<String>, ApiError> {
    let mut session = load_session(state, user_id).await?;

    let body = serde_json::json!({
        "type": list_type,
        "sortKeyList": sort_keys,
        "param": param,
    });

    let response = auth_request(
        &state.http_client,
        "social/getSortListInfo",
        Some(&body),
        &mut session,
        server,
    )
    .await?;

    let raw: serde_json::Value = response
        .json()
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    save_session(state, user_id, &session).await;

    let arr = raw
        .get("result")
        .and_then(|v| v.as_array())
        .ok_or(ApiError::BadRequest("malformed social response".into()))?;

    Ok(arr
        .iter()
        .filter_map(|e| e.get("uid").and_then(|u| u.as_str()).map(str::to_owned))
        .collect())
}

pub async fn get_raw_friend_ids(
    state: &AppState,
    user_id: &str,
    server: Server,
) -> Result<Vec<String>, ApiError> {
    social_sort_list(
        state,
        user_id,
        server,
        1,
        &["level", "infoShare"],
        serde_json::json!({}),
    )
    .await
}

pub async fn get_raw_friend_info(
    state: &AppState,
    user_id: &str,
    ids: &[String],
    server: Server,
) -> Result<serde_json::Value, ApiError> {
    let mut session = load_session(state, user_id).await?;

    let body = serde_json::json!({ "idList": ids });

    let response = auth_request(
        &state.http_client,
        "social/getFriendList",
        Some(&body),
        &mut session,
        server,
    )
    .await?;

    let raw: serde_json::Value = response
        .json()
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    save_session(state, user_id, &session).await;

    Ok(raw)
}

pub async fn get_friends(
    state: &AppState,
    user_id: &str,
    server: Server,
    limit: Option<usize>,
) -> Result<serde_json::Value, ApiError> {
    let mut ids = get_raw_friend_ids(state, user_id, server).await?;

    if let Some(n) = limit {
        ids.truncate(n);
    }

    if ids.is_empty() {
        return Ok(serde_json::json!({ "friends": [] }));
    }

    get_raw_friend_info(state, user_id, &ids, server).await
}

pub async fn search_players(
    state: &AppState,
    user_id: &str,
    query: &str,
    server: Server,
    limit: Option<usize>,
) -> Result<serde_json::Value, ApiError> {
    let (nick, num) = match query.split_once('#') {
        Some((n, t)) => (n, t),
        None => (query, ""),
    };

    let mut ids = search_raw_player_ids(state, user_id, nick, num, server).await?;
    if let Some(n) = limit {
        ids.truncate(n);
    }

    if ids.is_empty() {
        return Ok(serde_json::json!({ "friends": [] }));
    }

    get_raw_friend_info(state, user_id, &ids, server).await
}

pub async fn search_raw_player_ids(
    state: &AppState,
    user_id: &str,
    nickname: &str,
    nicknumber: &str,
    server: Server,
) -> Result<Vec<String>, ApiError> {
    social_sort_list(
        state,
        user_id,
        server,
        0,
        &["level"],
        serde_json::json!({ "nickName": nickname, "nickNumber": nicknumber }),
    )
    .await
}
