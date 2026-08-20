//! Service accounts: game accounts owned by the backend rather than by a user.
//!
//! Every `Domain::GS` route authenticates on the `uid` / `secret` / `seqnum`
//! headers; there is no anonymous mode. Impersonal reads — banner metadata and
//! the like — therefore still need a logged-in account, so the backend keeps
//! its own, one per server.
//!
//! # Invariants
//!
//! - **One active `secret` per account.** Re-minting invalidates the previous
//!   one, so a service account must never be a real player's: refreshing would
//!   sign them out of the game. Use a dedicated account per server.
//! - **`seqnum` is sequential per session.** Concurrent requests race on the
//!   counter and desync it. The [`Mutex`] on each account is what enforces the
//!   ordering, which is why every call funnels through
//!   [`ServiceAccount::request`] rather than touching the session directly.
//!
//! # Credentials
//!
//! A stored session carries a durable `yostar_uid` / `yostar_token` pair, which
//! [`refresh_secret`] trades for a live `secret` without an OTP round-trip. A
//! human logs in once per account (`cargo run --bin set-session`); the backend
//! renews itself from there.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use reqwest::{Client, StatusCode};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::core::hypergryph::{
    constants::{AuthSession, Server},
    fetch::{FetchError, auth_request, parse_json},
    session::refresh_secret,
};

const DEFAULT_SESSION_DIR: &str = "game_sessions";

/// How long a minted `secret` is trusted before a pre-emptive refresh. The
/// upstream lifetime is not published; short enough to rarely hit a 401, long
/// enough that a burst of calls costs one refresh rather than many.
const DEFAULT_SECRET_MAX_AGE_SECS: u64 = 30 * 60;

fn secret_max_age() -> Duration {
    Duration::from_secs(
        std::env::var("GAME_SESSION_MAX_AGE_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_SECRET_MAX_AGE_SECS),
    )
}

/// Directory holding one session file per server. Override with
/// `GAME_SESSION_DIR`.
pub fn session_dir() -> PathBuf {
    PathBuf::from(std::env::var("GAME_SESSION_DIR").unwrap_or_else(|_| DEFAULT_SESSION_DIR.into()))
}

/// Path of a single server's session file.
pub fn session_path(server: Server) -> PathBuf {
    session_dir().join(format!("{}.json", server.as_str()))
}

/// Read a persisted session. Returns `None` when absent, unreadable, or lacking
/// the durable token pair that makes unattended refresh possible.
pub fn read_session_file(path: &Path) -> Option<AuthSession> {
    let raw = std::fs::read_to_string(path).ok()?;
    let session: AuthSession = serde_json::from_str(&raw).ok()?;
    if session.yostar_uid.is_empty() || session.yostar_token.is_empty() {
        tracing::warn!(
            path = %path.display(),
            "session file has no durable token; re-run `set-session`"
        );
        return None;
    }
    Some(session)
}

/// Persist a session atomically (temp file + rename), `0600` on unix.
///
/// The contents are credential material — a live game `secret` and a durable
/// account token — not a cache.
pub fn write_session_file(path: &Path, session: &AuthSession) -> std::io::Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(session)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(&tmp, json)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600))?;
    }

    std::fs::rename(&tmp, path)
}

struct Inner {
    session: AuthSession,
    /// `None` until the first successful refresh in this process.
    refreshed_at: Option<Instant>,
}

/// A shared, serialized handle to one server's service account.
///
/// Cheap to clone via `Arc`; all callers share one session and one `seqnum`
/// counter by construction.
pub struct ServiceAccount {
    inner: Mutex<Inner>,
    server: Server,
    path: PathBuf,
    max_age: Duration,
}

impl ServiceAccount {
    /// Load this server's account from disk, or `None` when no usable
    /// credentials are stored for it.
    pub fn load(server: Server) -> Option<Arc<Self>> {
        let path = session_path(server);
        let session = read_session_file(&path)?;
        tracing::info!(
            uid = %session.uid,
            server = server.as_str(),
            path = %path.display(),
            "service account loaded"
        );
        Some(Self::new(session, server, path))
    }

