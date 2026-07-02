use std::path::Path;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use tokio_tungstenite::connect_async;

use crate::app::state::AppState;
use crate::core::gacha_resync::reconcile_rarities;
use crate::core::gamedata::assets::AssetIndex;
use crate::core::gamedata::init_game_data;
use crate::core::gamedata::tables::DataError;
use crate::core::hypergryph::constants::Server;
use crate::core::hypergryph::loaders::reload;

const DEBOUNCE_SECS: u64 = 5;
const MAX_BACKOFF: Duration = Duration::from_secs(30);

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
                perform_reload(state, server).await;
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

async fn perform_reload(state: &AppState, server: Server) {
    let sd = state.server_data(server);
    let data_dir = sd.game_data_dir.clone();
    let assets_dir = sd.assets_dir.clone();
    let http_client = state.http_client.clone();
    let is_default = server == state.default_server;

    let result = tokio::task::spawn_blocking(move || {
        let game_data = init_game_data(Path::new(&data_dir), Path::new(&assets_dir))?;
        let asset_index = AssetIndex::build(Path::new(&assets_dir));
        Ok::<_, DataError>((game_data, asset_index))
    })
    .await;

    match result {
        Ok(Ok((game_data, asset_index))) => {
            let op_count = game_data.operators.len();
            state.swap_game_data(server, game_data);
            state.swap_asset_index(server, asset_index);

            let prefix = if is_default {
                "static:".to_string()
            } else {
                format!("static:{}:", server.as_str())
            };
            state.cache.invalidate_by_prefix(&prefix).await;

            tracing::info!(
                server = server.as_str(),
                operators = op_count,
                "hot-reload complete"
            );

            if is_default {
                state.cache.invalidate_by_prefix("dps:list:").await;
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
            tracing::error!(
                server = server.as_str(),
                error = %e,
                "hot-reload failed: game data parse error, keeping old data"
            );
        }
        Err(e) => {
            tracing::error!(
                server = server.as_str(),
                error = %e,
                "hot-reload task panicked, keeping old data"
            );
        }
    }
}
