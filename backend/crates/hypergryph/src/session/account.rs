use std::collections::HashMap;
use std::sync::{Arc, PoisonError};
use std::time::{Duration, Instant};

use myrtle_types::Server;
use reqwest::StatusCode;
use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::error::{RefreshStep, UpstreamError};
use crate::fetch::{auth_request_with_timeout, parse_json_tolerating};
use crate::session::health::{HealthTracker, state_of};
use crate::session::store::{DurableCredential, SessionKey, SessionStore, SessionStoreError};
use crate::session::{AuthSession, refresh_secret_by_step};
use crate::upstream::Upstream;

pub const DEFAULT_SECRET_MAX_AGE: Duration = Duration::from_mins(30);
pub const SYNC_TIMEOUT: Duration = Duration::from_mins(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccountOptions {
    pub secret_max_age: Duration,
}

impl Default for AccountOptions {
    fn default() -> Self {
        Self {
            secret_max_age: DEFAULT_SECRET_MAX_AGE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountState {
    Ok,
    Refused,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccountHealth {
    pub server: Server,
    pub state: AccountState,
    pub refused_since: Option<i64>,
    pub last_ok_at: Option<i64>,
}

impl AccountHealth {
    #[must_use]
    pub const fn new(server: Server, refused_since: Option<i64>, last_ok_at: Option<i64>) -> Self {
        Self {
            server,
            state: state_of(refused_since, last_ok_at),
            refused_since,
            last_ok_at,
        }
    }
}

struct Inner {
    session: AuthSession,
    refreshed_at: Option<Instant>,
    alerted: bool,
}

enum Refusal {
    Token(UpstreamError),
    Other(UpstreamError),
}

/// Whether a u8 `getToken` failure means the token itself was refused
#[must_use]
pub const fn refuses_credential(e: &UpstreamError) -> bool {
    matches!(
        e,
        UpstreamError::Envelope(_)
            | UpstreamError::Maintenance(_)
            | UpstreamError::Parse { .. }
            | UpstreamError::LoginRejected { .. }
            | UpstreamError::RiskControl { .. }
            | UpstreamError::Unsupported { .. }
    )
}

fn differs(stored: &DurableCredential, session: &AuthSession) -> bool {
    !stored.yostar_uid.is_empty()
        && !stored.yostar_token.is_empty()
        && (*stored.yostar_uid != *session.yostar_uid
            || *stored.yostar_token != *session.yostar_token)
}

fn now_secs() -> i64 {
    chrono::Utc::now().timestamp()
}

pub struct GameAccount {
    inner: Mutex<Inner>,
    health: std::sync::Mutex<HealthTracker>,
    server: Server,
    key: SessionKey,
    store: Arc<dyn SessionStore>,
    options: AccountOptions,
}

impl GameAccount {
    pub async fn load(
        server: Server,
        store: Arc<dyn SessionStore>,
        options: AccountOptions,
    ) -> Result<Option<Arc<Self>>, SessionStoreError> {
        let key = SessionKey::ServiceAccount { server };
        let mut session = match store.load(&key).await {
            Ok(session) => session.unwrap_or_default(),
            Err(e) => {
                tracing::warn!(
                    server = server.as_str(),
                    error = e.kind(),
                    "service account session unreadable; trying the credential"
                );
                AuthSession::default()
            }
        };
        if DurableCredential::of(&session).is_none() {
            let Some(credential) = store.load_durable(&key).await? else {
                return Ok(None);
            };
            session.yostar_uid = credential.yostar_uid.into();
            session.yostar_token = credential.yostar_token.into();
            if DurableCredential::of(&session).is_none() {
                return Ok(None);
            }
        }
        tracing::info!(uid = %session.uid, server = server.as_str(), "service account loaded");
        Ok(Some(Self::new(session, server, store, options)))
    }

    pub fn new(
        session: AuthSession,
        server: Server,
        store: Arc<dyn SessionStore>,
        options: AccountOptions,
    ) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(Inner {
                session,
                refreshed_at: None,
                alerted: false,
            }),
            health: std::sync::Mutex::default(),
            server,
            key: SessionKey::ServiceAccount { server },
            store,
            options,
        })
    }

    pub const fn server(&self) -> Server {
        self.server
    }

    pub fn health(&self) -> AccountHealth {
        let health = self.health.lock().unwrap_or_else(PoisonError::into_inner);
        AccountHealth::new(self.server, health.refused_since(), health.last_ok_at())
    }

    pub async fn uid(&self) -> String {
        self.inner.lock().await.session.uid.to_string()
    }

    pub async fn request(
        &self,
        upstream: &Upstream,
        endpoint: &str,
        body: &Value,
    ) -> Result<Value, UpstreamError> {
        self.request_with(upstream, endpoint, body, &[], upstream.request_timeout())
            .await
    }

    pub async fn request_tolerating(
        &self,
        upstream: &Upstream,
        endpoint: &str,
        body: &Value,
        tolerated: &[i64],
    ) -> Result<Value, UpstreamError> {
        self.request_with(
            upstream,
            endpoint,
            body,
            tolerated,
            upstream.request_timeout(),
        )
        .await
    }

    pub async fn sync_player(&self, upstream: &Upstream) -> Result<(), UpstreamError> {
        self.request_with(
            upstream,
            "account/syncData",
            &json!({ "platform": 1 }),
            &[],
            SYNC_TIMEOUT,
        )
        .await
        .map(drop)
    }

    pub async fn probe(&self, upstream: &Upstream) -> Result<(), UpstreamError> {
        let mut inner = self.inner.lock().await;
        self.refresh(upstream, &mut inner).await
    }

    async fn request_with(
        &self,
        upstream: &Upstream,
        endpoint: &str,
        body: &Value,
        tolerated: &[i64],
        timeout: Duration,
    ) -> Result<Value, UpstreamError> {
        let mut guard = self.inner.lock().await;
        let inner = &mut *guard;

        if inner
            .refreshed_at
            .is_none_or(|at| at.elapsed() >= self.options.secret_max_age)
        {
            self.refresh(upstream, inner).await?;
        }

        let mut response = auth_request_with_timeout(
            upstream,
            endpoint,
            Some(body),
            &mut inner.session,
            self.server,
            timeout,
        )
        .await?;

        if response.status() == StatusCode::UNAUTHORIZED {
            tracing::debug!(
                endpoint,
                server = self.server.as_str(),
                "401, re-minting secret"
            );
            self.refresh(upstream, inner).await?;
            response = auth_request_with_timeout(
                upstream,
                endpoint,
                Some(body),
                &mut inner.session,
                self.server,
                timeout,
            )
            .await?;
            if response.status() == StatusCode::UNAUTHORIZED {
                return Err(UpstreamError::NotLoggedIn);
            }
        }
        let ok_at = self.mark_ok();

        let parsed = parse_json_tolerating(response, endpoint, tolerated).await;
        self.persist(&inner.session, ok_at).await;
        drop(guard);
        parsed
    }

    async fn refresh(&self, upstream: &Upstream, inner: &mut Inner) -> Result<(), UpstreamError> {
        let refused = match self.try_refresh(upstream, inner).await {
            Ok(()) => return Ok(()),
            Err(Refusal::Other(e)) => return Err(e),
            Err(Refusal::Token(e)) => e,
        };

        match self.store.load_durable(&self.key).await {
            Ok(Some(stored)) if differs(&stored, &inner.session) => {
                tracing::info!(
                    server = self.server.as_str(),
                    "service account token refused; adopting the stored credential"
                );
                inner.session.yostar_uid = stored.yostar_uid.into();
                inner.session.yostar_token = stored.yostar_token.into();
                match self.try_refresh(upstream, inner).await {
                    Ok(()) => return Ok(()),
                    Err(Refusal::Other(e)) => return Err(e),
                    Err(Refusal::Token(_)) => {}
                }
            }
            Ok(_) => {}
            Err(e) => {
                tracing::warn!(
                    server = self.server.as_str(),
                    error = e.kind(),
                    "durable credential unreadable"
                );
            }
        }

        tracing::warn!(
            server = self.server.as_str(),
            error = ?refused,
            "service account token refused; log it in again"
        );
        self.mark_refused().await;
        let first_of_outage = !inner.alerted;
        inner.alerted = true;
        Err(UpstreamError::ServiceAccountRefused {
            server: self.server,
            first_of_outage,
            reason: refused.to_string(),
        })
    }

    async fn try_refresh(&self, upstream: &Upstream, inner: &mut Inner) -> Result<(), Refusal> {
        match refresh_secret_by_step(upstream, &mut inner.session, self.server).await {
            Ok(()) => {}
            Err((RefreshStep::Token, e)) if refuses_credential(&e) => {
                return Err(Refusal::Token(e));
            }
            Err((_, e)) => return Err(Refusal::Other(e)),
        }
        inner.refreshed_at = Some(Instant::now());
        inner.alerted = false;
        let ok_at = self.mark_ok();
        self.persist(&inner.session, ok_at).await;
        Ok(())
    }

    fn mark_ok(&self) -> i64 {
        let now = now_secs();
        self.health
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .ok(now);
        now
    }

    async fn mark_refused(&self) {
        let now = now_secs();
        let started = self
            .health
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .refused(now);
        if started && let Err(e) = self.store.save_refused(self.server, now).await {
            tracing::warn!(
                server = self.server.as_str(),
                error = e.kind(),
                "failed to record the service account outage"
            );
        }
    }

    async fn persist(&self, session: &AuthSession, ok_at: i64) {
        if let Err(e) = self.store.save_used(self.server, session, ok_at).await {
            tracing::warn!(
                server = self.server.as_str(),
                error = e.kind(),
                "failed to persist service account session"
            );
        }
    }
}

#[derive(Default)]
pub struct GameAccounts {
    accounts: HashMap<Server, Arc<GameAccount>>,
}

impl GameAccounts {
    pub async fn load(
        servers: &[Server],
        store: &Arc<dyn SessionStore>,
        options: AccountOptions,
    ) -> Self {
        let mut accounts = Self::default();
        let with_channels = servers.iter().copied().flat_map(|server| {
            let channel = (server == Server::CN).then_some(Server::Bilibili);
            std::iter::once(server).chain(channel)
        });
        for server in with_channels {
            match GameAccount::load(server, Arc::clone(store), options).await {
                Ok(Some(account)) => accounts.insert(account),
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(
                        server = server.as_str(),
                        error = e.kind(),
                        "service account unreadable"
                    );
                }
            }
        }
        if accounts.is_empty() {
            tracing::info!("no service accounts configured");
        }
        accounts
    }

    pub fn insert(&mut self, account: Arc<GameAccount>) {
        self.accounts.insert(account.server(), account);
    }

    #[must_use]
    pub fn get(&self, server: Server) -> Option<&Arc<GameAccount>> {
        self.accounts.get(&server)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.accounts.len()
    }

    #[must_use]
    pub fn servers(&self) -> Vec<Server> {
        self.accounts.keys().copied().collect()
    }

    #[must_use]
    pub fn health(&self) -> Vec<AccountHealth> {
        let mut all: Vec<_> = self.accounts.values().map(|a| a.health()).collect();
        all.sort_by_key(|h| h.server.as_str());
        all
    }
}