    pub fn new(session: AuthSession, server: Server, path: PathBuf) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(Inner {
                session,
                refreshed_at: None,
            }),
            server,
            path,
            max_age: secret_max_age(),
        })
    }

    pub const fn server(&self) -> Server {
        self.server
    }

    /// The account's in-game UID, for logging.
    pub async fn uid(&self) -> String {
        self.inner.lock().await.session.uid.to_string()
    }

    /// Make an authenticated `Domain::GS` request.
    ///
    /// Holds the session lock for the whole call, serializing `seqnum` across
    /// callers. Refreshes the `secret` pre-emptively when stale and once
    /// reactively on a `401`; a second `401` means the durable token is dead and
    /// surfaces as [`FetchError::NotLoggedIn`].
    pub async fn request(
        &self,
        client: &Client,
        endpoint: &str,
        body: &Value,
    ) -> Result<Value, FetchError> {
        let mut guard = self.inner.lock().await;

        if guard
            .refreshed_at
            .is_none_or(|at| at.elapsed() >= self.max_age)
        {
            self.refresh(client, &mut guard).await?;
        }

        let response = auth_request(
            client,
            endpoint,
            Some(body),
            &mut guard.session,
            self.server,
        )
        .await?;

        let response = if response.status() == StatusCode::UNAUTHORIZED {
            tracing::debug!(
                endpoint,
                server = self.server.as_str(),
                "401, re-minting secret"
            );
            self.refresh(client, &mut guard).await?;

            let retried = auth_request(
                client,
                endpoint,
                Some(body),
                &mut guard.session,
                self.server,
            )
            .await?;
            if retried.status() == StatusCode::UNAUTHORIZED {
                return Err(FetchError::NotLoggedIn);
            }
            retried
        } else {
            response
        };

        let parsed = parse_json(response, endpoint).await;
        self.persist(&guard.session);
        parsed
    }

    async fn refresh(&self, client: &Client, inner: &mut Inner) -> Result<(), FetchError> {
        refresh_secret(client, &mut inner.session, self.server).await?;
        inner.refreshed_at = Some(Instant::now());
        self.persist(&inner.session);
        Ok(())
    }

    /// Best-effort write-back of the bumped `seqnum` and fresh `secret`. A
    /// failure costs a re-mint on next boot, so it warns rather than propagates.
    fn persist(&self, session: &AuthSession) {
        if let Err(e) = write_session_file(&self.path, session) {
            tracing::warn!(
                error = %e,
                path = %self.path.display(),
                "failed to persist service account session"
            );
        }
    }
}

/// Every service account the backend has credentials for, keyed by server.
///
/// Servers without a stored session are simply absent; callers treat a missing
/// account as "this feature is off for that server" rather than an error.
#[derive(Default)]
pub struct ServiceAccounts {
    accounts: HashMap<Server, Arc<ServiceAccount>>,
}

impl ServiceAccounts {
    /// Load an account for each of `servers` that has stored credentials.
    pub fn load(servers: &[Server]) -> Self {
        let accounts = servers
            .iter()
            .filter_map(|&server| ServiceAccount::load(server).map(|a| (server, a)))
            .collect::<HashMap<_, _>>();

        if accounts.is_empty() {
            tracing::info!(
                dir = %session_dir().display(),
                "no service accounts configured - run `cargo run --bin set-session`"
            );
        }

        Self { accounts }
    }

    pub fn get(&self, server: Server) -> Option<&Arc<ServiceAccount>> {
        self.accounts.get(&server)
    }

    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }

    pub fn len(&self) -> usize {
        self.accounts.len()
    }

    /// Every server that has a usable account.
    pub fn servers(&self) -> Vec<Server> {
        self.accounts.keys().copied().collect()
    }
}
