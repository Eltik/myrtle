use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use futures_util::{Stream, StreamExt};
use myrtle_types::Server;
use serde_json::Value;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

use crate::services::catalog::release::jobs::record;
use crate::{AppContext, AppError};

pub const DEBOUNCE: Duration = Duration::from_secs(5);
const MAX_BACKOFF: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineMessage {
    UpdateComplete { version: String },
    Error { message: String },
    Other(String),
}

#[must_use]
pub fn parse_message(text: &str) -> Option<PipelineMessage> {
    let value: Value = serde_json::from_str(text).ok()?;
    let field = |name: &str| {
        value
            .get(name)
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_owned()
    };
    Some(match value.get("type")?.as_str()? {
        "update_complete" => PipelineMessage::UpdateComplete {
            version: field("version"),
        },
        "error" => PipelineMessage::Error {
            message: field("message"),
        },
        other => PipelineMessage::Other(other.to_owned()),
    })
}

#[must_use]
pub fn pipeline_urls(configured: &BTreeMap<String, String>) -> Vec<(Server, String)> {
    let mut urls: Vec<(Server, String)> = Vec::new();
    for (name, url) in configured {
        let Some(named) = Server::parse(name.trim()) else {
            tracing::warn!(name, "assets.ws_urls names no server, skipped");
            continue;
        };
        let (server, url) = (named.data_server(), url.trim());
        if url.is_empty() {
            continue;
        }
        match urls.iter_mut().find(|(s, _)| *s == server) {
            Some(kept) => {
                tracing::warn!(
                    name,
                    server = server.as_str(),
                    "assets.ws_urls names one data server twice, one listener kept"
                ); // two listeners bump it twice per update
                if named == server {
                    kept.1 = url.to_owned();
                }
            }
            None => urls.push((server, url.to_owned())),
        }
    }
    urls
}

pub async fn listen_for_updates(ctx: &AppContext) {
    let urls = pipeline_urls(&ctx.config.assets.ws_urls);
    if urls.is_empty() {
        tracing::info!("no asset pipeline WebSocket configured, updates arrive only by restart");
        return;
    }
    futures_util::future::join_all(urls.iter().map(|(server, url)| listen(ctx, *server, url)))
        .await;
}

#[derive(Debug, Default)]
pub struct Announcer {
    pending: Option<String>,
    absorbed: bool,
    failures: u32,
    retry_at: Option<Instant>,
    finished: Option<Instant>,
}

impl Announcer {
    pub fn receive(&mut self, version: String, now: Instant) -> bool {
        let repeat = self
            .finished
            .is_some_and(|at| now.saturating_duration_since(at) < DEBOUNCE);
        if repeat && self.pending.is_none() {
            return false;
        }
        self.absorbed |= self.pending.is_some();
        self.pending = Some(version);
        true
    }

    #[must_use]
    pub fn due(&self, now: Instant) -> Option<Option<&str>> {
        let pending = self.pending.as_deref()?;
        self.retry_at
            .is_none_or(|at| now >= at)
            .then_some((!self.absorbed).then_some(pending))
    }

    #[must_use]
    pub fn wait(&self, now: Instant) -> Option<Duration> {
        self.pending.as_ref()?;
        Some(
            self.retry_at
                .map_or(Duration::ZERO, |at| at.saturating_duration_since(now)),
        )
    }

    pub fn finished(&mut self, now: Instant) {
        *self = Self {
            finished: Some(now),
            ..Self::default()
        };
    }

    pub fn failed(&mut self, now: Instant) {
        self.failures = self.failures.saturating_add(1);
        self.retry_at = Some(now + retry_delay(self.failures));
    }
}

#[must_use]
pub fn retry_delay(failures: u32) -> Duration {
    Duration::from_secs(1 << failures.saturating_sub(1).min(5)).min(MAX_BACKOFF)
}

async fn listen(ctx: &AppContext, server: Server, url: &str) {
    let mut announcer = Announcer::default();
    let mut backoff = Duration::from_secs(1);
    loop {
        announce_due(ctx, server, &mut announcer).await;
        match connect_async(url).await {
            Ok((stream, _)) => {
                tracing::info!(server = server.as_str(), url, "asset pipeline connected");
                backoff = Duration::from_secs(1);
                read(ctx, server, stream, &mut announcer).await;
                tracing::warn!(server = server.as_str(), "asset pipeline disconnected");
            }
            Err(e) => {
                tracing::warn!(server = server.as_str(), url, error = %e, retry_in = ?backoff, "asset pipeline connect failed");
            }
        }
        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }
}

async fn read<S>(ctx: &AppContext, server: Server, mut frames: S, announcer: &mut Announcer)
where
    S: Stream<Item = Result<Message, WsError>> + Unpin,
{
    loop {
        let next = match announcer.wait(Instant::now()) {
            Some(wait) => tokio::select! {
                frame = frames.next() => frame,
                () = tokio::time::sleep(wait) => {
                    announce_due(ctx, server, announcer).await;
                    continue;
                }
            },
            None => frames.next().await,
        };
        let Some(frame) = next else { return };
        let text = match frame {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) => return,
            Ok(_) => continue,
            Err(e) => {
                tracing::warn!(server = server.as_str(), error = %e, "asset pipeline read failed");
                return;
            }
        };
        match parse_message(text.as_str()) {
            Some(PipelineMessage::UpdateComplete { version }) => {
                tracing::info!(server = server.as_str(), version, "asset update complete");
                if announcer.receive(version, Instant::now()) {
                    announce_due(ctx, server, announcer).await;
                } else {
                    tracing::debug!(server = server.as_str(), "update repeated, skipped");
                }
            }
            Some(PipelineMessage::Error { message }) => {
                tracing::warn!(server = server.as_str(), message, "asset pipeline error");
            }
            Some(PipelineMessage::Other(kind)) => {
                tracing::debug!(server = server.as_str(), kind, "asset pipeline event");
            }
            None => {}
        }
    }
}

async fn announce_due(ctx: &AppContext, server: Server, announcer: &mut Announcer) {
    let Some(version) = announcer.due(Instant::now()).map(|v| v.map(str::to_owned)) else {
        return;
    };
    match announce_and_record(ctx, server, version.as_deref()).await {
        Ok(()) => announcer.finished(Instant::now()),
        Err(e) => {
            announcer.failed(Instant::now());
            tracing::error!(server = server.as_str(), ?version, failures = announcer.failures, error = %e, "update not announced, kept for a retry");
        }
    }
}

pub async fn announce_and_record(
    ctx: &AppContext,
    server: Server,
    version: Option<&str>,
) -> Result<(), AppError> {
    reload_logged(ctx).await;
    record_logged(ctx, server).await;
    ctx.announce_update(server, version).await?;
    reload_logged(ctx).await;
    record_logged(ctx, server).await;
    Ok(())
}

async fn reload_logged(ctx: &AppContext) {
    if let Err(e) = ctx.reload_if_changed().await {
        tracing::error!(error = %e, "game data generation check failed");
    }
}

async fn record_logged(ctx: &AppContext, server: Server) {
    match record(ctx, server).await {
        Ok(Some(recorded)) => tracing::info!(recorded, "release ledger recorded"),
        Ok(None) => {}
        Err(e) => {
            tracing::error!(server = server.as_str(), error = %e, "release ledger not recorded");
        }
    }
}
