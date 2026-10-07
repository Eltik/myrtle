use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use tokio_tungstenite::connect_async;

use crate::app::state::AppState;
use crate::core::gacha_resync::reconcile_rarities;
use crate::core::gamedata::init_game_data_with_art;
use crate::core::gamedata::tables::DataError;
use crate::core::gamedata::types::GameData;
use crate::core::hypergryph::constants::Server;
use crate::core::hypergryph::loaders::reload;

const DEBOUNCE_SECS: u64 = 5;
const MAX_BACKOFF: Duration = Duration::from_secs(30);

/// One game data build at a time, process-wide. Each build holds a second
/// `GameData` resident until its swap, and the sidecar jobs run every server
/// concurrently, so unguarded their reloads stack builds. Serialising build and
/// swap together also keeps swaps in build order: an older build can no longer
/// land after a newer one. Bilibili shares CN's cell, another reason the lock is
/// global rather than per server.
static RELOAD_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Take the process-wide reload lock, logging when another build holds it.
/// Every `GameData` build (hot reload, sidecar patch, lazy load) and every lazy
/// unload goes through this.
pub(crate) async fn lock_reloads(server: Server) -> tokio::sync::MutexGuard<'static, ()> {
    if let Ok(guard) = RELOAD_LOCK.try_lock() {
        guard
    } else {
        tracing::info!(
            server = server.as_str(),
            "another game data reload is running, waiting for it"
        );
        RELOAD_LOCK.lock().await
    }
}

/// Build one server's `GameData` and asset index from disk, aligning its
/// reference facts against `references` (empty for the default server).
/// Synchronous and CPU-bound: call it from the blocking pool.
pub(crate) fn build_server(
    data_dir: &str,
    assets_dir: &str,
    art_dir: &str,
    references: &[std::sync::Arc<GameData>],
) -> Result<(GameData, crate::core::gamedata::assets::AssetIndex), DataError> {
    let (mut game_data, asset_index) = init_game_data_with_art(
        Path::new(data_dir),
        Path::new(assets_dir),
        Path::new(art_dir),
    )?;
    if !references.is_empty() {
        let references: Vec<&GameData> = references.iter().map(|r| &**r).collect();
        crate::core::gamedata::enrich::reference::align_reference_facts(
            &mut game_data,
            &references,
        );
    }
    Ok((game_data, asset_index))
}

/// Spawn one hot-reload watcher per configured server WebSocket.
///
/// Each server's asset pipeline runs its own WS (typically a distinct port), so
/// an `update_complete` reloads only that server's [`ServerData`]. Bilibili has
/// no watcher of its own; it shares CN's cell and reloads when CN does.
///
/// [`ServerData`]: crate::app::state::ServerData
pub fn spawn(state: AppState) {
    if state.config.asset_ws_urls.is_empty() {
        tracing::info!("no asset WebSocket URLs configured, asset hot-reload disabled");
        return;
    }

    for (&server, url) in &state.config.asset_ws_urls {
        let url = url.clone();
        let state = state.clone();
        tokio::spawn(async move {
            connection_loop(&url, server, &state).await;
        });
    }
}

async fn connection_loop(url: &str, server: Server, state: &AppState) {
    let mut backoff = Duration::from_secs(1);
    let srv = server.as_str();

    loop {
        tracing::info!(server = srv, url, "connecting to asset pipeline WebSocket");

        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                tracing::info!(server = srv, "connected to asset pipeline WebSocket");
                backoff = Duration::from_secs(1);
                handle_connection(ws_stream, server, state).await;
                tracing::warn!(server = srv, "asset pipeline WebSocket disconnected");
            }
            Err(e) => {
                tracing::warn!(
                    server = srv,
                    error = %e,
                    retry_in = ?backoff,
                    "failed to connect to asset pipeline WebSocket"
                );
            }
        }

        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }
}

async fn handle_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    server: Server,
    state: &AppState,
) {
    let (_, mut read) = ws_stream.split();
    let mut last_reload = Instant::now() - Duration::from_secs(DEBOUNCE_SECS + 1);

    while let Some(msg) = read.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(server = server.as_str(), error = %e, "WebSocket read error");
                return;
            }
        };

        let text = match msg {
            tokio_tungstenite::tungstenite::Message::Text(t) => t,
            tokio_tungstenite::tungstenite::Message::Close(_) => return,
            _ => continue,
        };

        let parsed: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let msg_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("");

        match msg_type {
            "update_complete" => {
                let version = parsed
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                if last_reload.elapsed() < Duration::from_secs(DEBOUNCE_SECS) {
                    tracing::debug!(
                        server = server.as_str(),
                        version,
                        "skipping reload (debounced)"
                    );
                    continue;
                }

                tracing::info!(
                    server = server.as_str(),
                    version,
                    "asset update complete, reloading game data"
                );
                perform_reload(state, server, Some(version)).await;
                last_reload = Instant::now();
            }
            "error" => {
                let message = parsed
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                tracing::warn!(
                    server = server.as_str(),
                    message,
                    "asset pipeline reported error"
                );
            }
            "status" | "download_progress" | "download_complete" | "unpack_progress"
            | "update_available" => {
                tracing::debug!(server = server.as_str(), msg_type, "asset pipeline event");
            }
            _ => {
                tracing::debug!(
                    server = server.as_str(),
                    msg_type,
                    "unknown asset pipeline message"
                );
            }
        }
    }
}

