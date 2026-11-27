use std::fmt;

use async_trait::async_trait;
use myrtle_types::{GameUid, Server};
use thiserror::Error;

use crate::fetch::REDACTED;
use crate::session::AuthSession;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SessionKey {
    Player { server: Server, uid: GameUid },
    ServiceAccount { server: Server },
}

impl SessionKey {
    #[must_use]
    pub const fn server(&self) -> Server {
        match self {
            Self::Player { server, .. } | Self::ServiceAccount { server } => *server,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct DurableCredential {
    pub yostar_uid: String,
    pub yostar_token: String,
}

impl DurableCredential {
    #[must_use]
    pub fn of(session: &AuthSession) -> Option<Self> {
        (!session.yostar_uid.is_empty() && !session.yostar_token.is_empty()).then(|| Self {
            yostar_uid: session.yostar_uid.to_string(),
            yostar_token: session.yostar_token.to_string(),
        })
    }
}

impl fmt::Debug for DurableCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DurableCredential")
            .field("yostar_uid", &self.yostar_uid)
            .field("yostar_token", &REDACTED)
            .finish()
    }
}

#[derive(Error)]
pub enum SessionStoreError {
    #[error("session store unavailable")]
    Unavailable(String),
    #[error("stored session is unreadable")]
    Corrupt(String),
}

impl fmt::Debug for SessionStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SessionStoreError::{}", self.kind())
    }
}

impl SessionStoreError {
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Unavailable(_) => "unavailable",
            Self::Corrupt(_) => "corrupt",
        }
    }

    #[must_use]
    pub fn detail(&self) -> &str {
        match self {
            Self::Unavailable(d) | Self::Corrupt(d) => d,
        }
    }
}

#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn load(&self, key: &SessionKey) -> Result<Option<AuthSession>, SessionStoreError>;

    async fn save(&self, key: &SessionKey, session: &AuthSession) -> Result<(), SessionStoreError>;

    async fn load_durable(
        &self,
        key: &SessionKey,
    ) -> Result<Option<DurableCredential>, SessionStoreError>;

    async fn save_durable(
        &self,
        key: &SessionKey,
        credential: &DurableCredential,
    ) -> Result<(), SessionStoreError>;

    /// Drops the session and the credential
    async fn forget(&self, key: &SessionKey) -> Result<bool, SessionStoreError>;

    /// Saves a service account's session after the game accepted it at `ok_at`
    async fn save_used(
        &self,
        server: Server,
        session: &AuthSession,
        _ok_at: i64,
    ) -> Result<(), SessionStoreError> {
        self.save(&SessionKey::ServiceAccount { server }, session)
            .await
    }

    /// Records the start of a service account's outage
    async fn save_refused(&self, _server: Server, _since: i64) -> Result<(), SessionStoreError> {
        Ok(())
    }
}
