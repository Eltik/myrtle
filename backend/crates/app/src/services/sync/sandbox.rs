use std::time::Duration;

use myrtle_db::repos::{progress, users};
use myrtle_hypergryph::fetch::{auth_request, read_body};
use myrtle_hypergryph::{AuthSession, SessionKey, UpstreamError};
use myrtle_jobs::{JobKind, JobSpec};
use myrtle_types::{GameUid, Server, UserId};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::invalidate_user;
use crate::services::accounts::game_session;
use crate::services::grading;
use crate::{AppContext, AppError};

const CHANGE_TOPIC: &str = "sandboxPerm/changeTopic";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingTopic {
    pub template: String,
    pub topic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxBackfill {
    pub user_id: uuid::Uuid,
    pub loaded: String,
    pub topics: Vec<MissingTopic>,
}

fn held(stored: Option<&Value>, m: &MissingTopic) -> bool {
    stored
        .and_then(|s| s.pointer("/template"))
        .and_then(|t| t.get(&m.template))
        .and_then(|t| t.get(&m.topic))
        .is_some_and(Value::is_object)
}

#[must_use]
pub fn missing_topics(sync: &Value, stored: Option<&Value>) -> Vec<MissingTopic> {
    let Some(perm) = sync.pointer("/user/sandboxPerm") else {
        return Vec::new();
    };
    let loaded = perm
        .get("topic")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let flags = sync.pointer("/user/status/flags");
    let Some(summary) = perm.get("summary").and_then(Value::as_object) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (template, topics) in summary {
        for (topic, info) in topics.as_object().into_iter().flatten() {
            if topic == loaded {
                continue;
            }
            let entered = flags
                .and_then(|f| f.get(format!("Obt/SandboxPerm/{topic}/{topic}_entry")))
                .and_then(Value::as_i64)
                == Some(1);
            let progressed = info
                .as_object()
                .is_some_and(|o| o.values().any(|v| v.as_f64().is_some_and(|n| n > 0.0)));
            let m = MissingTopic {
                template: template.clone(),
                topic: topic.clone(),
            };
            if entered && progressed && !held(stored, &m) {
                out.push(m);
            }
        }
    }
    out
}

#[must_use]
pub fn blob_from_change<'a>(response: &'a Value, missing: &MissingTopic) -> Option<&'a Value> {
    if response.get("result").and_then(Value::as_i64) != Some(0) {
        return None;
    }
    response
        .pointer("/playerDataDelta/modified/sandboxPerm/template")?
        .get(&missing.template)?
        .get(&missing.topic)
        .filter(|b| b.is_object())
}

#[must_use]
pub fn plan(user: UserId, sync: &Value) -> Option<SandboxBackfill> {
    let topics = missing_topics(sync, None);
    let loaded = sync
        .pointer("/user/sandboxPerm/topic")
        .and_then(Value::as_str)
        .filter(|_| !topics.is_empty())?;
    Some(SandboxBackfill {
        user_id: user.0,
        loaded: loaded.to_owned(),
        topics,
    })
}

pub async fn enqueue(ctx: &AppContext, job: &SandboxBackfill) -> Result<(), AppError> {
    if !ctx.config.switches.sandbox_topic_backfill.enabled {
        return Ok(());
    }
    ctx.queue
        .enqueue(JobSpec {
            payload: serde_json::to_value(job).map_err(AppError::internal)?,
            dedupe_key: Some(job.user_id.to_string()),
            ..JobSpec::new(JobKind::SandboxBackfill)
        })
        .await?;
    Ok(())
}

pub async fn backfill(ctx: &AppContext, job: &SandboxBackfill) -> Result<usize, AppError> {
    let user = UserId(job.user_id);
    let mut conn = ctx.db.conn().await?;
    let account = users::account(&mut conn, user)
        .await?
        .ok_or(AppError::NotFound(None))?;
    let stored = progress::sandbox(&mut conn, user).await?;
    drop(conn);
    let missing: Vec<&MissingTopic> = job
        .topics
        .iter()
        .filter(|m| !held(stored.as_ref(), m))
        .collect();
    if missing.is_empty() {
        return Ok(0);
    }
    let server = Server::from_index(account.server_id)
        .ok_or_else(|| AppError::internal(format!("unknown server id {}", account.server_id)))?;
    let key = SessionKey::Player {
        server,
        uid: GameUid(account.uid.clone().into()),
    };
    let mut session = game_session::ensure_fresh(ctx, &account.uid, server).await?;
    let mut fetched = 0;
    for m in missing {
        let response = match change_topic(ctx, &key, &mut session, &m.topic).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(uid = %account.uid, topic = %m.topic, error = %e, "RA topic backfill: switch refused");
                continue;
            }
        };
        if let Some(blob) = blob_from_change(&response, m) {
            let mut conn = ctx.db.conn().await?;
            progress::store_sandbox_topic(&mut conn, user, &m.template, &m.topic, blob).await?;
            fetched += 1;
        } else {
            tracing::warn!(uid = %account.uid, topic = %m.topic, "RA topic backfill: the switch answered without the topic");
        }
        switch_back(ctx, &key, &mut session, &job.loaded).await;
    }
    if fetched > 0 {
        topics_stored(ctx, user, &account.uid).await?;
    }
    Ok(fetched)
}

pub async fn topics_stored(ctx: &AppContext, user: UserId, uid: &str) -> Result<(), AppError> {
    users::touch(&mut *ctx.db.conn().await?, user).await?;
    grading::enqueue_sync_regrade(ctx, user).await?;
    invalidate_user(ctx, uid).await;
    Ok(())
}

async fn change_topic(
    ctx: &AppContext,
    key: &SessionKey,
    session: &mut AuthSession,
    topic: &str,
) -> Result<Value, UpstreamError> {
    let body = json!({ "topicId": topic });
    let sent = auth_request(
        &ctx.upstream,
        CHANGE_TOPIC,
        Some(&body),
        session,
        key.server(),
    )
    .await;
    if let Err(e) = ctx.sessions.save(key, session).await {
        tracing::warn!(error = e.kind(), "failed to save the game session");
    }
    let text = read_body(sent?, CHANGE_TOPIC).await?;
    serde_json::from_str(&text).map_err(|e| UpstreamError::Parse {
        context: CHANGE_TOPIC.to_owned(),
        message: e.to_string(),
    })
}

async fn switch_back(ctx: &AppContext, key: &SessionKey, session: &mut AuthSession, loaded: &str) {
    for attempt in 1..=2 {
        tokio::time::sleep(Duration::from_secs(
            ctx.config.jobs.sandbox_backfill.switch_wait_secs,
        ))
        .await;
        match change_topic(ctx, key, session, loaded).await {
            Ok(r) if r.get("result").and_then(Value::as_i64) == Some(0) => return,
            Ok(r) => {
                tracing::warn!(attempt, result = ?r.get("result"), "RA topic backfill: switch back answered non-zero");
            }
            Err(e) => tracing::warn!(attempt, error = %e, "RA topic backfill: switch back failed"),
        }
    }
    tracing::error!(
        loaded,
        "RA topic backfill could not switch back; the account is left on the fetched topic"
    );
}
