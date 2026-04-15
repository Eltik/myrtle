use std::path::Path;

use std::time::{Duration, Instant};

use futures_util::StreamExt;
use tokio_tungstenite::connect_async;

use crate::app::state::AppState;
use crate::core::gamedata::assets::AssetIndex;

const DEBOUNCE_SECS: u64 = 5;
const MAX_BACKOFF: Duration = Duration::from_secs(30);

pub fn spawn(state: AppState) {
    let url = match &state.config.asset_ws_url {
        Some(url) => url.clone(),
        None => {
            tracing::info!("ASSET_WS_URL not set, asset hot-reload disabled");
            return;
        }
    };

    tokio::spawn(async move {
        connection_loop(&url, &state).await;
    });
}

async fn connection_loop(url: &str, state: &AppState) {
    let mut backoff = Duration::from_secs(1);

    loop {
        tracing::info!(url, "connecting to asset pipeline WebSocket");

        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                tracing::info!(url, "connected to asset pipeline WebSocket");
                backoff = Duration::from_secs(1);
                handle_connection(ws_stream, state).await;
                tracing::warn!("asset pipeline WebSocket disconnected");
            }
            Err(e) => {
                tracing::warn!(
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
    state: &AppState,
) {
    let (_, mut read) = ws_stream.split();
    let mut last_reload = Instant::now() - Duration::from_secs(DEBOUNCE_SECS + 1);

    while let Some(msg) = read.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(error = %e, "WebSocket read error");
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
                    tracing::debug!(version, "skipping reload (debounced)");
                    continue;
                }

                tracing::info!(version, "asset update complete, reloading game data");
                perform_reload(state).await;
                last_reload = Instant::now();
            }
            "error" => {
                let message = parsed
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                tracing::warn!(message, "asset pipeline reported error");
            }
            "status" | "download_progress" | "download_complete" | "unpack_progress"
            | "update_available" => {
                tracing::debug!(msg_type, "asset pipeline event");
            }
            _ => {
                tracing::debug!(msg_type, "unknown asset pipeline message");
            }
        }
    }
}

async fn perform_reload(state: &AppState) {
    let data_dir = state.config.game_data_dir.clone();
    let assets_dir = state.config.assets_dir.clone();
    let http_client = state.http_client.clone();

    let result = tokio::task::spawn_blocking(move || {
        let game_data =
            crate::core::gamedata::init_game_data(Path::new(&data_dir), Path::new(&assets_dir))?;
        let asset_index = AssetIndex::build(Path::new(&assets_dir));
        Ok::<_, crate::core::gamedata::tables::DataError>((game_data, asset_index))
    })
    .await;

    match result {
        Ok(Ok((game_data, asset_index))) => {
            let op_count = game_data.operators.len();
            state.swap_game_data(game_data);
            state.swap_asset_index(asset_index);

            // Reload version/network configs from Hypergryph servers
            crate::core::hypergryph::loaders::reload(&http_client).await;

            // Flush cached static data so next request rebuilds from new game data
            state.cache.invalidate_by_prefix("static:").await;

            tracing::info!(operators = op_count, "hot-reload complete");
        }
        Ok(Err(e)) => {
            tracing::error!(error = %e, "hot-reload failed: game data parse error, keeping old data");
        }
        Err(e) => {
            tracing::error!(error = %e, "hot-reload task panicked, keeping old data");
        }
    }
}
