use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use dashmap::DashMap;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::grade::stages::types::StageClear;

pub struct UserStageData {
    pub clears: HashMap<String, StageClear>,
    /// Unix-seconds timestamp of the row's last update, used by the event-pool
    /// scorer to skip activities that started after the user's last sync.
    pub last_synced_ts: Option<i64>,
}

/// How long a server's known-stage set is reused, from `KNOWN_STAGES_CACHE_SECS`.
/// Default 300. `0` turns the cache off and restores the one-query-per-call
/// behaviour EXACTLY; a value that is not a whole number of seconds is ignored
/// with a warning.
static KNOWN_STAGES_TTL: LazyLock<Duration> =
    LazyLock::new(|| known_stages_ttl(std::env::var("KNOWN_STAGES_CACHE_SECS").ok().as_deref()));

const KNOWN_STAGES_TTL_DEFAULT: Duration = Duration::from_mins(5);

fn known_stages_ttl(raw: Option<&str>) -> Duration {
    let Some(raw) = raw else {
        return KNOWN_STAGES_TTL_DEFAULT;
    };
    raw.trim().parse::<u64>().map_or_else(
        |_| {
            tracing::warn!(
                value = raw,
                "KNOWN_STAGES_CACHE_SECS is not a whole number of seconds; using 300"
            );
            KNOWN_STAGES_TTL_DEFAULT
        },
        Duration::from_secs,
    )
}

/// The stage ids known on one server, shared by every caller that reads them.
type KnownStages = Arc<HashSet<String>>;

/// One server's set and when it was read, keyed by `servers.id`.
static KNOWN_STAGES: LazyLock<DashMap<i16, (Instant, KnownStages)>> = LazyLock::new(DashMap::new);

/// The cached set for `server_id` while it is younger than `ttl`.
fn cached_known_stages(server_id: i16, ttl: Duration) -> Option<KnownStages> {
    KNOWN_STAGES.get(&server_id).and_then(|entry| {
        let (read_at, set) = entry.value();
        (read_at.elapsed() < ttl).then(|| Arc::clone(set))
    })
}

/// Drop `server_id`'s cached set, so the next read scans again.
///
/// The set is a function of `user_stage_progress`, which only a sync writes, so
/// the sync calls this once its rows are in: the grade that follows sees the
/// stages that sync brought, as it did before the cache. What the cache can
/// still lag on is an account deletion, by at most the TTL, and only by keeping
/// a stage that account alone had seen.
pub fn forget_known_stage_ids(server_id: i16) {
    KNOWN_STAGES.remove(&server_id);
}