/// `res_version` is the asset pipeline's version string when the reload was
/// triggered by an `update_complete` message, and `None` for reloads with no
/// version to record (the pool-detail job). The release ledger keys the
/// forward-only operator/banner debut link on it.
pub(crate) async fn perform_reload(state: &AppState, server: Server, res_version: Option<&str>) {
    let Some(sd) = state.entry(server) else {
        tracing::warn!(
            server = server.as_str(),
            "reload for an unconfigured server ignored"
        );
        return;
    };
    // Taken before the residency check: a lazy load holds this lock while it
    // reads the files, so an update landing mid-load waits for it and then
    // finds the server resident and rebuilds, instead of being skipped while
    // the load publishes the files as they were.
    let reload_guard = lock_reloads(server).await;

    // A lazy server that is not resident has nothing to rebuild: its next
    // request loads the files as they are now. Building it here would only
    // make it resident until the idle reaper drops it again. What the rebuild
    // would have done besides still happens: its cached responses go (they
    // are kept 24 h and would otherwise list the old operators), and the
    // update's version waits for the load's release-ledger record.
    if sd.residency.lazy && !sd.loaded.load(Ordering::Acquire) {
        if let Some(v) = res_version {
            sd.residency.defer_res_version(v);
        }
        drop(reload_guard);
        state
            .cache
            .invalidate_by_prefix(&format!("static:{}:", server.as_str()))
            .await;
        tracing::info!(
            server = server.as_str(),
            "lazy server not resident; rebuild skipped, cache cleared, the next request loads the new data"
        );
        return;
    }
    let data_dir = sd.game_data_dir.clone();
    let assets_dir = sd.assets_dir.clone();
    let art_dir = sd.art_dir.clone();
    let http_client = state.http_client.clone();
    let is_default = server == state.default_server;
    // Taken before the build reads the files, so a sidecar written while it
    // runs stays pending for the next job run; re-marked below on failure.
    crate::core::sidecar::take_pending(server);

    // A non-default server reads the facts its own wording does not carry from
    // the other servers; see `enrich::reference::align_reference_facts`.
    let references = if is_default {
        Vec::new()
    } else {
        state.reference_servers(server)
    };
    let result = tokio::task::spawn_blocking(move || {
        build_server(&data_dir, &assets_dir, &art_dir, &references)
    })
    .await;

    match result {
        Ok(Ok((game_data, asset_index))) => {
            let op_count = game_data.operators.len();
            state.swap_game_data(server, game_data);
            state.swap_asset_index(server, asset_index);
            // Everything after the swap reads the new data and may wait on the
            // network; none of it needs the lock.
            drop(reload_guard);
            let warnings = sd.game_data.load_full().table_warnings.clone();
            crate::core::alerts::report_degraded_tables(
                &state.http_client,
                server.as_str(),
                &warnings,
            )
            .await;

            let was_loaded = sd.loaded.swap(true, Ordering::Release);

            crate::core::release::ledger::spawn_record(
                state.clone(),
                server,
                res_version.map(str::to_owned),
            );

            // The story index is keyed on the `GameData` allocation, so the
            // swap above just invalidated it. Rebuild it now rather than
            // leaving the cost to the next reader.
            crate::app::services::story::spawn_warm(state.clone(), server);

            let prefix = if is_default {
                "static:".to_string()
            } else {
                format!("static:{}:", server.as_str())
            };
            state.cache.invalidate_by_prefix(&prefix).await;

            if was_loaded {
                tracing::info!(
                    server = server.as_str(),
                    operators = op_count,
                    "hot-reload complete"
                );
            } else {
                tracing::info!(
                    server = server.as_str(),
                    operators = op_count,
                    "hot-reload complete, server now loaded"
                );
            }

            if is_default {
                // The whole `dps:` subtree, not just the list: memoised
                // simulations are keyed on the request body alone, so a game data
                // reload is the only thing that can make one wrong.
                state.cache.invalidate_by_prefix("dps:").await;
                // Improvements is built against `default_game_data()`, so a
                // default-server reload can change which events are open and which
                // medals are still earnable under an unchanged sync generation.
                state.cache.invalidate_by_prefix("improvements:").await;
                reload(&http_client).await;

                let state = state.clone();
                tokio::spawn(async move {
                    let gd = state.default_game_data();
                    match reconcile_rarities(&state.db, &gd).await {
                        Ok(stats) if stats.rows_updated > 0 => {
                            tracing::info!(
                                rows = stats.rows_updated,
                                char_ids = stats.fixed_char_ids,
                                distinct_pairs = stats.distinct_pairs,
                                "gacha resync complete",
                            );
                            state.cache.invalidate_by_prefix("gacha:").await;
                        }
                        Ok(stats) => {
                            tracing::debug!(
                                distinct_pairs = stats.distinct_pairs,
                                "gacha resync: no changes",
                            );
                        }
                        Err(e) => tracing::warn!(error = %e, "gacha resync failed"),
                    }
                });
            }
        }
        Ok(Err(e)) => {
            crate::core::sidecar::mark_pending(server);
            tracing::error!(
                server = server.as_str(),
                error = %e,
                "hot-reload failed: game data parse error, keeping old data"
            );
        }
        Err(e) => {
            crate::core::sidecar::mark_pending(server);
            tracing::error!(
                server = server.as_str(),
                error = %e,
                "hot-reload task panicked, keeping old data"
            );
        }
    }
}

