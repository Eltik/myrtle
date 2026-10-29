//! Fetch the RA topics a sync does not carry.
//!
//! Since the 2026-10-28 EN update, `account/syncData` holds only the RA topic
//! the player has loaded in game (`sandboxPerm.topic`); every other topic is
//! `null` under `sandboxPerm.template`, with a two-field `summary`. Migration
//! v045 keeps a stored topic across such syncs, but a topic that was never
//! stored (a player first seen after the update, or one whose row was
//! overwritten before v045) stays missing and grades 0.
//!
//! The CN 2.7.81 client has no read-only topic call: `sandboxPerm/changeTopic`
//! (`SandboxPermChangeTopicRequest { topicId }`) is the only request that makes
//! the server send another topic. Measured on one EN account on 2026-10-29: the
//! switch answered `result: 0` with the full 21-key `sandbox_1` blob in
//! `playerDataDelta.modified.sandboxPerm.template`, the switch back 15 s later
//! answered `result: 0`, and a full syncData before and after differed in
//! `sandboxPerm.loadTs` and the response `ts` only.
//!
//! After a sync, for each topic the player has entered but the stored row
//! lacks: switch to it, store its blob, wait out the switch cooldown, switch
//! back to the loaded topic, then rescore the RA section. Once per topic per
//! player: v045 keeps the blob afterwards, and the topic only changes while it
//! is loaded, when a normal sync carries it.
//!
//! `SANDBOX_TOPIC_BACKFILL=0` (or `off`, `false`) turns it off; unset is on.
//! `SANDBOX_TOPIC_BACKFILL_WAIT_S` sets the wait (default 15; the client's
//! `CD_REQUEST_SWITCH_TOPIC` reads 10, unit not in the metadata).

use std::collections::HashSet;
use std::sync::{LazyLock, Mutex, PoisonError};
use std::time::Duration;

use serde_json::Value;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::services::game_session;
use crate::app::state::AppState;
use crate::core::grade::calculate::rescore_sandbox_section;
use crate::core::hypergryph::constants::{AuthSession, Server};
use crate::core::hypergryph::fetch::{FetchError, auth_request, read_body};
use crate::database::queries::sandbox::get_user_sandbox_progress;

const DEFAULT_WAIT: Duration = Duration::from_secs(15);

/// Accounts with a backfill in flight, so a second refresh does not start a
/// second pair of switches on the same account.
static IN_FLIGHT: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

/// A topic to fetch: its `template` key (`SANDBOX_V2`) and topic id (`sandbox_1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingTopic {
    pub template: String,
    pub topic: String,
}

fn enabled() -> bool {
    match std::env::var("SANDBOX_TOPIC_BACKFILL") {
        Err(_) => true,
        Ok(v) => !matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "0" | "off" | "false"
        ),
    }
}

fn wait() -> Duration {
    std::env::var("SANDBOX_TOPIC_BACKFILL_WAIT_S")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map_or(DEFAULT_WAIT, Duration::from_secs)
}

/// The topics `sync` lists in `sandboxPerm.summary` that are not loaded, that
/// the player has entered, and that `stored` holds no object for.
///
/// "Entered" is the `Obt/SandboxPerm/<topic>/<topic>_entry` status flag set to
/// 1 AND a positive number in the topic's summary (`day`, `baseLv`), so a topic
/// the player never opened is never loaded on their account by us: a first
/// load may start its intro, which a later sync would not undo.
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
        let Some(topics) = topics.as_object() else {
            continue;
        };
        for (topic, info) in topics {
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
            let held = stored
                .and_then(|s| s.get("template"))
                .and_then(|t| t.get(template))
                .and_then(|t| t.get(topic))
                .is_some_and(Value::is_object);
            if entered && progressed && !held {
                out.push(MissingTopic {
                    template: template.clone(),
                    topic: topic.clone(),
                });
            }
        }
    }
    out
}

/// The topic blob a `changeTopic` answer carries, when it carries one.
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

