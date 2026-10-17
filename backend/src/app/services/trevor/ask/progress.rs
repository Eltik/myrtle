//! The reader's spoiler horizon: the stories their progress marks read
//! (re-derived from stage progress when nothing is stored), bucketed to the
//! active version's eras.

use std::collections::HashSet;

use serde_json::Value;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::services::story_progress;
use crate::app::services::trevor::{Era, Horizon, bucket_horizon, horizon_pos};
use crate::app::state::AppState;
use crate::core::hypergryph::constants::Server;
use crate::database::queries::trevor as q;

/// The stories a progress response says were read: the game's read flags
/// plus the site's own marks, minus what the reader marked unread.
fn reads_of(progress: story_progress::StoryProgressResponse) -> Vec<String> {
    let mut read: HashSet<String> = progress.game_read.into_iter().collect();
    if let Some(doc) = &progress.progress {
        if let Some(marks) = doc.0.get("read").and_then(Value::as_object) {
            read.extend(marks.keys().cloned());
        }
        if let Some(unread) = doc.0.get("unread").and_then(Value::as_object) {
            for id in unread.keys() {
                read.remove(id);
            }
        }
    }
    read.into_iter().collect()
}

/// Whether a progress response carries nothing to read from: no
/// `user_game_story_read` row at all (read or unread) and no site read mark.
fn holds_no_reads(progress: &story_progress::StoryProgressResponse) -> bool {
    progress.game_synced_at.is_none()
        && progress.game_read.is_empty()
        && progress.game_unread.is_empty()
        && progress.progress.as_ref().is_none_or(|doc| {
            doc.0
                .get("read")
                .and_then(Value::as_object)
                .is_none_or(serde_json::Map::is_empty)
        })
}

/// The stories the account has read: the game's read flags plus its own
/// marks, minus what it marked unread.
///
/// An account that has stage progress but no game read rows and no site
/// marks (2,628 of the 2,630 local accounts on 2026-10-17: the rows only
/// exist since a sync after 2026-09-24) would ask at gatePos 0. For it, the
/// first ask in this process re-derives and stores the game's verdict from
/// the stored stage progress ([`story_progress::reverdict`], the same rows a
/// sync writes) and reads again. Once per account per process, whatever it
/// yields; a failed re-derivation is logged and the ask goes on with what
/// was stored. `TREVOR_REVERDICT_ON_ASK=0` skips it.
pub async fn read_stories(
    state: &AppState,
    user_id: Uuid,
    server: &str,
) -> Result<Vec<String>, ApiError> {
    let progress = story_progress::get(state, user_id).await?;
    if !state.trevor.config().reverdict_on_ask || !holds_no_reads(&progress) {
        return Ok(reads_of(progress));
    }
    let Some(game_server) = Server::parse(server) else {
        return Ok(reads_of(progress));
    };
    if !state.trevor.first_reverdict(user_id) {
        return Ok(reads_of(progress));
    }
    let has_stages: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM user_stage_progress WHERE user_id = $1)")
            .bind(user_id)
            .fetch_one(&state.db)
            .await?;
    if !has_stages {
        return Ok(reads_of(progress));
    }
    state
        .trevor
        .reverdicts
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let started = std::time::Instant::now();
    match story_progress::reverdict(state, user_id, game_server).await {
        Ok(fresh) => {
            let read = reads_of(fresh);
            tracing::info!(
                %user_id,
                server,
                read = read.len(),
                ms = started.elapsed().as_millis(),
                "trevor: read verdict re-derived on ask"
            );
            Ok(read)
        }
        Err(e) => {
            tracing::warn!(
                %user_id,
                server,
                error = %e,
                ms = started.elapsed().as_millis(),
                "trevor: re-deriving the read verdict on ask failed; asking with the stored reads"
            );
            Ok(reads_of(progress))
        }
    }
}

/// The active version and its eras; an unparsable `eras` reads as none.
async fn active_eras(
    state: &AppState,
    server: &str,
) -> Result<Option<(String, Vec<Era>)>, ApiError> {
    Ok(q::active_version_eras(&state.db, server)
        .await?
        .map(|(version, eras)| {
            let eras = eras
                .and_then(|v| serde_json::from_value::<Vec<Era>>(v).ok())
                .unwrap_or_default();
            (version, eras)
        }))
}

/// The ask's horizon (contract section 1): the furthest gatePos the account
/// has read in the active version, lowered by `horizonStory`, bucketed to its
/// era. No active version means nothing is released: eraPos 0.
pub async fn resolve_horizon(
    state: &AppState,
    user_id: Uuid,
    server: &str,
    horizon_story: Option<&str>,
) -> Result<(Option<String>, Horizon), ApiError> {
    let buckets = state.trevor.config().era_buckets;
    let Some((version, eras)) = active_eras(state, server).await? else {
        return Ok((None, bucket_horizon(&[], 0, buckets)));
    };
    let read = read_stories(state, user_id, server).await?;
    let profile_pos = q::max_gate_pos(&state.db, &version, server, &read)
        .await?
        .unwrap_or(0);
    let story_pos = match horizon_story.map(str::trim).filter(|s| !s.is_empty()) {
        None => None,
        Some(story) => {
            if story.len() > 200 {
                return Err(ApiError::BadRequest("horizonStory is too long".into()));
            }
            match q::story_gate_pos(&state.db, &version, server, story).await? {
                Some(pos) => Some(pos.unwrap_or(0)),
                None => {
                    return Err(ApiError::BadRequest(format!(
                        "horizonStory {story} is not in the active Trevor version"
                    )));
                }
            }
        }
    };
    let pos = horizon_pos(profile_pos, story_pos);
    Ok((Some(version), bucket_horizon(&eras, pos, buckets)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn progress(
        doc: Option<Value>,
        game_read: &[&str],
        game_unread: &[&str],
    ) -> story_progress::StoryProgressResponse {
        let synced = !(game_read.is_empty() && game_unread.is_empty());
        story_progress::StoryProgressResponse {
            progress: doc.map(story_progress::StoryProgressDocument),
            updated_at: None,
            game_read: game_read.iter().map(|s| (*s).to_owned()).collect(),
            game_unread: game_unread.iter().map(|s| (*s).to_owned()).collect(),
            game_archived: 0,
            game_synced_at: synced.then_some(1),
        }
    }

    #[test]
    fn only_an_account_with_nothing_stored_is_re_derived() {
        assert!(holds_no_reads(&progress(None, &[], &[])));
        let empty = serde_json::json!({ "v": 2, "read": {}, "pos": {} });
        assert!(holds_no_reads(&progress(Some(empty), &[], &[])));
        // Unread marks alone are not reads: still re-derived.
        let unread = serde_json::json!({ "v": 2, "read": {}, "unread": { "a": 1 }, "pos": {} });
        assert!(holds_no_reads(&progress(Some(unread), &[], &[])));
        let marked = serde_json::json!({ "v": 2, "read": { "a": 1 }, "pos": {} });
        assert!(!holds_no_reads(&progress(Some(marked), &[], &[])));
        assert!(!holds_no_reads(&progress(None, &["a"], &[])));
        // Rows that are all unread are still a stored verdict.
        assert!(!holds_no_reads(&progress(None, &[], &["a"])));
    }
}