/// Publish a sidecar refresh (pool details, event shops) without rebuilding
/// the server's tables.
///
/// The pool-detail and event-shop jobs used to call [`perform_reload`], which
/// rebuilds every table and holds that build next to the live `GameData` until
/// the swap: two full copies resident at the peak, four times a day per
/// server, for a change confined to `gacha` and `event_shops`. This rebuilds
/// only those two (`gamedata::load_sidecar_parts`, the same code the full load
/// runs for them) and publishes a `GameData` that SHARES the live tables
/// (`GameData::with_sidecars`). Same reload lock, same pending-sidecar
/// bookkeeping, same cache invalidation and follow-ups as a full reload, minus
/// the two that depend only on the tables (the DPS memo and the rarity resync).
///
/// Falls back to [`perform_reload`] for a server that is not loaded (the
/// retry path for a failed load), and entirely with `SIDECAR_PATCH=0`, which
/// restores the old full reload exactly.
///
/// Not refreshed by a patch: `table_warnings` (it lives in the shared tables);
/// a sidecar warning is logged and alerted instead.
pub(crate) async fn perform_sidecar_patch(state: &AppState, server: Server) {
    if crate::utils::env::switched_off("SIDECAR_PATCH") {
        perform_reload(state, server, None).await;
        return;
    }
    let Some(sd) = state.entry(server) else {
        tracing::warn!(
            server = server.as_str(),
            "sidecar patch for an unconfigured server ignored"
        );
        return;
    };
    if !sd.loaded.load(Ordering::Acquire) {
        perform_reload(state, server, None).await;
        return;
    }
    let is_default = server == state.default_server;
    let data_dir = sd.game_data_dir.clone();
    let assets_dir = sd.assets_dir.clone();

    let guard = lock_reloads(server).await;
    crate::core::sidecar::take_pending(server);
    let started = Instant::now();
    let parts = tokio::task::spawn_blocking(move || {
        crate::core::gamedata::load_sidecar_parts(Path::new(&data_dir), Path::new(&assets_dir))
    })
    .await;
    let parts = match parts {
        Ok(parts) => parts,
        Err(e) => {
            crate::core::sidecar::mark_pending(server);
            tracing::error!(server = server.as_str(), error = %e, "sidecar patch task panicked, keeping old data");
            return;
        }
    };
    let live = sd.game_data.load_full();
    let patched = live.with_sidecars(parts.gacha, parts.event_shops);
    let banners = patched.gacha.gacha_pool_client.len();
    let shops = patched.event_shops.len();
    sd.game_data.store(std::sync::Arc::new(patched));
    drop(live);
    drop(guard);

    tracing::info!(
        server = server.as_str(),
        banners,
        shops,
        elapsed_ms = started.elapsed().as_millis() as u64,
        "sidecar patch published (tables shared, not rebuilt)"
    );
    for w in &parts.warnings {
        tracing::warn!(server = server.as_str(), "{w}");
    }
    crate::core::alerts::report_degraded_tables(
        &state.http_client,
        server.as_str(),
        &parts.warnings,
    )
    .await;

    crate::core::release::ledger::spawn_record(state.clone(), server, None);
    // The story index is keyed on the `GameData` allocation, which the patch
    // replaced; warm it as a full reload does.
    crate::app::services::story::spawn_warm(state.clone(), server);
    let prefix = if is_default {
        "static:".to_string()
    } else {
        format!("static:{}:", server.as_str())
    };
    state.cache.invalidate_by_prefix(&prefix).await;
    if is_default {
        // Event shops feed the improvements pipeline.
        state.cache.invalidate_by_prefix("improvements:").await;
        reload(&state.http_client).await;
    }
}
