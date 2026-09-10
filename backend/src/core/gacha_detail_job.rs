//! Keeps each server's gacha pool-detail sidecar current.
//!
//! A banner's full rate-up roster lives on the game server, not in
//! `gacha_table.json` (see [`super::gamedata::types::gacha_detail`]).
//! This job walks every server that has a service account, fetches the pool
//! details it is missing, writes them beside that server's gamedata, and
//! triggers a reload so the running process picks them up.
//!
//! # Cost
//!
//! An ended banner is immutable, so each pool is fetched once and cached
//! indefinitely; only pools whose `end_time` is still in the future are
//! re-fetched. A cold start walks the full pool list, every run after that is a
//! handful of calls.
//!
//! # Concurrency
//!
//! Calls are serial *per account* because `seqnum` is sequential per session
//! ([`ServiceAccount::request`] enforces this). Different servers use different
//! accounts and so are refreshed concurrently.
//!
//! [`ServiceAccount::request`]: super::service_account::ServiceAccount::request

use std::{path::Path, time::Duration};

use serde_json::json;

use crate::{
    app::state::AppState,
    core::{
        asset_watcher,
        gamedata::types::gacha_detail::{
            GachaPoolDetail, POOL_DETAIL_FILE_VERSION, PoolDetailFile, pool_detail_path,
        },
        hypergryph::{constants::Server, fetch::FetchError},
    },
};

const ENDPOINT: &str = "gacha/getPoolDetail";
const DEFAULT_INTERVAL_SECS: u64 = 6 * 60 * 60;

/// Pacing between calls on one account. The upstream rate limit is unpublished,
/// so this is deliberate insurance.
const DEFAULT_CALL_DELAY_MS: u64 = 120;

/// Abandon a server's run after this many consecutive per-pool failures, so a
/// server refusing everything cannot burn the whole pool list.
const DEFAULT_MAX_FAILURES: u32 = 5;

/// Flush the sidecar every N successful fetches.
///
/// An abort inside the loop already falls through to the final write, so this
/// exists for the ungraceful cases -- a crash, a kill, a redeploy mid-walk --
/// where nothing would otherwise reach disk. Because the next run only asks for
/// pools it has no entry for, a checkpoint is all that is needed to make a cold
/// backfill resumable.
const DEFAULT_CHECKPOINT_EVERY: usize = 50;

fn env_or<T: std::str::FromStr>(key: &str, fallback: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(fallback)
}

pub fn spawn(state: AppState) {
    if state.service_accounts.is_empty() {
        tracing::info!("gacha detail job disabled - no service accounts");
        return;
    }

    let interval_secs: u64 = env_or("GACHA_DETAIL_REFRESH_SECS", DEFAULT_INTERVAL_SECS);
    tokio::spawn(async move {
        run_loop(state, interval_secs).await;
    });
}

async fn run_loop(state: AppState, interval_secs: u64) {
    tracing::info!(
        interval_secs,
        accounts = state.service_accounts.len(),
        "gacha detail job started"
    );
    let mut tick = tokio::time::interval(Duration::from_secs(interval_secs));
    loop {
        tick.tick().await;
        refresh_all(&state).await;
    }
}

/// Refresh every server that has a service account, concurrently.
/// One pass, shared by the loop and by `core::refresh`.
///
/// `refresh_all` reports per-server outcomes through tracing and never fails as
/// a whole, because one unreachable server must not stop the others; this
/// wrapper exists so the task registry sees the same uniform shape as the rest.
pub async fn refresh_once(state: &AppState) -> anyhow::Result<String> {
    refresh_all(state).await;
    Ok("gacha pool details refreshed for every configured server".to_owned())
}

pub async fn refresh_all(state: &AppState) {
    let servers = state.service_accounts.servers();
    let mut tasks = tokio::task::JoinSet::new();

    for server in servers {
        let state = state.clone();
        tasks.spawn(async move {
            match refresh(&state, server).await {
                Ok(0) => tracing::debug!(server = server.as_str(), "pool details already current"),
                Ok(n) => tracing::info!(server = server.as_str(), fetched = n, "pool details refreshed"),
                Err(e) => {
                    tracing::warn!(server = server.as_str(), error = %e, "pool detail refresh failed");
                }
            }
        });
    }

    while tasks.join_next().await.is_some() {}
}

