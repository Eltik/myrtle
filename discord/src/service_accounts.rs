//! Announces a backend service account that needs a human to log it in again.
//!
//! The backend reads the game with its own accounts (EN, and the CN Bilibili channel `bili`).
//! When an account's durable token is refused, every job on that server stalls until someone
//! runs `set-session`. This poller reads `GET /api/service-accounts` and posts to the asset
//! channels on each transition: one embed when an account goes down, one when it is back.
//!
//! The last announced state per account lives in `service_account_announced`, so a bot restart
//! neither repeats an outage nor misses a recovery. A backend that cannot be reached is not an
//! account outage: it is logged and the poll is skipped.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serenity::all::{CreateEmbed, Http, Timestamp};
use sqlx::SqlitePool;

use crate::api::service_accounts::{AccountState, AccountStatus, service_accounts};
use crate::config::ServiceAccountsConfig;
use crate::db;
use crate::ui::{COLOR_BAD, COLOR_OK};
use crate::watcher::broadcast_one;

/// What one poll owes an account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub announce: Option<Announcement>,
    /// The state to store, when it differs from the stored one.
    pub remember: Option<AccountState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Announcement {
    /// Went down; `since` is the backend's first refusal of this outage.
    Refused { since: Option<i64> },
    /// Back up after an outage that started at `since`, as announced.
    Recovered { since: Option<i64> },
}

/// Decide one account's poll from the state last stored for it (`None` when never seen) and
/// what the backend reports now.
///
/// - Down while not announced as down: announce the outage.
/// - Up while announced as down: announce the recovery.
/// - `unknown` (the backend has not used the account since it started) changes nothing, so a
///   backend restart mid-outage neither repeats the outage nor fakes a recovery.
/// - A first sighting of a working account is stored without a post.
#[must_use]
pub fn step(previous: Option<(AccountState, Option<i64>)>, fetched: &AccountStatus) -> Step {
    let prev_state = previous.map(|(s, _)| s);
    match fetched.state {
        AccountState::Unknown => Step {
            announce: None,
            remember: None,
        },
        AccountState::Refused if prev_state != Some(AccountState::Refused) => Step {
            announce: Some(Announcement::Refused {
                since: fetched.refused_since,
            }),
            remember: Some(AccountState::Refused),
        },
        AccountState::Ok if prev_state == Some(AccountState::Refused) => Step {
            announce: Some(Announcement::Recovered {
                since: previous.and_then(|(_, since)| since),
            }),
            remember: Some(AccountState::Ok),
        },
        state => Step {
            announce: None,
            remember: (prev_state != Some(state)).then_some(state),
        },
    }
}

/// The asset-channel label for a backend server code. The `bili` account reads CN's data, so its
/// outages go to channels that hear CN.
#[must_use]
pub fn label_for(server: &str) -> String {
    match server.to_ascii_lowercase().as_str() {
        "bili" | "bilibili" | "cn" => "CN".to_string(),
        other => other.to_ascii_uppercase(),
    }
}

/// How to log the account on `server` in again, as embed markdown.
#[must_use]
pub fn relogin_steps(server: &str) -> String {
    let deploy = format!(
        "Then copy `backend/game_sessions/{server}.json` to the VPS's `backend/game_sessions/`. \
         The backend adopts the file on its next attempt; no restart needed."
    );
    if server == "bili" {
        format!(
            "In `backend/`, text a code to the account's phone (national number; add \
             `--country-code <c>` outside mainland China), then log in with it:\n```\n\
             cargo run --bin set-session -- --server bili --bili-phone <phone>\n\
             cargo run --bin set-session -- --server bili --bili-phone <phone> --otp <code>\n\
             ```\n{deploy}"
        )
    } else {
        format!(
            "In `backend/`:\n```\n\
             cargo run --bin set-session -- --server {server} --email <addr>\n\
             cargo run --bin set-session -- --server {server} --email <addr> --code <code>\n\
             ```\n{deploy}"
        )
    }
}

fn when(unix: i64) -> String {
    format!("<t:{unix}:f> (<t:{unix}:R>)")
}

pub fn build_embed(server: &str, announcement: Announcement) -> CreateEmbed {
    let embed = match announcement {
        Announcement::Refused { since } => {
            let embed = CreateEmbed::new()
                .title(format!("Service account needs a re-login ({server})"))
                .colour(COLOR_BAD)
                .description(format!(
                    "The backend's `{server}` game account was refused, so the jobs that use it \
                     (pool details, event shops, probes) are stalled until it is logged in again.\n\n{}",
                    relogin_steps(server)
                ));
            match since {
                Some(t) => embed.field("Refused since", when(t), true),
                None => embed,
            }
        }
        Announcement::Recovered { since } => {
            let embed = CreateEmbed::new()
                .title(format!("Service account back ({server})"))
                .colour(COLOR_OK)
                .description(format!(
                    "The backend's `{server}` game account logs in again; its jobs resume on their next run."
                ));
            match since {
                Some(t) => embed.field("Down since", when(t), true),
                None => embed,
            }
        }
    };
    embed
        .field("Server", label_for(server), true)
        .timestamp(Timestamp::now())
}

