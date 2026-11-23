use myrtle_hypergryph::fetch::auth_request;
use myrtle_hypergryph::session::ensure_fresh;
use myrtle_hypergryph::{AuthSession, SessionKey, UpstreamError};
use myrtle_types::GameUid;
use serde_json::{Value, json};

use crate::policy::Subject;
use crate::{AppContext, AppError};

const SORT_LIST: &str = "social/getSortListInfo";
const FRIEND_LIST: &str = "social/getFriendList";

pub async fn friends(
    ctx: &AppContext,
    subject: &Subject,
    limit: Option<usize>,
) -> Result<Value, AppError> {
    lookup(ctx, subject, &friends_request(), limit).await
}

pub async fn search_players(
    ctx: &AppContext,
    subject: &Subject,
    query: &str,
    limit: Option<usize>,
) -> Result<Value, AppError> {
    lookup(ctx, subject, &search_request(query), limit).await
}

#[must_use]
pub fn friends_request() -> Value {
    sort_list(1, &["level", "infoShare"], json!({}))
}

#[must_use]
pub fn search_request(query: &str) -> Value {
    let (nick, number) = query.split_once('#').unwrap_or((query, ""));
    sort_list(
        0,
        &["level"],
        json!({ "nickName": nick, "nickNumber": number }),
    )
}

#[must_use]
pub fn friend_info_request(ids: &[String]) -> Value {
    json!({ "idList": ids })
}

pub fn listed_uids(raw: &Value, limit: Option<usize>) -> Result<Vec<String>, AppError> {
    let entries = raw
        .get("result")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::bad_request("malformed social response"))?;
    let uids = entries
        .iter()
        .filter_map(|e| e.get("uid").and_then(Value::as_str).map(str::to_owned));
    Ok(match limit {
        Some(n) => uids.take(n).collect(),
        None => uids.collect(),
    })
}

#[must_use]
pub fn no_friends() -> Value {
    json!({ "friends": [] })
}

#[must_use]
pub fn upstream_error(e: UpstreamError) -> AppError {
    match e {
        UpstreamError::NotLoggedIn => AppError::GameLoginRequired,
        other => AppError::Upstream(other),
    }
}

fn sort_list(list_type: u8, sort_keys: &[&str], param: Value) -> Value {
    json!({ "type": list_type, "sortKeyList": sort_keys, "param": param })
}

async fn lookup(
    ctx: &AppContext,
    subject: &Subject,
    sort_list: &Value,
    limit: Option<usize>,
) -> Result<Value, AppError> {
    let key = player_key(subject)?;
    let mut session = ensure_fresh(&ctx.upstream, ctx.sessions.as_ref(), &key)
        .await
        .map_err(upstream_error)?;
    let listed = request(ctx, &key, &mut session, SORT_LIST, sort_list).await?;
    let ids = listed_uids(&listed, limit)?;
    if ids.is_empty() {
        return Ok(no_friends());
    }
    request(
        ctx,
        &key,
        &mut session,
        FRIEND_LIST,
        &friend_info_request(&ids),
    )
    .await
}

fn player_key(subject: &Subject) -> Result<SessionKey, AppError> {
    let uid = subject.uid().ok_or(AppError::Unauthorized)?;
    let server = subject.game_server()?;
    Ok(SessionKey::Player {
        server,
        uid: GameUid(uid.into()),
    })
}

async fn request(
    ctx: &AppContext,
    key: &SessionKey,
    session: &mut AuthSession,
    endpoint: &str,
    body: &Value,
) -> Result<Value, AppError> {
    let response = auth_request(&ctx.upstream, endpoint, Some(body), session, key.server())
        .await
        .map_err(upstream_error)?;
    let raw = response.json::<Value>().await.map_err(AppError::internal)?;
    if let Err(e) = ctx.sessions.save(key, session).await {
        tracing::warn!(error = e.kind(), "failed to save the game session");
    }
    Ok(raw)
}
