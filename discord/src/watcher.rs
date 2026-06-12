//! WebSocket watcher for the Arknights asset pipeline (`assets/run.mjs ws`).
//!
//! Connects to the pipeline's WS endpoint, fans selected events out as embeds to every guild that
//! has bound an announcement channel via `/assets channel set`, and exposes a shared
//! `AssetsState` so slash commands can read the last-known status or send commands back
//! (`list_resources`).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serenity::all::{Channel, ChannelId, ChannelType, Http, Timestamp};
use serenity::builder::{CreateEmbed, CreateMessage};
use sqlx::SqlitePool;
use tokio::sync::{Mutex, RwLock, mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::db;

#[allow(clippy::unreadable_literal)]
const COLOR_OK: u32 = 0x57F287;
#[allow(clippy::unreadable_literal)]
const COLOR_WARN: u32 = 0xFEE75C;
#[allow(clippy::unreadable_literal)]
const COLOR_BAD: u32 = 0xED4245;
#[allow(clippy::unreadable_literal)]
const COLOR_INFO: u32 = 0x5865F2;

/// Shared state owned by both the watcher task and the slash-command handlers.
///
/// - `status` reflects the most recent `status/update_complete` event so `/assets status`
///   can answer without a fresh round-trip.
/// - `pending_resource_list` holds a one-shot reply channel that `/assets resources`
///   installs before sending `list_resources`; the next inbound `resource_list` frame fulfills it.
/// - `tx` is the outbound queue; everything written here is forwarded to the live WS connection
///   (and buffered while disconnected).
pub struct AssetsState {
    pub status: RwLock<AssetStatus>,
    pub pending_resource_list: Mutex<Option<oneshot::Sender<ResourceListPayload>>>,
    pub tx: mpsc::UnboundedSender<String>,
}

/// One [`AssetsState`] per watched server, keyed by label (e.g. "EN", "CN").
/// Shared between the per-server watcher tasks and the slash-command handlers.
pub type AssetsStates = Arc<HashMap<String, Arc<AssetsState>>>;

#[derive(Debug, Clone, Default)]
pub struct AssetStatus {
    pub state: Option<String>,
    pub current_version: Option<String>,
    /// Version being downloaded/unpacked in the current cycle, set by `UpdateAvailable`
    /// and cleared by `UpdateComplete`. `Status` events don't carry it inline.
    pub pending_new_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct VersionInfo {
    #[serde(default)]
    current: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResourceFile {
    pub name: String,
    pub size: u64,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResourceListPayload {
    pub files: Vec<ResourceFile>,
    #[serde(rename = "totalSize", default)]
    pub total_size: u64,
    #[serde(rename = "totalSizeFormatted", default)]
    pub total_size_formatted: Option<String>,
}

/// Mirrors the `type`-tagged JSON broadcast by `assets/run.mjs`. Variants the bot doesn't
/// care about (e.g. progress ticks) fall into `Unknown` so deserialization can't fail on them.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AssetEvent {
    Status {
        state: String,
        #[serde(default)]
        version: VersionInfo,
    },
    UpdateAvailable {
        #[serde(rename = "currentVersion", default)]
        current_version: Option<String>,
        #[serde(rename = "newVersion")]
        new_version: String,
        #[serde(rename = "clientVersion", default)]
        client_version: Option<String>,
    },
    DownloadComplete {
        downloaded: u64,
        failed: u64,
        #[serde(rename = "totalBytes", default)]
        total_bytes: u64,
        #[serde(rename = "totalBytesFormatted", default)]
        total_bytes_formatted: Option<String>,
    },
    PruneComplete {
        deleted: u64,
        #[serde(rename = "freedBytes", default)]
        freed_bytes: u64,
    },
    UpdateComplete {
        version: String,
        downloaded: u64,
        failed: u64,
        exported: u64,
    },
    ResourceList {
        #[serde(flatten)]
        payload: ResourceListPayload,
    },
    Error {
        message: String,
    },
    #[serde(other)]
    Unknown,
}

/// Entry point: reconnect-forever loop. Returns only if `ws_url` is empty.
pub async fn run(
    http: Arc<Http>,
    pool: SqlitePool,
    label: String,
    ws_url: String,
    reconnect_secs: u64,
    state: Arc<AssetsState>,
    mut rx: mpsc::UnboundedReceiver<String>,
) {
    if ws_url.is_empty() {
        tracing::info!("assets watcher [{label}]: ws_url is empty, watcher disabled");
        return;
    }
    let reconnect = Duration::from_secs(reconnect_secs.max(1));
    loop {
        match connect_async(&ws_url).await {
            Ok((ws, _)) => {
                tracing::info!("assets watcher [{label}]: connected to {ws_url}");
                if let Err(e) = pump(ws, &http, &pool, &state, &label, &mut rx).await {
                    tracing::warn!("assets watcher [{label}]: connection ended: {e}");
                } else {
                    tracing::info!("assets watcher [{label}]: peer closed connection");
                }
            }
            Err(e) => {
                tracing::warn!(
                    "assets watcher [{label}]: connect to {ws_url} failed ({e}); retry in {reconnect:?}"
                );
            }
        }
        // Reset the cached state on disconnect so `/assets status` reports `unknown` rather
        // than a stale snapshot.
        *state.status.write().await = AssetStatus::default();
        tokio::time::sleep(reconnect).await;
    }
}

async fn pump<S>(
    ws: tokio_tungstenite::WebSocketStream<S>,
    http: &Arc<Http>,
    pool: &SqlitePool,
    state: &Arc<AssetsState>,
    label: &str,
    rx: &mut mpsc::UnboundedReceiver<String>,
) -> Result<(), tokio_tungstenite::tungstenite::Error>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let (mut sink, mut stream) = ws.split();
    loop {
        tokio::select! {
            frame = stream.next() => {
                let Some(msg) = frame else { return Ok(()); };
                match msg? {
                    Message::Text(t) => handle_event(t.as_str(), http, pool, state, label).await,
                    Message::Ping(p) => { sink.send(Message::Pong(p)).await?; }
                    Message::Close(reason) => {
                        let echo = reason;
                        let _ = sink.send(Message::Close(echo)).await;
                        return Ok(());
                    }
                    _ => {}
                }
            }
            cmd = rx.recv() => {
                let Some(cmd) = cmd else { return Ok(()); };
                sink.send(Message::text(cmd)).await?;
            }
        }
    }
}

async fn handle_event(
    text: &str,
    http: &Arc<Http>,
    pool: &SqlitePool,
    state: &Arc<AssetsState>,
    label: &str,
) {
    let event: AssetEvent = match serde_json::from_str(text) {
        Ok(e) => e,
        Err(e) => {
            tracing::debug!("assets watcher: undecodable event: {e}: {text}");
            return;
        }
    };

    update_status_cache(state, &event).await;

    if let AssetEvent::ResourceList { payload } = &event
        && let Some(tx) = state.pending_resource_list.lock().await.take()
    {
        let _ = tx.send(payload.clone());
    }

    let pending_new = state.status.read().await.pending_new_version.clone();

    let Some(embed) = build_embed(&event, pending_new.as_deref()) else {
        return;
    };
    // Tag every announcement with the server it came from. The bot opens one WS
    // per server; the pipeline events themselves carry no server identity.
    let embed = embed.field("Server", label, true);

    let channels = match db::list_assets_channels(pool).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("assets watcher: list channels: {e}");
            return;
        }
    };

    for (_guild, channel) in channels {
        broadcast_one(http, channel, embed.clone()).await;
    }
}