/// Stage ids in the dungeon JSON of at least one user on `user_id`'s server.
///
/// Bundled gamedata can be ahead of the user's server (EN players graded
/// against a CN-era `stage_table`), leaving stages nobody there can clear; this
/// set caps each server's universe at content live there. Key presence alone
/// counts (any `state`, `state: 0` "unlocked but not cleared" included), so a
/// running event is available before anyone has cleared it.
///
/// The scan expands every `stages` object on the server, and it used to run
/// once per graded user and once per improvements miss: a regrade pass paid it
/// once per player, so the pass grew with the square of the player count. The
/// set is the same for everyone on a server, so it is read once and reused for
/// [`KNOWN_STAGES_TTL`]; what each call still pays is the primary-key read that
/// names the user's server.
pub async fn get_known_stage_ids_for_server(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<KnownStages, sqlx::Error> {
    let ttl = *KNOWN_STAGES_TTL;
    let server_id: Option<i16> = sqlx::query_scalar("SELECT server_id FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    // No such user: no server, so no stages, which is what the scan answered.
    let Some(server_id) = server_id else {
        return Ok(Arc::new(HashSet::new()));
    };
    if !ttl.is_zero()
        && let Some(hit) = cached_known_stages(server_id, ttl)
    {
        return Ok(hit);
    }

    let rows: Vec<(String,)> = sqlx::query_as(
        r"
        SELECT DISTINCT key
        FROM users u
        JOIN user_stage_progress usp ON usp.user_id = u.id
        CROSS JOIN LATERAL jsonb_object_keys(usp.stages) AS key
        WHERE u.server_id = $1
        ",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;
    let set: KnownStages = Arc::new(rows.into_iter().map(|(s,)| s).collect());
    if !ttl.is_zero() {
        KNOWN_STAGES.insert(server_id, (Instant::now(), Arc::clone(&set)));
    }
    Ok(set)
}

pub async fn get_user_stage_clears(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<UserStageData, sqlx::Error> {
    let row: Option<(serde_json::Value, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        r"
        SELECT usp.stages, u.updated_at
        FROM user_stage_progress usp
        JOIN users u ON u.id = usp.user_id
        WHERE usp.user_id = $1
        ",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let Some((json, updated_at)) = row else {
        return Ok(UserStageData {
            clears: HashMap::new(),
            last_synced_ts: None,
        });
    };

    let last_synced_ts = updated_at.map(|ts| ts.timestamp());

    let Some(obj) = json.as_object() else {
        return Ok(UserStageData {
            clears: HashMap::new(),
            last_synced_ts,
        });
    };

    let mut clears = HashMap::with_capacity(obj.len());
    for (stage_id, entry) in obj {
        let state = entry
            .get("state")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0) as i16;
        // Records folded from mission, medal and unlock evidence at sync time
        // (`stage_evidence`); rows synced before 2026-09-15 tag them with a
        // string instead of `true`.
        let inferred = entry
            .get("inferred")
            .is_some_and(|v| v.as_bool() == Some(true) || v.is_string());
        let state_max = entry
            .get("stateMax")
            .and_then(serde_json::Value::as_i64)
            .map_or(state, |v| v as i16);
        let complete_times = entry
            .get("completeTimes")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0) as i32;
        let practice_times = entry
            .get("practiceTimes")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0) as i32;

        clears.insert(
            stage_id.clone(),
            StageClear {
                state,
                state_max,
                inferred,
                complete_times,
                practice_times,
            },
        );
    }

    Ok(UserStageData {
        clears,
        last_synced_ts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ttl_defaults_to_five_minutes_and_zero_turns_the_cache_off() {
        assert_eq!(known_stages_ttl(None), Duration::from_mins(5));
        assert_eq!(known_stages_ttl(Some("45")), Duration::from_secs(45));
        assert_eq!(known_stages_ttl(Some(" 900 ")), Duration::from_mins(15));
        assert!(known_stages_ttl(Some("0")).is_zero());
        for bad in ["", "soon", "-1", "1.5"] {
            assert_eq!(
                known_stages_ttl(Some(bad)),
                Duration::from_mins(5),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn a_cached_set_is_served_until_it_expires_or_is_forgotten() {
        // Server ids no real server uses, so this cannot meet another test's entry.
        let (fresh, stale, forgotten) = (i16::MAX, i16::MAX - 1, i16::MAX - 2);
        let set: KnownStages = Arc::new(HashSet::from(["main_01-07".to_owned()]));
        let ttl = Duration::from_mins(5);

        KNOWN_STAGES.insert(fresh, (Instant::now(), Arc::clone(&set)));
        assert_eq!(cached_known_stages(fresh, ttl).as_deref(), Some(&*set));

        // Read six minutes ago: past the TTL, so it is a miss.
        if let Some(long_ago) = Instant::now().checked_sub(Duration::from_mins(6)) {
            KNOWN_STAGES.insert(stale, (long_ago, Arc::clone(&set)));
            assert!(cached_known_stages(stale, ttl).is_none());
        }

        KNOWN_STAGES.insert(forgotten, (Instant::now(), Arc::clone(&set)));
        forget_known_stage_ids(forgotten);
        assert!(cached_known_stages(forgotten, ttl).is_none());

        // A server nothing was read for.
        assert!(cached_known_stages(i16::MAX - 3, ttl).is_none());

        for id in [fresh, stale] {
            KNOWN_STAGES.remove(&id);
        }
    }
}
