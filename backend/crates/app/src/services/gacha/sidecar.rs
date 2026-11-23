use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use myrtle_types::Server;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::snapshots::write_atomic;
use crate::{AppContext, AppError};

static PENDING: Mutex<Vec<Server>> = Mutex::new(Vec::new());

pub fn mark_pending(server: Server) {
    let server = server.data_server();
    let mut pending = PENDING.lock().unwrap_or_else(PoisonError::into_inner);
    if !pending.contains(&server) {
        pending.push(server);
    }
}

pub fn is_pending(server: Server) -> bool {
    PENDING
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .contains(&server.data_server())
}

pub fn take_pending(server: Server) -> bool {
    let server = server.data_server();
    let mut pending = PENDING.lock().unwrap_or_else(PoisonError::into_inner);
    let before = pending.len();
    pending.retain(|s| *s != server);
    pending.len() != before
}

#[derive(Debug, Clone)]
pub struct Fingerprint(Option<Value>);

impl Fingerprint {
    pub fn of<T: Serialize + ?Sized>(data: &T) -> Self {
        Self(serde_json::to_value(data).ok())
    }

    #[must_use]
    pub fn differs(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(a), Some(b)) => a != b,
            _ => true,
        }
    }
}

#[must_use]
pub fn path(ctx: &AppContext, server: Server, dir: &str, rel_path: &str) -> PathBuf {
    if dir.trim().is_empty() {
        ctx.snapshots.server_dir(server).join(rel_path)
    } else {
        PathBuf::from(dir).join(format!("{}.json", server.data_server().as_str()))
    }
}

#[must_use]
pub fn read<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes)
        .inspect_err(|e| tracing::warn!(path = %path.display(), error = %e, "sidecar unreadable, rebuilding"))
        .ok()
}

pub fn write(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    write_atomic(path, bytes).map_err(AppError::internal)
}

pub async fn reload(ctx: &AppContext, server: Server) -> Result<(), AppError> {
    take_pending(server);
    ctx.reload_game_data(server).await.inspect_err(|e| {
        mark_pending(server);
        tracing::error!(server = server.as_str(), error = %e, "sidecar reload failed");
    })
}

#[must_use]
pub fn account_servers(ctx: &AppContext) -> Vec<Server> {
    let mut servers = ctx.accounts.servers();
    servers.sort_by_key(|s| (s.data_server().as_str(), *s != s.data_server()));
    servers.dedup_by_key(|s| s.data_server());
    servers
}
