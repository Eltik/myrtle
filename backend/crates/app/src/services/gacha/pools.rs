use std::path::{Path, PathBuf};
use std::time::Duration;

use myrtle_gamedata_raw::tables::gacha::GachaPoolClient;
use myrtle_gamedata_raw::tables::gacha_detail::{
    GachaPoolDetail, POOL_DETAIL_FILE_VERSION, POOL_DETAIL_REL_PATH, PoolDetailFile,
};
use myrtle_hypergryph::UpstreamError;
use myrtle_types::Server;
use serde_json::json;

use super::sidecar::{self, Fingerprint};
use crate::{AppContext, AppError};

const ENDPOINT: &str = "gacha/getPoolDetail";

#[must_use]
pub fn pool_detail_path(ctx: &AppContext, server: Server) -> PathBuf {
    sidecar::path(
        ctx,
        server,
        &ctx.config.jobs.gacha_detail.dir,
        POOL_DETAIL_REL_PATH,
    )
}

pub async fn refresh_pool_details(
    ctx: &AppContext,
    server: Option<Server>,
) -> Result<usize, AppError> {
    if let Some(server) = server {
        return refresh_server(ctx, server).await;
    }
    let mut fetched = 0;
    for server in sidecar::account_servers(ctx) {
        match refresh_server(ctx, server).await {
            Ok(n) => fetched += n,
            Err(e) => {
                tracing::warn!(server = server.as_str(), error = %e, "pool detail refresh failed");
            }
        }
    }
    Ok(fetched)
}

#[must_use]
pub fn pools_to_fetch(pools: &[GachaPoolClient], file: &PoolDetailFile, now: i64) -> Vec<String> {
    pools
        .iter()
        .filter(|p| !p.gacha_pool_id.is_empty())
        .filter(|p| p.end_time > now || !file.pools.contains_key(&p.gacha_pool_id))
        .map(|p| p.gacha_pool_id.clone())
        .collect()
}

async fn refresh_server(ctx: &AppContext, server: Server) -> Result<usize, AppError> {
    let (Some(account), Some(snapshot)) = (ctx.accounts.get(server), ctx.snapshots.get(server))
    else {
        return Ok(0);
    };
    let path = pool_detail_path(ctx, server);
    let mut file = read_pool_details(&path);
    let now = chrono::Utc::now().timestamp();
    let wanted = pools_to_fetch(&snapshot.model.gacha.gacha_pool_client, &file, now);
    drop(snapshot);
    if wanted.is_empty() {
        return Ok(0);
    }
    tracing::info!(
        server = server.as_str(),
        pools = wanted.len(),
        cached = file.pools.len(),
        "fetching gacha pool details"
    );

    let job = &ctx.config.jobs.gacha_detail;
    let always = ctx.config.switches.sidecar_reload_always.enabled;
    let delay = Duration::from_millis(job.call_delay_ms);
    let checkpoint_every = job.checkpoint_every.unwrap_or(0);
    let loaded = Fingerprint::of(&file.pools);
    let mut on_disk = loaded.clone();
    let (mut fetched, mut since_checkpoint, mut failures) = (0_usize, 0_usize, 0_u32);

    for pool_id in &wanted {
        match account
            .request(&ctx.upstream, ENDPOINT, &json!({ "poolId": pool_id }))
            .await
        {
            Ok(value) => match serde_json::from_value::<GachaPoolDetail>(value) {
                Ok(detail) => {
                    file.pools.insert(pool_id.clone(), detail);
                    fetched += 1;
                    since_checkpoint += 1;
                    failures = 0;
                }
                Err(e) => {
                    tracing::warn!(pool = %pool_id, error = %e, "pool detail did not deserialize");
                    failures += 1;
                }
            },
            Err(UpstreamError::Envelope(e)) => {
                tracing::warn!(pool = %pool_id, error = %e, "pool detail rejected upstream");
                failures += 1;
            }
            Err(e) => {
                tracing::warn!(pool = %pool_id, error = %e, "pool detail fetch failed, aborting run");
                break;
            }
        }
        if failures >= job.max_failures {
            tracing::warn!(
                server = server.as_str(),
                failures,
                "too many consecutive failures, aborting run"
            );
            break;
        }
        if checkpoint_every > 0 && since_checkpoint >= checkpoint_every {
            let current = Fingerprint::of(&file.pools);
            if always || current.differs(&on_disk) {
                match commit_pool_details(&path, &mut file, server, now) {
                    Ok(()) => {
                        on_disk = current;
                        sidecar::mark_pending(server);
                    }
                    Err(e) => tracing::warn!(error = %e, "pool detail checkpoint failed"),
                }
            }
            since_checkpoint = 0;
        }
        tokio::time::sleep(delay).await;
    }

    if fetched == 0 {
        return Ok(0);
    }
    let current = Fingerprint::of(&file.pools);
    if always || current.differs(&on_disk) {
        commit_pool_details(&path, &mut file, server, now)?;
        sidecar::mark_pending(server);
    }
    if always || current.differs(&loaded) || sidecar::is_pending(server) {
        tracing::info!(
            server = server.as_str(),
            fetched,
            "pool details changed, reloading"
        );
        sidecar::reload(ctx, server).await?;
    } else {
        tracing::info!(
            server = server.as_str(),
            fetched,
            "pool details unchanged, reload skipped"
        );
    }
    Ok(fetched)
}

#[must_use]
pub fn read_pool_details(path: &Path) -> PoolDetailFile {
    sidecar::read::<PoolDetailFile>(path)
        .filter(|file| file.version == POOL_DETAIL_FILE_VERSION)
        .unwrap_or_default()
}

pub fn commit_pool_details(
    path: &Path,
    file: &mut PoolDetailFile,
    server: Server,
    now: i64,
) -> Result<(), AppError> {
    file.version = POOL_DETAIL_FILE_VERSION;
    file.fetched_at = now;
    file.server = server.as_str().to_owned();
    let bytes = serde_json::to_vec_pretty(file).map_err(AppError::internal)?;
    sidecar::write(path, &bytes)
}
