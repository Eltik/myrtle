use crate::{
    app::{error::ApiError, services::game_session, state::AppState},
    core::hypergryph::{constants::Server, fetch::auth_request},
};

async fn social_sort_list(
    state: &AppState,
    user_id: &str,
    server: Server,
    list_type: u8,
    sort_keys: &[&str],
    param: serde_json::Value,
) -> Result<Vec<String>, ApiError> {
    let mut session = game_session::load(state, user_id).await?;

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

    game_session::save(state, user_id, &session).await;

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
    let mut session = game_session::load(state, user_id).await?;

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

    game_session::save(state, user_id, &session).await;

    Ok(raw)
}

pub async fn get_friends(
    state: &AppState,
    user_id: &str,
    server: Server,
    limit: Option<usize>,
) -> Result<serde_json::Value, ApiError> {
    game_session::ensure_fresh(state, user_id, server).await?;

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
    game_session::ensure_fresh(state, user_id, server).await?;

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