/// The poller. Returns at once when turned off in config or with no backend URL.
pub async fn run(
    http: Arc<Http>,
    pool: SqlitePool,
    client: reqwest::Client,
    backend: String,
    config: ServiceAccountsConfig,
) {
    if !config.enabled() {
        tracing::info!("service accounts: poller disabled in config");
        return;
    }
    if backend.is_empty() {
        tracing::info!("service accounts: endpoints.public_backend is empty, poller disabled");
        return;
    }
    let interval = Duration::from_secs(config.poll_secs());
    loop {
        tick(&http, &pool, &client, &backend).await;
        tokio::time::sleep(interval).await;
    }
}

async fn tick(http: &Arc<Http>, pool: &SqlitePool, client: &reqwest::Client, backend: &str) {
    let fetched = match service_accounts(client, backend).await {
        Ok(r) => r.accounts,
        Err(e) => {
            tracing::warn!("service accounts: backend unreachable, nothing announced: {e}");
            return;
        }
    };
    let stored: HashMap<String, (AccountState, Option<i64>)> =
        match db::list_service_account_states(pool).await {
            Ok(rows) => rows
                .into_iter()
                .map(|(server, (state, since))| {
                    (server, (AccountState::from_db_str(&state), since))
                })
                .collect(),
            Err(e) => {
                tracing::error!("service accounts: read stored states: {e}");
                return;
            }
        };

    for account in &fetched {
        let step = step(stored.get(&account.server).copied(), account);
        let Some(state) = step.remember else {
            continue;
        };
        if let Some(announcement) = step.announce
            && !announce(http, pool, &account.server, announcement).await
        {
            // Stored state untouched: the next poll tries again.
            continue;
        }
        let since = (state == AccountState::Refused)
            .then_some(account.refused_since)
            .flatten();
        if let Err(e) = db::set_service_account_state(
            pool,
            &account.server,
            state.as_db_str(),
            since,
            Timestamp::now().unix_timestamp(),
        )
        .await
        {
            tracing::error!("service accounts: store state for {}: {e}", account.server);
        }
    }
}

/// Post to every asset channel that hears the account's label. Returns whether the post is done:
/// it reached at least one channel, or no channel listens.
async fn announce(
    http: &Arc<Http>,
    pool: &SqlitePool,
    server: &str,
    announcement: Announcement,
) -> bool {
    let channels = match db::list_assets_channels(pool).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("service accounts: list channels: {e}");
            return false;
        }
    };
    let label = label_for(server);
    let targets: Vec<_> = channels
        .into_iter()
        .filter(|(_, b)| b.wants(&label))
        .collect();
    tracing::info!(
        "service accounts: {server} {announcement:?}, posting to {} channel(s)",
        targets.len()
    );
    if targets.is_empty() {
        return true;
    }
    let embed = build_embed(server, announcement);
    let mut delivered = false;
    for (_guild, binding) in targets {
        delivered |= broadcast_one(http, binding.channel_id, embed.clone()).await;
    }
    delivered
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fetched(state: AccountState, refused_since: Option<i64>) -> AccountStatus {
        AccountStatus {
            server: "en".into(),
            state,
            refused_since,
            last_ok_at: None,
        }
    }

    const OK: AccountState = AccountState::Ok;
    const REFUSED: AccountState = AccountState::Refused;
    const UNKNOWN: AccountState = AccountState::Unknown;

    #[test]
    fn an_outage_is_announced_once() {
        let down = fetched(REFUSED, Some(100));
        for previous in [None, Some((OK, None))] {
            assert_eq!(
                step(previous, &down),
                Step {
                    announce: Some(Announcement::Refused { since: Some(100) }),
                    remember: Some(REFUSED),
                },
                "{previous:?}"
            );
        }
        // Already announced: nothing, even when a backend restart moved `refused_since`.
        assert_eq!(
            step(Some((REFUSED, Some(100))), &fetched(REFUSED, Some(900))),
            Step {
                announce: None,
                remember: None
            }
        );
    }

    #[test]
    fn a_recovery_is_announced_with_the_stored_start() {
        assert_eq!(
            step(Some((REFUSED, Some(100))), &fetched(OK, None)),
            Step {
                announce: Some(Announcement::Recovered { since: Some(100) }),
                remember: Some(OK),
            }
        );
    }

    #[test]
    fn unknown_changes_nothing() {
        for previous in [None, Some((OK, None)), Some((REFUSED, Some(100)))] {
            assert_eq!(
                step(previous, &fetched(UNKNOWN, None)),
                Step {
                    announce: None,
                    remember: None
                },
                "{previous:?}"
            );
        }
    }

    #[test]
    fn a_working_account_is_stored_quietly() {
        assert_eq!(
            step(None, &fetched(OK, None)),
            Step {
                announce: None,
                remember: Some(OK)
            }
        );
        assert_eq!(
            step(Some((OK, None)), &fetched(OK, None)),
            Step {
                announce: None,
                remember: None
            }
        );
    }

    #[test]
    fn bili_maps_to_cn_and_the_rest_upper_case() {
        assert_eq!(label_for("en"), "EN");
        assert_eq!(label_for("bili"), "CN");
        assert_eq!(label_for("cn"), "CN");
        assert_eq!(label_for("jp"), "JP");
        assert_eq!(label_for("kr"), "KR");
    }

    #[test]
    fn relogin_steps_name_the_server() {
        let en = relogin_steps("en");
        assert!(en.contains("set-session -- --server en --email <addr> --code <code>"));
        assert!(en.contains("game_sessions/en.json"));
        let bili = relogin_steps("bili");
        assert!(bili.contains("--bili-phone"));
        assert!(bili.contains("game_sessions/bili.json"));
        assert!(!bili.contains("--email"));
    }
}