/// Fetch the pool details missing for one server, plus any pool still running.
///
/// Returns how many pools were newly fetched. Safe to call directly (a bin, an
/// admin route); it takes no locks beyond the account's own.
pub async fn refresh(state: &AppState, server: Server) -> anyhow::Result<usize> {
    let Some(account) = state.service_accounts.get(server) else {
        return Ok(0);
    };

    let server_data = state.server_data(server);
    let path = pool_detail_path(Path::new(&server_data.assets_dir));

    let mut file = read_sidecar(&path);
    let now = chrono::Utc::now().timestamp();

    // A pool needs fetching if it has never been seen, or if it is still live
    // and could therefore still change. Ended banners are immutable.
    let wanted: Vec<String> = state
        .game_data(server)
        .gacha
        .gacha_pool_client
        .iter()
        .filter(|p| !p.gacha_pool_id.is_empty())
        .filter(|p| p.end_time > now || !file.pools.contains_key(&p.gacha_pool_id))
        .map(|p| p.gacha_pool_id.clone())
        .collect();

    if wanted.is_empty() {
        return Ok(0);
    }

    tracing::info!(
        server = server.as_str(),
        pools = wanted.len(),
        cached = file.pools.len(),
        "fetching gacha pool details"
    );

    let delay = Duration::from_millis(env_or("GACHA_DETAIL_CALL_DELAY_MS", DEFAULT_CALL_DELAY_MS));
    let max_failures: u32 = env_or("GACHA_DETAIL_MAX_FAILURES", DEFAULT_MAX_FAILURES);
    // 0 disables checkpointing, leaving only the final write.
    let checkpoint_every: usize = env_or("GACHA_DETAIL_CHECKPOINT_EVERY", DEFAULT_CHECKPOINT_EVERY);

    let mut fetched = 0usize;
    let mut since_checkpoint = 0usize;
    let mut consecutive_failures = 0u32;

    for pool_id in &wanted {
        match account
            .request(&state.http_client, ENDPOINT, &json!({ "poolId": pool_id }))
            .await
        {
            Ok(value) => match serde_json::from_value::<GachaPoolDetail>(value) {
                Ok(detail) => {
                    file.pools.insert(pool_id.clone(), detail);
                    fetched += 1;
                    since_checkpoint += 1;
                    consecutive_failures = 0;
                }
                Err(e) => {
                    tracing::warn!(pool = %pool_id, error = %e, "pool detail did not deserialize");
                    consecutive_failures += 1;
                }
            },
            // A single rejected pool must not abort a run that is otherwise
            // making progress.
            Err(FetchError::Upstream(err)) => {
                tracing::warn!(pool = %pool_id, error = %err, "pool detail rejected upstream");
                consecutive_failures += 1;
            }
            // Auth or transport failure: the next pool would fail identically.
            Err(e) => {
                tracing::warn!(pool = %pool_id, error = ?e, "pool detail fetch failed, aborting run");
                break;
            }
        }

        if consecutive_failures >= max_failures {
            tracing::warn!(
                server = server.as_str(),
                failures = consecutive_failures,
                "too many consecutive failures, aborting run"
            );
            break;
        }

        if checkpoint_every > 0 && since_checkpoint >= checkpoint_every {
            // A failed checkpoint is not fatal -- the entries are still in
            // memory and the next checkpoint, or the final write, carries them.
            match commit_sidecar(&path, &mut file, server, now) {
                Ok(()) => {
                    tracing::debug!(pools = file.pools.len(), "pool detail checkpoint written");
                }
                Err(e) => tracing::warn!(error = %e, "pool detail checkpoint failed"),
            }
            since_checkpoint = 0;
        }

        tokio::time::sleep(delay).await;
    }

    if fetched == 0 {
        return Ok(0);
    }

    commit_sidecar(&path, &mut file, server, now)?;

    // The loader reads this file, so a reload is what makes the new rate-ups
    // visible on the static endpoints. Deliberately not done per checkpoint:
    // a reload rebuilds every table for the server and is far too heavy to
    // repeat mid-walk.
    asset_watcher::perform_reload(state, server).await;

    Ok(fetched)
}

/// Stamp the run's metadata onto the sidecar and write it.
fn commit_sidecar(
    path: &Path,
    file: &mut PoolDetailFile,
    server: Server,
    now: i64,
) -> anyhow::Result<()> {
    file.version = POOL_DETAIL_FILE_VERSION;
    file.fetched_at = now;
    file.server = server.as_str().to_owned();
    write_sidecar(path, file)
}

/// Read the sidecar, or start a fresh one. A corrupt or version-mismatched file
/// is discarded rather than repaired: refetching is cheap, and a half-parsed
/// cache is worse than none.
fn read_sidecar(path: &Path) -> PoolDetailFile {
    let Ok(bytes) = std::fs::read(path) else {
        return PoolDetailFile::default();
    };
    match serde_json::from_slice::<PoolDetailFile>(&bytes) {
        Ok(file) if file.version == POOL_DETAIL_FILE_VERSION => file,
        Ok(file) => {
            tracing::warn!(
                found = file.version,
                expected = POOL_DETAIL_FILE_VERSION,
                "pool detail sidecar version mismatch, rebuilding"
            );
            PoolDetailFile::default()
        }
        Err(e) => {
            tracing::warn!(error = %e, "pool detail sidecar unreadable, rebuilding");
            PoolDetailFile::default()
        }
    }
}

/// Atomic write (temp + rename) so a crash mid-write cannot leave the loader a
/// truncated file.
fn write_sidecar(path: &Path, file: &PoolDetailFile) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(file)?)?;
    std::fs::rename(&tmp, path)?;
    tracing::debug!(path = %path.display(), pools = file.pools.len(), "wrote pool detail sidecar");
    Ok(())
}
