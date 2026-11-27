pub mod payload;
pub mod roster;
pub mod sandbox;

use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use myrtle_db::repos::{roster as roster_repo, users};
use myrtle_hypergryph::login::yostar::sync_data_raw;
use myrtle_hypergryph::{SessionKey, UpstreamError};
use myrtle_jobs::{JobKind, JobSpec};
use myrtle_types::{GameUid, Server, UserId};
use serde_json::{Value, json};

use crate::services::accounts::game_session;
use crate::services::catalog::maintenance;
use crate::services::grading;
use crate::services::leaderboard::lookback;
use crate::services::story::progress;
use crate::{AppContext, AppError, Subject};

const OWNERSHIP_DEDUPE: &str = "sync";
const OWNERSHIP_DELAY: TimeDelta = TimeDelta::minutes(1);

#[must_use]
pub fn user_key(uid: &str) -> String {
    format!("user:{uid}")
}

#[must_use]
pub fn improvements_prefix(uid: &str) -> String {
    format!("improvements:{uid}:")
}

async fn invalidate_user(ctx: &AppContext, uid: &str) {
    ctx.cache.invalidate(&user_key(uid)).await;
    ctx.cache.invalidate_prefix(&improvements_prefix(uid)).await;
}

pub async fn refresh(ctx: &AppContext, subject: &Subject) -> Result<Value, AppError> {
    let uid = subject.uid().ok_or(AppError::Unauthorized)?;
    let server = subject.game_server()?;
    if let Some(window) = maintenance::blocking_window(ctx, server).await {
        return Err(AppError::GameMaintenance {
            ends_at: window.ends_at,
        });
    }

    let mut session = game_session::ensure_fresh(ctx, uid, server).await?;
    let text = match sync_data_raw(&ctx.upstream, &mut session, server).await {
        Ok(text) => text,
        Err(e @ UpstreamError::Maintenance(_)) => {
            tracing::info!(uid, server = server.as_str(), error = %e, "account/syncData refused for maintenance");
            return Err(AppError::GameMaintenance { ends_at: None });
        }
        Err(e) => {
            tracing::warn!(uid, server = server.as_str(), error = %e, "account/syncData failed");
            return Err(e.into());
        }
    };
    let key = SessionKey::Player {
        server,
        uid: GameUid(uid.into()),
    };
    if let Err(e) = ctx.sessions.save(&key, &session).await {
        tracing::warn!(error = e.kind(), "failed to save the game session");
    }
    store(ctx, uid, server, text).await
}

pub async fn store(
    ctx: &AppContext,
    uid: &str,
    server: Server,
    text: String,
) -> Result<Value, AppError> {
    let data = ctx.snapshots.or_default(server)?;
    let default = ctx.snapshots.or_default(ctx.snapshots.default_server())?;
    let dump_dir = ctx.config.debug.sync_dump_dir.clone();
    let owner = uid.to_owned();
    let (raw, rows) = ctx
        .cpu
        .offload("sync_parse", move || {
            if !dump_dir.is_empty() {
                dump(&dump_dir, &owner, server, &text);
            }
            let raw: Value = serde_json::from_str(&text).map_err(AppError::internal)?;
            let now = chrono::Utc::now().timestamp();
            let rows = payload::rows(&raw, &data.model, &default.model.campaign_rotations, now)?;
            Ok::<_, AppError>((raw, rows))
        })
        .await??;
    let raw = Arc::new(raw);
    let game_story = progress::game_read_set(ctx, uid, server, Arc::clone(&raw)).await;
    tracing::info!(
        uid,
        story_block = game_story.present,
        key = ?game_story.key,
        groups = game_story.groups,
        stories = game_story.stories,
        flags = game_story.flags,
        flag_hits = game_story.flag_hits,
        flag_misses = game_story.flag_misses,
        flag_miss_sample = ?game_story.flag_miss_sample,
        review_hits = game_story.review_hits,
        stage_records = game_story.stage_records,
        stage_hits = game_story.stage_hits,
        archive_only_unread = game_story.archive_only_unread,
        union = game_story.read.len(),
        read = game_story.read_count(),
        sample = ?game_story.sample,
        "syncData story block census"
    );

    let mut conn = ctx.db.conn().await?;
    roster_repo::sync_user_data(&mut conn, &rows.input(uid, server)).await?;
    let found = users::find(&mut conn, uid, server).await?;
    drop(conn);
    invalidate_user(ctx, uid).await;

    let mut import =
        json!({ "ok": false, "error": "the account row was not found after the sync" });
    if let Some(user) = found.map(|row| UserId(row.id)) {
        import = progress::import_game_read(ctx, uid, user, &game_story).await;
        if let Err(e) = grading::enqueue_sync_regrade(ctx, user).await {
            tracing::error!(uid, error = %e, "the regrade was not queued; the roster sync stands");
        }
        if let Some(backfill) = sandbox::plan(user, &raw)
            && let Err(e) = sandbox::enqueue(ctx, &backfill).await
        {
            tracing::error!(uid, error = %e, "the RA topic backfill was not queued");
        }
    }
    let ownership = JobSpec {
        dedupe_key: Some(OWNERSHIP_DEDUPE.to_owned()),
        run_at: Some(Utc::now() + OWNERSHIP_DELAY),
        ..JobSpec::new(JobKind::OperatorOwnershipStats)
    };
    if let Err(e) = ctx.queue.enqueue(ownership).await {
        tracing::error!(uid, error = %e, "the ownership refresh was not queued");
    }
    let mut payload = Arc::try_unwrap(raw).unwrap_or_else(|shared| (*shared).clone());
    if let Some(object) = payload.as_object_mut() {
        object.insert("gameReadImport".to_owned(), import);
    }
    Ok(payload)
}

pub async fn after_grade(ctx: &AppContext, user: UserId) {
    let recorded = async {
        let account = users::account(&mut *ctx.db.conn().await?, user)
            .await?
            .ok_or(AppError::NotFound(None))?;
        let server =
            Server::from_index(account.server_id).unwrap_or_else(|| ctx.snapshots.default_server());
        let recorded = lookback::record(ctx, user, server).await;
        invalidate_user(ctx, &account.uid).await;
        recorded
    };
    if let Err(e) = recorded.await {
        tracing::error!(user = %user.0, error = %e, "the lookback snapshot failed; the grade stands");
    }
}

fn dump(dir: &str, uid: &str, server: Server, text: &str) {
    let path = std::path::Path::new(dir).join(format!(
        "{}_{uid}_{}.json",
        server.as_str(),
        chrono::Utc::now().timestamp()
    ));
    match std::fs::create_dir_all(dir).and_then(|()| std::fs::write(&path, text)) {
        Ok(()) => tracing::info!(uid, path = %path.display(), "syncData dumped"),
        Err(e) => tracing::warn!(uid, error = %e, "syncData dump failed"),
    }
}
