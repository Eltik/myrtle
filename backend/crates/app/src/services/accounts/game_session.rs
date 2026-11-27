use myrtle_db::repos::credentials;
use myrtle_hypergryph::{AuthSession, SessionKey, UpstreamError, session};
use myrtle_types::{GameUid, Server};
use serde::Serialize;

use super::sessions::{portal_session_key, session_key};
use crate::{AppContext, AppError, Subject};

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DisconnectResult {
    pub status: String,
    pub removed: bool,
}

pub async fn load(ctx: &AppContext, uid: &str) -> Result<AuthSession, AppError> {
    let json = ctx
        .cache
        .get(&session_key(uid))
        .await
        .ok_or(AppError::GameLoginRequired)?;
    serde_json::from_str(&json).map_err(|_| AppError::bad_request("invalid game session"))
}

pub async fn ensure_fresh(
    ctx: &AppContext,
    uid: &str,
    server: Server,
) -> Result<AuthSession, AppError> {
    let key = SessionKey::Player {
        server,
        uid: GameUid(uid.into()),
    };
    session::ensure_fresh(&ctx.upstream, ctx.sessions.as_ref(), &key)
        .await
        .map_err(|e| match e {
            UpstreamError::NotLoggedIn => AppError::GameLoginRequired,
            other => AppError::Upstream(other),
        })
}

pub async fn disconnect(ctx: &AppContext, subject: &Subject) -> Result<DisconnectResult, AppError> {
    let (Some(user), Some(uid)) = (subject.user_id(), subject.uid()) else {
        return Err(AppError::Unauthorized);
    };
    ctx.cache.invalidate(&session_key(uid)).await;
    ctx.cache.invalidate(&portal_session_key(uid)).await;
    let removed = credentials::delete(&mut *ctx.db.conn().await?, user).await?;
    Ok(DisconnectResult {
        status: "ok".to_owned(),
        removed,
    })
}