async fn update_status_cache(state: &Arc<AssetsState>, event: &AssetEvent) {
    match event {
        AssetEvent::Status { state: s, version } => {
            let mut st = state.status.write().await;
            st.state = Some(s.clone());
            if version.current.is_some() {
                st.current_version.clone_from(&version.current);
            }
        }
        AssetEvent::UpdateAvailable { new_version, .. } => {
            state.status.write().await.pending_new_version = Some(new_version.clone());
        }
        AssetEvent::UpdateComplete { version, .. } => {
            let mut st = state.status.write().await;
            st.state = Some("idle".to_string());
            st.current_version = Some(version.clone());
            st.pending_new_version = None;
        }
        _ => {}
    }
}

fn build_embed(event: &AssetEvent, pending_new_version: Option<&str>) -> Option<CreateEmbed> {
    match event {
        // Phase-start announcements. Only the transitions matter (one embed per phase per
        // cycle); the per-file progress ticks stay in `Unknown` so the channel doesn't flood.
        AssetEvent::Status { state, .. } if state == "downloading" => {
            Some(phase_embed("Downloading assets…", pending_new_version))
        }
        AssetEvent::Status { state, .. } if state == "unpacking" => {
            Some(phase_embed("Unpacking assets…", pending_new_version))
        }
        AssetEvent::UpdateAvailable {
            current_version,
            new_version,
            client_version,
        } => {
            let mut embed = CreateEmbed::new()
                .title("Asset update available")
                .colour(COLOR_WARN)
                .field(
                    "Current",
                    current_version.as_deref().unwrap_or("(none)"),
                    true,
                )
                .field("New", new_version.as_str(), true)
                .timestamp(Timestamp::now());
            if let Some(c) = client_version.as_deref() {
                embed = embed.field("Client", c, true);
            }
            Some(embed)
        }
        AssetEvent::UpdateComplete {
            version,
            downloaded,
            failed,
            exported,
        } => Some(
            CreateEmbed::new()
                .title("Asset update complete")
                .colour(COLOR_OK)
                .field("Version", version.as_str(), true)
                .field("Downloaded", downloaded.to_string(), true)
                .field("Failed", failed.to_string(), true)
                .field("Exported", exported.to_string(), true)
                .timestamp(Timestamp::now()),
        ),
        AssetEvent::DownloadComplete {
            downloaded,
            failed,
            total_bytes,
            total_bytes_formatted,
        } => {
            // Prefer the pipeline's preformatted size — its rounding matches the CLI output.
            let size = total_bytes_formatted
                .clone()
                .unwrap_or_else(|| format_bytes(*total_bytes));
            let mut embed = CreateEmbed::new()
                .title("Download complete")
                .colour(COLOR_OK)
                .field("Downloaded", downloaded.to_string(), true)
                .field("Failed", failed.to_string(), true)
                .field("Size", size, true)
                .timestamp(Timestamp::now());
            if let Some(v) = pending_new_version {
                embed = embed.field("Version", v, true);
            }
            Some(embed)
        }
        AssetEvent::PruneComplete {
            deleted,
            freed_bytes,
        } if *deleted > 0 => Some(
            CreateEmbed::new()
                .title("Orphan bundles pruned")
                .colour(COLOR_INFO)
                .field("Files", deleted.to_string(), true)
                .field("Freed", format_bytes(*freed_bytes), true)
                .timestamp(Timestamp::now()),
        ),
        AssetEvent::Error { message } => Some(
            CreateEmbed::new()
                .title("Asset pipeline error")
                .colour(COLOR_BAD)
                .description(message)
                .timestamp(Timestamp::now()),
        ),
        _ => None,
    }
}

