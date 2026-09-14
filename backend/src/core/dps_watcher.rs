//! Watches the upstream DPS reference repository and reports when it moves.
//!
//! Report-only by design. It does not fetch, check out, regenerate or rebuild
//! anything, and it never restarts the process: regenerating
//! `dps/custom/generated.rs` from a third party's commit is a reviewed change
//! belonging in CI and a normal deploy, not something a server does to itself
//! while serving.
//!
//! So the contract is narrow - notice the upstream commit, say so once, and
//! back off when the API is unavailable.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::app::state::AppState;

// ── Configuration ───────────────────────────────────────────────────────

struct DpsWatcherConfig {
    poll_interval: u64,
    upstream_repo: String,
    upstream_branch: String,
    state_file: String,
}

impl DpsWatcherConfig {
    fn from_env() -> Option<Self> {
        let poll_interval: u64 = std::env::var("DPS_POLL_INTERVAL")
            .ok()
            .and_then(|v| v.parse().ok())?;

        Some(Self {
            poll_interval,
            upstream_repo: std::env::var("DPS_UPSTREAM_REPO")
                .unwrap_or_else(|_| "WhoAteMyCQQkie/ArknightsDpsCompare".into()),
            upstream_branch: std::env::var("DPS_UPSTREAM_BRANCH").unwrap_or_else(|_| "main".into()),
            state_file: std::env::var("DPS_STATE_FILE")
                .unwrap_or_else(|_| ".dps-updater-state.json".into()),
        })
    }
}

// ── Persisted state ─────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Default)]
struct WatcherState {
    last_commit_sha: Option<String>,
    #[serde(default)]
    consecutive_failures: u32,
}

fn load_state(path: &str) -> WatcherState {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Write via a temp file and rename, so a crash mid-write cannot leave a
/// truncated JSON document. `load_state` discards anything it cannot parse, so
/// a torn write would lose the last-seen sha and re-report a handled commit.
fn save_state(path: &str, state: &WatcherState) {
    let Ok(json) = serde_json::to_string_pretty(state) else {
        return;
    };
    let tmp = format!("{path}.tmp");
    if std::fs::write(&tmp, json).is_ok() && std::fs::rename(&tmp, path).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
}

// ── GitHub polling ──────────────────────────────────────────────────────

enum CheckResult {
    NoChange,
    NewCommit { sha: String },
}

/// Poll the upstream default branch for its newest commit.
///
/// The returned `ETag` is the one to send next time. On any non-success status
/// the previous etag is carried forward rather than replaced: an error response
/// carries no useful validator, and discarding the good one would turn every
/// later poll into a full response against the same rate limit that just
/// rejected this one.
async fn check_upstream(
    client: &reqwest::Client,
    config: &DpsWatcherConfig,
    etag: Option<&str>,
) -> Result<(CheckResult, Option<String>), reqwest::Error> {
    let url = format!(
        "https://api.github.com/repos/{}/commits?sha={}&per_page=1",
        config.upstream_repo, config.upstream_branch,
    );

    let mut req = client
        .get(&url)
        .timeout(Duration::from_secs(15))
        .header("User-Agent", "myrtle-dps-watcher");

    if let Some(etag) = etag {
        req = req.header("If-None-Match", etag);
    }

    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        req = req.header("Authorization", format!("Bearer {token}"));
    }

    let resp = req.send().await?;
    let carried = etag.map(String::from);

    if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
        return Ok((CheckResult::NoChange, carried));
    }

    if !resp.status().is_success() {
        let retry_after = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("-");
        let remaining = resp
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("-");
        tracing::warn!(
            status = %resp.status(),
            retry_after,
            rate_limit_remaining = remaining,
            "GitHub API returned non-success status"
        );
        return Ok((CheckResult::NoChange, carried));
    }

    let new_etag = resp
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .or(carried);

    let commits: Vec<serde_json::Value> = resp.json().await?;
    let sha = commits
        .first()
        .and_then(|c| c["sha"].as_str())
        .map(String::from);

    match sha {
        Some(sha) => Ok((CheckResult::NewCommit { sha }, new_etag)),
        None => Ok((CheckResult::NoChange, new_etag)),
    }
}

