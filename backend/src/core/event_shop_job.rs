//! Keeps each server's event-shop sidecar current.
//!
//! The event token shop lives on the game server (see
//! [`super::gamedata::types::event_shop`]). This job walks every server that
//! has a service account, asks `templateShop/getGoodList` for the shop of each
//! activity that names one (`template_shop_id`), has started on that server
//! and is not cached yet (or is still open and could change), writes the
//! answers beside that server's gamedata and triggers a reload.
//!
//! A listed shop the server nevertheless refuses is remembered with the time
//! of the attempt and not asked for again.
//!
//! Calls are serial per account (`seqnum`), servers run concurrently, exactly
//! like [`super::gacha_detail_job`].

use std::{path::Path, time::Duration};

use serde_json::json;

use crate::{
    app::state::AppState,
    core::{
        asset_watcher,
        gamedata::types::event_shop::{
            EVENT_SHOP_FILE_VERSION, EventShopData, EventShopFile, event_shop_path,
        },
        hypergryph::{
            constants::Server,
            fetch::{FetchError, upstream_code},
        },
    },
};

const ENDPOINT: &str = "templateShop/getGoodList";
const DEFAULT_INTERVAL_SECS: u64 = 6 * 60 * 60;
const DEFAULT_CALL_DELAY_MS: u64 = 120;
const DEFAULT_MAX_FAILURES: u32 = 5;

fn env_or<T: std::str::FromStr>(key: &str, fallback: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(fallback)
}

pub fn spawn(state: AppState) {
    if state.service_accounts.is_empty() {
        tracing::info!("event shop job disabled - no service accounts");
        return;
    }
    let interval_secs: u64 = env_or("EVENT_SHOP_REFRESH_SECS", DEFAULT_INTERVAL_SECS);
    tokio::spawn(async move {
        crate::core::jobs::stagger("event_shop").await;
        let mut tick = tokio::time::interval(Duration::from_secs(interval_secs));
        loop {
            tick.tick().await;
            refresh_all(&state).await;
        }
    });
}

pub async fn refresh_once(state: &AppState) -> anyhow::Result<String> {
    refresh_all(state).await;
    Ok("event shops refreshed for every configured server".to_owned())
}

pub async fn refresh_all(state: &AppState) {
    let mut tasks = tokio::task::JoinSet::new();
    for server in state.service_accounts.servers() {
        let state = state.clone();
        tasks.spawn(async move {
            match refresh(&state, server).await {
                Ok(0) => tracing::debug!(server = server.as_str(), "event shops already current"),
                Ok(n) => {
                    tracing::info!(server = server.as_str(), fetched = n, "event shops refreshed");
                }
                Err(e) => {
                    tracing::warn!(server = server.as_str(), error = %e, "event shop refresh failed");
                }
            }
        });
    }
    while tasks.join_next().await.is_some() {}
}

/// Fetch the shops this server has opened and the sidecar does not hold yet,
/// plus any shop still open. Returns how many were newly fetched.
pub async fn refresh(state: &AppState, server: Server) -> anyhow::Result<usize> {
    let Some(account) = state.service_accounts.get(server) else {
        return Ok(0);
    };
    let server_data = state.server_data(server);
    let path = event_shop_path(Path::new(&server_data.assets_dir));
    let mut file = read_sidecar(&path);
    let now = chrono::Utc::now().timestamp();

    let mut wanted: Vec<(String, String, i64)> = state
        .game_data(server)
        .activities
        .values()
        .filter(|a| a.start_time > 0 && a.start_time <= now)
        .filter_map(|a| {
            a.template_shop_id
                .clone()
                .map(|shop| (a.id.clone(), shop, a.end_time))
        })
        .filter(|(id, _, end)| {
            !file.absent.contains_key(id) && (*end > now || !file.shops.contains_key(id))
        })
        .collect();
    wanted.sort_by_key(|(_, _, end)| std::cmp::Reverse(*end));
    if wanted.is_empty() {
        return Ok(0);
    }
    tracing::info!(
        server = server.as_str(),
        shops = wanted.len(),
        cached = file.shops.len(),
        "fetching event shops"
    );

    // The server only answers shop queries for a materialised player; see
    // `ServiceAccount::sync_player`.
    account
        .sync_player(&state.http_client)
        .await
        .map_err(|e| anyhow::anyhow!("account/syncData failed: {e:?}"))?;

    let delay = Duration::from_millis(env_or("EVENT_SHOP_CALL_DELAY_MS", DEFAULT_CALL_DELAY_MS));
    let max_failures: u32 = env_or("EVENT_SHOP_MAX_FAILURES", DEFAULT_MAX_FAILURES);
    let mut fetched = 0usize;
    let mut consecutive_failures = 0u32;

    for (id, shop_id, _) in &wanted {
        match account
            .request_tolerating(
                &state.http_client,
                ENDPOINT,
                &json!({ "shopId": shop_id }),
                &[upstream_code::INVALID_SHOP_ID],
            )
            .await
        {
            Ok(value) => {
                if let Some(shop) = EventShopData::from_response(&value) {
                    file.shops.insert(id.clone(), shop);
                    file.absent.remove(id);
                    fetched += 1;
                    consecutive_failures = 0;
                } else {
                    tracing::warn!(shop = %id, "event shop response had no data");
                    consecutive_failures += 1;
                }
            }
            Err(FetchError::Upstream(err)) => {
                tracing::warn!(shop = %shop_id, error = %err, "listed event shop rejected upstream");
                file.absent.insert(id.clone(), now);
                consecutive_failures = 0;
            }
            Err(e) => {
                tracing::warn!(shop = %id, error = ?e, "event shop fetch failed, aborting run");
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
        tokio::time::sleep(delay).await;
    }

    file.version = EVENT_SHOP_FILE_VERSION;
    file.fetched_at = now;
    file.server = server.as_str().to_owned();
    write_sidecar(&path, &file)?;
    if fetched > 0 {
        asset_watcher::perform_reload(state, server, None).await;
    }
    Ok(fetched)
}

fn read_sidecar(path: &Path) -> EventShopFile {
    let Ok(bytes) = std::fs::read(path) else {
        return EventShopFile::default();
    };
    match serde_json::from_slice::<EventShopFile>(&bytes) {
        Ok(file) if file.version == EVENT_SHOP_FILE_VERSION => file,
        Ok(file) => {
            tracing::warn!(path = %path.display(), version = file.version, "event shop sidecar version mismatch, refetching");
            EventShopFile::default()
        }
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "event shop sidecar unreadable, refetching");
            EventShopFile::default()
        }
    }
}

fn write_sidecar(path: &Path, file: &EventShopFile) -> anyhow::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec(file)?)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
