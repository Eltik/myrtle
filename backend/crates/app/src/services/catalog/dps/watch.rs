use chrono::Utc;
use myrtle_jobs::JobKind;
use myrtle_jobs::schedule;
use serde::{Deserialize, Serialize};

use crate::config::SecretRef;
use crate::{AppContext, AppError};

pub const MAX_BACKOFF_SECS: u64 = 6 * 3600;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WatchMemory {
    pub last_commit_sha: Option<String>,
    #[serde(default)]
    pub consecutive_failures: u32,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(default)]
    pub last_checked_at: i64,
    #[serde(default)]
    pub skipped_runs: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Head {
    pub sha: Option<String>,
    pub etag: Option<String>,
}

#[must_use]
pub fn interval_secs() -> u64 {
    schedule::DEFAULTS
        .iter()
        .find(|s| s.kind == JobKind::DpsUpstreamWatch)
        .map_or(3600, |s| s.every.as_secs())
}

#[must_use]
pub fn backoff_secs(base: u64, failures: u32) -> u64 {
    base.saturating_mul(1u64 << failures.min(6))
        .min(MAX_BACKOFF_SECS)
}

#[must_use]
pub fn backing_off(memory: &WatchMemory, base: u64, now: i64) -> bool {
    let backoff = backoff_secs(base, memory.consecutive_failures);
    let runs = backoff / base.max(1);
    let waited = now.saturating_sub(memory.last_checked_at);
    memory.consecutive_failures > 0
        && u64::from(memory.skipped_runs) + 1 < runs
        && u64::try_from(waited).unwrap_or(0) < backoff
}

#[must_use]
pub fn observe(
    mut memory: WatchMemory,
    checked: Option<Head>,
    now: i64,
) -> (WatchMemory, Option<String>) {
    memory.last_checked_at = now;
    memory.skipped_runs = 0;
    let Some(head) = checked else {
        memory.consecutive_failures = memory.consecutive_failures.saturating_add(1);
        return (memory, None);
    };
    memory.consecutive_failures = 0;
    memory.etag = head.etag;
    let fresh = head
        .sha
        .filter(|sha| memory.last_commit_sha.as_ref() != Some(sha));
    if let Some(sha) = &fresh {
        memory.last_commit_sha = Some(sha.clone());
    }
    (memory, fresh)
}

pub async fn check(
    client: &reqwest::Client,
    url: &str,
    etag: Option<&str>,
    token: Option<&str>,
) -> Option<Head> {
    request(client, url, etag, token)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "GitHub API check failed; backing off"))
        .ok()
        .flatten()
}

async fn request(
    client: &reqwest::Client,
    url: &str,
    etag: Option<&str>,
    token: Option<&str>,
) -> Result<Option<Head>, reqwest::Error> {
    let mut req = client.get(url).header("User-Agent", "myrtle-dps-watcher");
    if let Some(etag) = etag {
        req = req.header("If-None-Match", etag);
    }
    if let Some(token) = token {
        req = req.header("Authorization", format!("Bearer {token}"));
    }
    let resp = req.send().await?;
    let carried = etag.map(str::to_owned);
    let status = resp.status();
    if status == reqwest::StatusCode::NOT_MODIFIED {
        return Ok(Some(Head {
            sha: None,
            etag: carried,
        }));
    }
    if !status.is_success() {
        let header = |name: &str| {
            resp.headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("-")
                .to_owned()
        };
        tracing::warn!(
            %status,
            retry_after = header("retry-after"),
            rate_limit_remaining = header("x-ratelimit-remaining"),
            "GitHub API refused the DPS upstream check; backing off"
        );
        return Ok(None);
    }
    let etag = resp
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
        .or(carried);
    let commits: Vec<serde_json::Value> = resp.json().await?;
    let sha = commits
        .first()
        .and_then(|c| c["sha"].as_str())
        .map(str::to_owned);
    Ok(Some(Head { sha, etag }))
}

pub async fn watch_upstream(ctx: &AppContext) -> Result<String, AppError> {
    let watch = &ctx.config.jobs.dps_watch;
    if watch.every_secs == 0 {
        return Ok("DPS upstream watch is off".to_owned());
    }
    let kind = JobKind::DpsUpstreamWatch;
    let stored = schedule::memory(&mut *ctx.db.conn().await?, kind, None).await?;
    let mut memory: WatchMemory = serde_json::from_value(stored).unwrap_or_default();
    let now = Utc::now().timestamp();
    if backing_off(&memory, interval_secs(), now) {
        memory.skipped_runs = memory.skipped_runs.saturating_add(1);
        let stored = serde_json::to_value(&memory).map_err(AppError::internal)?;
        schedule::remember(&mut *ctx.db.conn().await?, kind, None, &stored).await?;
        return Ok(format!(
            "backing off after {} failures",
            memory.consecutive_failures
        ));
    }
    let url = format!(
        "https://api.github.com/repos/{}/commits?sha={}&per_page=1",
        watch.upstream_repo, watch.upstream_branch
    );
    let token = SecretRef::optional(ctx.config.secrets.github_token.as_ref());
    let checked = check(
        ctx.upstream.client(),
        &url,
        memory.etag.as_deref(),
        token.as_deref(),
    )
    .await;
    let (memory, fresh) = observe(memory, checked, now);
    let stored = serde_json::to_value(&memory).map_err(AppError::internal)?;
    schedule::remember(&mut *ctx.db.conn().await?, kind, None, &stored).await?;
    Ok(match fresh {
        Some(sha) => {
            tracing::warn!(
                sha,
                repo = watch.upstream_repo,
                branch = watch.upstream_branch,
                compare = format!(
                    "https://github.com/{}/commits/{}",
                    watch.upstream_repo, watch.upstream_branch
                ),
                "DPS upstream has a new commit; regenerate the formulas in CI and deploy normally"
            );
            format!("new upstream commit {sha}")
        }
        None if memory.consecutive_failures > 0 => {
            format!("check failed ({} in a row)", memory.consecutive_failures)
        }
        None => "no new upstream commit".to_owned(),
    })
}