// ── Poll loop ───────────────────────────────────────────────────────────

/// Cap on the exponential backoff applied after repeated check failures, so a
/// GitHub outage degrades to one poll an hour instead of hammering it.
const MAX_BACKOFF_SECS: u64 = 3600;

fn backoff_secs(base: u64, failures: u32) -> u64 {
    base.saturating_mul(1u64 << failures.min(6))
        .min(MAX_BACKOFF_SECS)
}

async fn poll_loop(config: DpsWatcherConfig, state: AppState) {
    let mut etag: Option<String> = None;
    let mut watcher_state = load_state(&config.state_file);

    tracing::info!(
        repo = %config.upstream_repo,
        branch = %config.upstream_branch,
        interval_secs = config.poll_interval,
        "DPS watcher started (report-only; it does not modify or restart this process)"
    );

    let mut first_run = true;

    loop {
        if first_run {
            first_run = false;
        } else {
            let wait = backoff_secs(config.poll_interval, watcher_state.consecutive_failures);
            tokio::time::sleep(Duration::from_secs(wait)).await;
        }

        match check_upstream(&state.http_client, &config, etag.as_deref()).await {
            Ok((CheckResult::NoChange, new_etag)) => {
                etag = new_etag;
                watcher_state.consecutive_failures = 0;
                tracing::debug!("DPS upstream: no changes");
            }
            Ok((CheckResult::NewCommit { sha }, new_etag)) => {
                etag = new_etag;
                watcher_state.consecutive_failures = 0;

                if watcher_state.last_commit_sha.as_ref() == Some(&sha) {
                    tracing::debug!(sha = %sha, "DPS upstream: commit already reported");
                    continue;
                }

                tracing::warn!(
                    sha = %sha,
                    repo = %config.upstream_repo,
                    branch = %config.upstream_branch,
                    compare = %format!(
                        "https://github.com/{}/commits/{}",
                        config.upstream_repo, config.upstream_branch
                    ),
                    "DPS upstream has a new commit; regenerate dps/custom/generated.rs in CI \
                     (cargo run --bin generate-dps -- --formulas --transpile) and deploy normally"
                );
                watcher_state.last_commit_sha = Some(sha);
                save_state(&config.state_file, &watcher_state);
            }
            Err(e) => {
                watcher_state.consecutive_failures =
                    watcher_state.consecutive_failures.saturating_add(1);
                tracing::warn!(
                    error = %e,
                    failures = watcher_state.consecutive_failures,
                    "GitHub API check failed; backing off"
                );
            }
        }
    }
}

// ── Public API ──────────────────────────────────────────────────────────

pub fn spawn(state: AppState) {
    let Some(config) = DpsWatcherConfig::from_env() else {
        tracing::info!("DPS_POLL_INTERVAL not set, DPS upstream watcher disabled");
        return;
    };

    // No local checkout is involved, so there is nothing to verify here.
    tokio::spawn(async move {
        poll_loop(config, state).await;
    });
}

#[cfg(test)]
mod tests {
    use super::{MAX_BACKOFF_SECS, backoff_secs};

    #[test]
    fn backoff_doubles_then_caps() {
        assert_eq!(backoff_secs(60, 0), 60);
        assert_eq!(backoff_secs(60, 1), 120);
        assert_eq!(backoff_secs(60, 4), 960);
        // Doubling stops at 2^6, and the cap holds for any failure count.
        assert_eq!(backoff_secs(60, 6), 3840_u64.min(MAX_BACKOFF_SECS));
        assert_eq!(backoff_secs(60, 99), MAX_BACKOFF_SECS);
        // A large base cannot overflow into a short sleep.
        assert_eq!(backoff_secs(u64::MAX, 3), MAX_BACKOFF_SECS);
    }
}
