//! Change detection, atomic writes and the pending-reload flag for the `derived/` sidecars.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use myrtle_types::Server;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{AppContext, AppError};

/// Data servers whose sidecar on disk may be ahead of the loaded snapshot: a write landed or a reload failed
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

/// Clears the flag, returning whether it was set
pub fn take_pending(server: Server) -> bool {
    let server = server.data_server();
    let mut pending = PENDING.lock().unwrap_or_else(PoisonError::into_inner);
    let before = pending.len();
    pending.retain(|s| *s != server);
    pending.len() != before
}

/// The comparable JSON of a sidecar's data part; map key order does not count
#[derive(Debug, Clone)]
pub struct Fingerprint(Option<Value>);

impl Fingerprint {
    pub fn of<T: Serialize + ?Sized>(data: &T) -> Self {
        Self(serde_json::to_value(data).ok())
    }

    /// A side that failed to serialize always differs, so a failure can only cost a reload
    #[must_use]
    pub fn differs(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(a), Some(b)) => a != b,
            _ => true,
        }
    }
}

/// `{dir}/{server}.json` when the job's `dir` is set, else `rel_path` under the server's extract
#[must_use]
pub fn path(ctx: &AppContext, server: Server, dir: &str, rel_path: &str) -> PathBuf {
    if dir.trim().is_empty() {
        ctx.snapshots.server_dir(server).join(rel_path)
    } else {
        PathBuf::from(dir).join(format!("{}.json", server.data_server().as_str()))
    }
}

/// The parsed file, `None` when absent or unreadable: a refetch is cheaper than a repair
pub fn read<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes)
        .inspect_err(|e| tracing::warn!(path = %path.display(), error = %e, "sidecar unreadable, rebuilding"))
        .ok()
}

/// Temp file then rename, so the loader never reads a truncated file
pub fn write(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(AppError::internal)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, bytes).map_err(AppError::internal)?;
    std::fs::rename(&tmp, path).map_err(AppError::internal)
}

/// Rebuilds `server`'s snapshot; the flag is taken first so a write during the build stays pending
pub async fn reload(ctx: &AppContext, server: Server) -> Result<(), AppError> {
    take_pending(server);
    ctx.reload_game_data(server).await.inspect_err(|e| {
        mark_pending(server);
        tracing::error!(server = server.as_str(), error = %e, "sidecar reload failed");
    })
}

/// One account per data server, the server's own ahead of a channel sharing its data
#[must_use]
pub fn account_servers(ctx: &AppContext) -> Vec<Server> {
    let mut servers = ctx.accounts.servers();
    servers.sort_by_key(|s| (s.data_server().as_str(), *s != s.data_server()));
    servers.dedup_by_key(|s| s.data_server());
    servers
}
