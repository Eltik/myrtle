use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use myrtle_gamedata_raw::tables::activity::ActivityBasicInfo;
use myrtle_gamedata_raw::tables::event_shop::{
    EVENT_SHOP_FILE_VERSION, EVENT_SHOP_REL_PATH, EventShopData, EventShopFile,
};
use myrtle_hypergryph::{UpstreamError, envelope_code};
use myrtle_types::Server;
use serde_json::json;

use super::sidecar::{self, Fingerprint};
use crate::{AppContext, AppError};

const ENDPOINT: &str = "templateShop/getGoodList";

#[must_use]
pub fn event_shop_path(ctx: &AppContext, server: Server) -> PathBuf {
    sidecar::path(
        ctx,
        server,
        &ctx.config.jobs.event_shop.dir,
        EVENT_SHOP_REL_PATH,
    )
}

/// The `EventShopRefresh` job: one server, or every server with a service account
pub async fn refresh_event_shops(
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
                tracing::warn!(server = server.as_str(), error = %e, "event shop refresh failed");
            }
        }
    }
    Ok(fetched)
}

/// (activity id, shop id) of started activities whose shop is unheld or open and never refused, latest end first
#[must_use]
pub fn shops_to_fetch(
    activities: &HashMap<String, ActivityBasicInfo>,
    file: &EventShopFile,
    now: i64,
) -> Vec<(String, String)> {
    let mut wanted: Vec<(&ActivityBasicInfo, &String)> = activities
        .values()
        .filter(|a| a.start_time > 0 && a.start_time <= now)
        .filter_map(|a| a.template_shop_id.as_ref().map(|shop| (a, shop)))
        .filter(|(a, _)| {
            !file.absent.contains_key(&a.id)
                && (a.end_time > now || !file.shops.contains_key(&a.id))
        })
        .collect();
    wanted.sort_by_key(|(a, _)| std::cmp::Reverse(a.end_time));
    wanted
        .into_iter()
        .map(|(a, shop)| (a.id.clone(), shop.clone()))
        .collect()
}

async fn refresh_server(ctx: &AppContext, server: Server) -> Result<usize, AppError> {
    let (Some(account), Some(snapshot)) = (ctx.accounts.get(server), ctx.snapshots.get(server))
    else {
        return Ok(0);
    };
    let path = event_shop_path(ctx, server);
    let mut file = read_event_shops(&path);
    let now = chrono::Utc::now().timestamp();
    let wanted = shops_to_fetch(&snapshot.model.activities, &file, now);
    drop(snapshot);
    if wanted.is_empty() {
        return Ok(0);
    }
    tracing::info!(
        server = server.as_str(),
        shops = wanted.len(),
        cached = file.shops.len(),
        "fetching event shops"
    );
    let loaded_shops = Fingerprint::of(&file.shops); // only `shops` decides the reload; refusals are just persisted
    let on_disk = stored_data(&file);

    account.sync_player(&ctx.upstream).await?; // shop queries answer only for a materialised player

    let job = &ctx.config.jobs.event_shop;
    let delay = Duration::from_millis(job.call_delay_ms);
    let (mut fetched, mut failures) = (0_usize, 0_u32);
    for (id, shop_id) in &wanted {
        match account
            .request_tolerating(
                &ctx.upstream,
                ENDPOINT,
                &json!({ "shopId": shop_id }),
                &[envelope_code::INVALID_SHOP_ID],
            )
            .await
        {
            Ok(value) => {
                if let Some(shop) = EventShopData::from_response(&value) {
                    file.shops.insert(id.clone(), shop);
                    file.absent.remove(id);
                    fetched += 1;
                    failures = 0;
                } else {
                    tracing::warn!(shop = %id, "event shop response had no data");
                    failures += 1;
                }
            }
            Err(UpstreamError::Envelope(e)) => {
                tracing::warn!(shop = %shop_id, error = %e, "listed event shop rejected upstream");
                file.absent.insert(id.clone(), now);
                failures = 0;
            }
            Err(e) => {
                tracing::warn!(shop = %id, error = %e, "event shop fetch failed, aborting run");
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
        tokio::time::sleep(delay).await;
    }

    let always = ctx.config.switches.sidecar_reload_always.enabled;
    if always || stored_data(&file).differs(&on_disk) {
        commit_event_shops(&path, &mut file, server, now)?;
    }
    let reload = if always {
        fetched > 0
    } else {
        Fingerprint::of(&file.shops).differs(&loaded_shops) || sidecar::is_pending(server)
    };
    if reload {
        sidecar::mark_pending(server);
        tracing::info!(
            server = server.as_str(),
            fetched,
            "event shops changed, reloading"
        );
        sidecar::reload(ctx, server).await?;
    } else if fetched > 0 {
        tracing::info!(
            server = server.as_str(),
            fetched,
            "event shops unchanged, reload skipped"
        );
    }
    Ok(fetched)
}

/// Everything the sidecar persists but the run metadata
#[must_use]
pub fn stored_data(file: &EventShopFile) -> Fingerprint {
    Fingerprint::of(&(&file.shops, &file.absent))
}

/// The sidecar, or an empty one when absent, unreadable or another version
pub fn read_event_shops(path: &Path) -> EventShopFile {
    sidecar::read::<EventShopFile>(path)
        .filter(|file| file.version == EVENT_SHOP_FILE_VERSION)
        .unwrap_or_default()
}

pub fn commit_event_shops(
    path: &Path,
    file: &mut EventShopFile,
    server: Server,
    now: i64,
) -> Result<(), AppError> {
    file.version = EVENT_SHOP_FILE_VERSION;
    file.fetched_at = now;
    file.server = server.as_str().to_owned();
    let bytes = serde_json::to_vec(file).map_err(AppError::internal)?;
    sidecar::write(path, &bytes)
}
