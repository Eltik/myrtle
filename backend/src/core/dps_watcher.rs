use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::app::state::AppState;

// ── Configuration ───────────────────────────────────────────────────────

struct DpsWatcherConfig {
    poll_interval: u64,
    upstream_repo: String,
    upstream_branch: String,
    local_repo_path: String,
    state_file: String,
    auto_build: bool,
    auto_restart: bool,
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
            local_repo_path: std::env::var("DPS_LOCAL_REPO_PATH")
                .unwrap_or_else(|_| "external/ArknightsDpsCompare".into()),
            state_file: std::env::var("DPS_STATE_FILE")
                .unwrap_or_else(|_| ".dps-updater-state.json".into()),
            auto_build: std::env::var("DPS_AUTO_BUILD")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true),
            auto_restart: std::env::var("DPS_AUTO_RESTART")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
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

fn save_state(path: &str, state: &WatcherState) {
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = std::fs::write(path, json);
    }
}

// ── GitHub polling ──────────────────────────────────────────────────────

enum CheckResult {
    NoChange,
    NewCommit { sha: String },
}

async fn check_upstream(
    client: &reqwest::Client,
    config: &DpsWatcherConfig,
    etag: Option<&str>,
) -> Result<(CheckResult, Option<String>), reqwest::Error> {
    let url = format!(
        "https://api.github.com/repos/{}/commits?sha={}&per_page=1",
        config.upstream_repo, config.upstream_branch,
    );

    let mut req = client.get(&url).header("User-Agent", "myrtle-dps-watcher");

    if let Some(etag) = etag {
        req = req.header("If-None-Match", etag);
    }

    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        req = req.header("Authorization", format!("Bearer {token}"));
    }

    let resp = req.send().await?;

    if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
        return Ok((CheckResult::NoChange, etag.map(String::from)));
    }

    let new_etag = resp
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    if !resp.status().is_success() {
        tracing::warn!(status = %resp.status(), "GitHub API returned non-success status");
        return Ok((CheckResult::NoChange, new_etag));
    }

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

// ── Update pipeline ─────────────────────────────────────────────────────

async fn run_command(program: &str, args: &[&str], label: &str) -> Result<(), String> {
    tracing::info!(label, cmd = %format!("{program} {}", args.join(" ")), "running");

    let output = tokio::process::Command::new(program)
        .args(args)
        .output()
        .await
        .map_err(|e| format!("{label}: failed to spawn: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "{label}: exit code {:?}\nstdout: {stdout}\nstderr: {stderr}",
            output.status.code()
        ));
    }

    tracing::info!(label, "completed successfully");
    Ok(())
}

async fn run_update_pipeline(config: &DpsWatcherConfig) -> Result<(), String> {
    let local = &config.local_repo_path;
    let branch = &config.upstream_branch;

    // 1. Fetch latest
    run_command(
        "git",
        &["-C", local, "fetch", "origin", branch],
        "git fetch",
    )
    .await?;

    // 2. Reset to upstream (no merge conflicts since we never modify locally)
    let reset_ref = format!("origin/{branch}");
    run_command(
        "git",
        &["-C", local, "reset", "--hard", &reset_ref],
        "git reset",
    )
    .await?;

    // 3. Regenerate formulas + transpile (skip --expected, that's for dev/CI)
    run_command(
        "cargo",
        &[
            "run",
            "--bin",
            "generate-dps",
            "--",
            "--formulas",
            "--transpile",
        ],
        "generate-dps",
    )
    .await?;

    // 4. Build the backend binary (if enabled)
    if config.auto_build {
        run_command(
            "cargo",
            &["build", "--release", "--bin", "backend"],
            "cargo build",
        )
        .await?;
    }

    Ok(())
}

// ── Poll loop ───────────────────────────────────────────────────────────

async fn poll_loop(config: DpsWatcherConfig, state: AppState) {
    let mut etag: Option<String> = None;
    let mut watcher_state = load_state(&config.state_file);

    tracing::info!(
        repo = %config.upstream_repo,
        branch = %config.upstream_branch,
        interval_secs = config.poll_interval,
        auto_build = config.auto_build,
        auto_restart = config.auto_restart,
        "DPS watcher started"
    );

    let mut first_run = true;

    loop {
        if first_run {
            first_run = false;
        } else {
            tokio::time::sleep(Duration::from_secs(config.poll_interval)).await;
        }

        let result = check_upstream(&state.http_client, &config, etag.as_deref()).await;

        match result {
            Ok((CheckResult::NoChange, new_etag)) => {
                etag = new_etag;
                tracing::debug!("DPS upstream: no changes");
            }
            Ok((CheckResult::NewCommit { sha }, new_etag)) => {
                etag = new_etag;

                if watcher_state.last_commit_sha.as_ref() == Some(&sha) {
                    tracing::debug!(sha = %sha, "DPS upstream: commit already processed");
                    continue;
                }

                tracing::info!(sha = %sha, "DPS upstream: new commit detected");

                match run_update_pipeline(&config).await {
                    Ok(()) => {
                        tracing::info!(sha = %sha, "DPS update pipeline completed successfully");
                        watcher_state.last_commit_sha = Some(sha);
                        watcher_state.consecutive_failures = 0;
                        save_state(&config.state_file, &watcher_state);

                        if config.auto_restart {
                            tracing::info!(
                                "DPS update complete, initiating graceful shutdown for restart"
                            );
                            // Exit the process; a process supervisor (systemd/Docker) restarts
                            // with the newly built binary
                            std::process::exit(0);
                        }
                    }
                    Err(e) => {
                        watcher_state.consecutive_failures += 1;
                        save_state(&config.state_file, &watcher_state);
                        tracing::error!(
                            error = %e,
                            failures = watcher_state.consecutive_failures,
                            "DPS update pipeline failed"
                        );
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "GitHub API check failed, will retry next cycle");
            }
        }
    }
}

// ── Public API ──────────────────────────────────────────────────────────

pub fn spawn(state: AppState) {
    let config = match DpsWatcherConfig::from_env() {
        Some(c) => c,
        None => {
            tracing::info!("DPS_POLL_INTERVAL not set, DPS auto-update disabled");
            return;
        }
    };

    // Verify local repo exists
    if !Path::new(&config.local_repo_path).exists() {
        tracing::warn!(
            path = %config.local_repo_path,
            "DPS local repo not found, auto-update disabled"
        );
        return;
    }

    tokio::spawn(async move {
        poll_loop(config, state).await;
    });
}