/// After a sync stored the player's data: fetch every topic `missing_topics`
/// names on a background task, so the switch cooldown never holds the refresh
/// request. `grading` is the sync's own grade task when it outlived its
/// budget; it is awaited first so its RA score cannot land after this one.
pub fn spawn(
    state: &AppState,
    uid: &str,
    user_id: Uuid,
    server: Server,
    session: &AuthSession,
    sync: &Value,
    grading: Option<JoinHandle<Result<(), ApiError>>>,
) {
    if !enabled() {
        return;
    }
    // Cheap pre-check on the payload alone: nothing listed, nothing to read.
    if missing_topics(sync, None).is_empty() {
        return;
    }
    {
        let mut in_flight = IN_FLIGHT.lock().unwrap_or_else(PoisonError::into_inner);
        if !in_flight.insert(uid.to_owned()) {
            return;
        }
    }
    let state = state.clone();
    let uid = uid.to_owned();
    let mut session = session.clone();
    let sync = sync.clone();
    tokio::spawn(async move {
        if let Some(grading) = grading {
            let _ = grading.await;
        }
        let result = run(&state, &uid, user_id, server, &mut session, &sync).await;
        IN_FLIGHT
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&uid);
        if let Err(e) = result {
            tracing::error!(uid = %uid, error = ?e, "RA topic backfill failed");
        }
    });
}

async fn run(
    state: &AppState,
    uid: &str,
    user_id: Uuid,
    server: Server,
    session: &mut AuthSession,
    sync: &Value,
) -> Result<(), ApiError> {
    let stored = get_user_sandbox_progress(&state.db, user_id).await?;
    let missing = missing_topics(sync, stored.as_ref());
    let Some(loaded) = sync
        .pointer("/user/sandboxPerm/topic")
        .and_then(Value::as_str)
        .map(str::to_owned)
    else {
        return Ok(());
    };
    let mut fetched = 0usize;
    for m in &missing {
        let switched = change_topic(state, session, server, &m.topic).await;
        game_session::save(state, uid, session).await;
        let response = match switched {
            Ok(r) => r,
            Err(e) => {
                // Refused: the loaded topic did not move, nothing to undo.
                tracing::warn!(uid, topic = %m.topic, error = ?e, "RA topic backfill: switch refused");
                continue;
            }
        };
        match blob_from_change(&response, m) {
            Some(blob) => {
                store_topic(state, user_id, m, blob).await?;
                fetched += 1;
                tracing::info!(uid, topic = %m.topic, "RA topic backfill: stored");
            }
            None => tracing::warn!(
                uid,
                topic = %m.topic,
                "RA topic backfill: the switch answered without the topic"
            ),
        }
        switch_back(state, uid, session, server, &loaded).await;
    }
    if fetched > 0 {
        let game_data = state.default_game_data();
        rescore_sandbox_section(&state.db, user_id, &game_data, false).await?;
        state.cache.invalidate(&CacheKey::User { uid }).await;
        state
            .cache
            .invalidate_by_prefix(&format!("improvements:{uid}:"))
            .await;
    }
    Ok(())
}

async fn change_topic(
    state: &AppState,
    session: &mut AuthSession,
    server: Server,
    topic: &str,
) -> Result<Value, FetchError> {
    let body = serde_json::json!({ "topicId": topic });
    let response = auth_request(
        &state.http_client,
        "sandboxPerm/changeTopic",
        Some(&body),
        session,
        server,
    )
    .await?;
    let text = read_body(response, "sandboxPerm/changeTopic").await?;
    serde_json::from_str(&text)
        .map_err(|e| FetchError::ParseError(format!("sandboxPerm/changeTopic: {e}")))
}

/// Back to the topic the player had loaded, after the cooldown; one retry
/// after a second wait. A failure leaves the account on the fetched topic,
/// which the player can switch back in game.
async fn switch_back(
    state: &AppState,
    uid: &str,
    session: &mut AuthSession,
    server: Server,
    loaded: &str,
) {
    for attempt in 1..=2 {
        tokio::time::sleep(wait()).await;
        let result = change_topic(state, session, server, loaded).await;
        game_session::save(state, uid, session).await;
        match result {
            Ok(r) if r.get("result").and_then(Value::as_i64) == Some(0) => return,
            Ok(r) => {
                tracing::warn!(uid, attempt, result = ?r.get("result"), "RA topic backfill: switch back answered non-zero")
            }
            Err(e) => {
                tracing::warn!(uid, attempt, error = ?e, "RA topic backfill: switch back failed")
            }
        }
    }
    tracing::error!(
        uid,
        loaded,
        "RA topic backfill could not switch back; the account is left on the fetched topic"
    );
}