fn phase_embed(title: &str, version: Option<&str>) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title(title)
        .colour(COLOR_INFO)
        .timestamp(Timestamp::now());
    if let Some(v) = version {
        embed = embed.field("Version", v, true);
    }
    embed
}

async fn broadcast_one(http: &Arc<Http>, channel: ChannelId, embed: CreateEmbed) {
    let msg = CreateMessage::new().embed(embed);
    let sent = match channel.send_message(http.as_ref(), msg).await {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!("assets watcher: send to {channel}: {e}");
            return;
        }
    };

    if is_announcement_channel(http, channel).await
        && let Err(e) = sent.crosspost(http.as_ref()).await
    {
        tracing::warn!(
            "assets watcher: crosspost message {} in {channel}: {e}",
            sent.id
        );
    }
}

async fn is_announcement_channel(http: &Arc<Http>, channel: ChannelId) -> bool {
    match channel.to_channel(http.as_ref()).await {
        Ok(Channel::Guild(c)) => c.kind == ChannelType::News,
        Ok(_) => false,
        Err(e) => {
            tracing::warn!("assets watcher: fetch channel {channel} for crosspost check: {e}");
            false
        }
    }
}

#[allow(clippy::cast_precision_loss)]
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;
    let f = bytes as f64;
    if bytes < KB {
        format!("{bytes} B")
    } else if bytes < MB {
        format!("{:.1} KB", f / KB as f64)
    } else if bytes < GB {
        format!("{:.1} MB", f / MB as f64)
    } else {
        format!("{:.2} GB", f / GB as f64)
    }
}