/// Write one topic's blob into the stored row, creating `template.<key>` when
/// absent. Migration v045 keeps every other stored topic.
async fn store_topic(
    state: &AppState,
    user_id: Uuid,
    m: &MissingTopic,
    blob: &Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE user_sandbox_progress SET progress = jsonb_set( \
             jsonb_set(progress, ARRAY['template', $2], \
                 CASE WHEN jsonb_typeof(progress #> ARRAY['template', $2]) = 'object' \
                      THEN progress #> ARRAY['template', $2] ELSE '{}'::jsonb END, true), \
             ARRAY['template', $2, $3], $4, true) \
         WHERE user_id = $1 AND jsonb_typeof(progress -> 'template') = 'object'",
    )
    .bind(user_id)
    .bind(&m.template)
    .bind(&m.topic)
    .bind(blob)
    .execute(&state.db)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{MissingTopic, blob_from_change, missing_topics};
    use serde_json::json;

    /// The shape of the EN sync of 2026-10-29: `sandbox_2` loaded, `sandbox_1`
    /// null, both in `summary`, `sandbox_1` entered.
    fn sync() -> serde_json::Value {
        json!({ "user": {
            "status": { "flags": {
                "Obt/SandboxPerm/sandbox_1/sandbox_1_entry": 1,
                "Obt/SandboxPerm/sandbox_1/sandbox_1_ending_1": 1,
            } },
            "sandboxPerm": {
                "topic": "sandbox_2",
                "template": {
                    "SANDBOX_V2": { "sandbox_1": null },
                    "SANDBOX_V3": { "sandbox_2": { "base": { "level": 2 } } },
                },
                "summary": {
                    "SANDBOX_V2": { "sandbox_1": { "day": 42, "inChallenge": false } },
                    "SANDBOX_V3": { "sandbox_2": { "baseLv": 2, "inCurrent": true } },
                },
            },
        } })
    }

    fn sandbox_1() -> MissingTopic {
        MissingTopic {
            template: "SANDBOX_V2".into(),
            topic: "sandbox_1".into(),
        }
    }

    #[test]
    fn an_entered_unloaded_topic_the_row_lacks_is_fetched() {
        let stored = json!({ "template": { "SANDBOX_V2": { "sandbox_1": null } } });
        assert_eq!(missing_topics(&sync(), Some(&stored)), vec![sandbox_1()]);
        assert_eq!(missing_topics(&sync(), None), vec![sandbox_1()]);
    }

    #[test]
    fn a_stored_topic_is_not_fetched_again() {
        let stored = json!({ "template": { "SANDBOX_V2": { "sandbox_1": { "tech": {} } } } });
        assert!(missing_topics(&sync(), Some(&stored)).is_empty());
    }

    #[test]
    fn a_topic_never_entered_is_never_loaded() {
        let mut s = sync();
        s["user"]["status"]["flags"]
            .as_object_mut()
            .unwrap()
            .remove("Obt/SandboxPerm/sandbox_1/sandbox_1_entry");
        assert!(missing_topics(&s, None).is_empty(), "no entry flag");
        let mut s = sync();
        s["user"]["sandboxPerm"]["summary"]["SANDBOX_V2"]["sandbox_1"] =
            json!({ "day": 0, "inChallenge": false });
        assert!(
            missing_topics(&s, None).is_empty(),
            "no progress in the summary"
        );
    }

    #[test]
    fn the_loaded_topic_and_a_pre_update_sync_are_left_alone() {
        let mut s = sync();
        s["user"]["sandboxPerm"]["topic"] = json!("sandbox_1");
        assert!(missing_topics(&s, None).is_empty());
        // Before 2026-10-28: no summary, every topic carried.
        let old = json!({ "user": { "sandboxPerm": {
            "template": { "SANDBOX_V2": { "sandbox_1": { "tech": {} } } },
        } } });
        assert!(missing_topics(&old, None).is_empty());
    }

    #[test]
    fn the_blob_is_read_from_a_successful_switch_only() {
        let ok = json!({ "result": 0, "playerDataDelta": { "modified": { "sandboxPerm": {
            "template": { "SANDBOX_V2": { "sandbox_1": { "tech": { "unlock": [] } } },
                          "SANDBOX_V3": { "sandbox_2": null } },
            "topic": "sandbox_1",
        } }, "deleted": {} } });
        assert!(blob_from_change(&ok, &sandbox_1()).is_some());
        let mut refused = ok.clone();
        refused["result"] = json!(1);
        assert!(blob_from_change(&refused, &sandbox_1()).is_none());
        let mut empty = ok;
        empty["playerDataDelta"]["modified"]["sandboxPerm"]["template"]["SANDBOX_V2"]["sandbox_1"] =
            json!(null);
        assert!(blob_from_change(&empty, &sandbox_1()).is_none());
    }
}
