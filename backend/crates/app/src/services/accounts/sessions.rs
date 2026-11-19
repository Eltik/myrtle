//! The game session store: players in the cache with their sealed credential in
//! `user_game_credentials`, service accounts sealed in `service_account_sessions`.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use myrtle_db::repos::service_sessions::{self, SealedSessionRow};
use myrtle_db::repos::{credentials, users};
use myrtle_db::{Db, DbError};
use myrtle_hypergryph::{
    AuthSession, DurableCredential, SessionKey, SessionStore, SessionStoreError,
};
use myrtle_types::{GameUid, Server, UserId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::credentials::CredentialKey;
use crate::cache::Cache;

pub const SESSION_TTL: Duration = Duration::from_hours(1);
pub const PORTAL_SESSION_TTL: Duration = Duration::from_hours(168);

#[must_use]
pub fn session_key(uid: &str) -> String {
    format!("game_session:{uid}")
}

#[must_use]
pub fn portal_session_key(uid: &str) -> String {
    format!("portal_session:{uid}")
}

#[derive(Serialize, Deserialize)]
struct StoredCredential {
    yostar_uid: String,
    yostar_token: String,
}

#[derive(Debug, Clone)]
pub struct Sessions {
    db: Db,
    cache: Cache,
    key: Arc<CredentialKey>,
}

fn unavailable(e: impl std::fmt::Display) -> SessionStoreError {
    SessionStoreError::Unavailable(e.to_string())
}

fn corrupt(e: impl std::fmt::Display) -> SessionStoreError {
    SessionStoreError::Corrupt(e.to_string())
}

/// Never serde's message: it can quote a session field
fn malformed(e: &serde_json::Error) -> SessionStoreError {
    SessionStoreError::Corrupt(format!(
        "{:?} at line {}, column {}",
        e.classify(),
        e.line(),
        e.column()
    ))
}

/// The owner a service account's blob is sealed to
const fn service_owner(server: Server) -> Uuid {
    Uuid::from_u128(server.index() as u128 + 1)
}

impl Sessions {
    #[must_use]
    pub const fn new(db: Db, cache: Cache, key: Arc<CredentialKey>) -> Self {
        Self { db, cache, key }
    }

    async fn owner(&self, server: Server, uid: &GameUid) -> Result<Option<UserId>, DbError> {
        let mut conn = self.db.conn().await?;
        Ok(users::find(&mut conn, uid.as_str(), server)
            .await?
            .map(|user| UserId(user.id)))
    }

    /// Seals a player's credential under `user`; a session with no durable half is skipped
    pub async fn store_credential(
        &self,
        user: UserId,
        session: &AuthSession,
    ) -> Result<(), SessionStoreError> {
        let Some(credential) = DurableCredential::of(session) else {
            return Ok(());
        };
        self.seal_player(user, &credential).await
    }

    async fn seal_player(
        &self,
        user: UserId,
        credential: &DurableCredential,
    ) -> Result<(), SessionStoreError> {
        let plaintext = serde_json::to_vec(&StoredCredential {
            yostar_uid: credential.yostar_uid.clone(),
            yostar_token: credential.yostar_token.clone(),
        })
        .map_err(corrupt)?;
        let (nonce, ciphertext) = self.key.seal(user.0, &plaintext).map_err(corrupt)?;
        let mut conn = self.db.conn().await.map_err(unavailable)?;
        credentials::store(&mut conn, user, &nonce, &ciphertext)
            .await
            .map_err(unavailable)
    }

    pub async fn service_session(
        &self,
        server: Server,
    ) -> Result<Option<AuthSession>, SessionStoreError> {
        let mut conn = self.db.conn().await.map_err(unavailable)?;
        let row = service_sessions::get(&mut conn, server)
            .await
            .map_err(unavailable)?;
        row.map(|row| self.open_service(server, &row)).transpose()
    }

    fn open_service(
        &self,
        server: Server,
        row: &SealedSessionRow,
    ) -> Result<AuthSession, SessionStoreError> {
        let plaintext = self
            .key
            .open(service_owner(server), &row.nonce, &row.ciphertext)
            .map_err(corrupt)?;
        serde_json::from_slice(&plaintext).map_err(|e| malformed(&e))
    }

    fn seal_service(
        &self,
        server: Server,
        session: &AuthSession,
    ) -> Result<(Vec<u8>, Vec<u8>), SessionStoreError> {
        let plaintext = serde_json::to_vec(session).map_err(|e| malformed(&e))?;
        self.key
            .seal(service_owner(server), &plaintext)
            .map_err(corrupt)
    }

    /// A new login: replaces the whole session, or with `replace` false only fills an empty row; false when nothing was written
    pub async fn seed(
        &self,
        server: Server,
        session: &AuthSession,
        replace: bool,
    ) -> Result<bool, SessionStoreError> {
        let (nonce, ciphertext) = self.seal_service(server, session)?;
        let mut conn = self.db.conn().await.map_err(unavailable)?;
        if replace {
            service_sessions::upsert(&mut conn, server, &nonce, &ciphertext)
                .await
                .map_err(unavailable)?;
            return Ok(true);
        }
        service_sessions::insert_new(&mut conn, server, &nonce, &ciphertext)
            .await
            .map_err(unavailable)
    }

    /// A running account's session, dropped when the stored one is another login: a seed landed since it was read
    pub async fn persist_service(
        &self,
        server: Server,
        session: &AuthSession,
    ) -> Result<(), SessionStoreError> {
        let live = DurableCredential::of(session);
        self.merge_service(server, |stored| {
            let seeded = stored.as_ref().and_then(DurableCredential::of);
            match seeded {
                Some(seeded) if Some(&seeded) != live.as_ref() => None,
                _ => Some(session.clone()),
            }
        })
        .await
    }

    /// `merge` runs on the stored session under its row lock, so no write lands between the read and the write; `None` writes nothing
    async fn merge_service(
        &self,
        server: Server,
        merge: impl FnOnce(Option<AuthSession>) -> Option<AuthSession> + Send,
    ) -> Result<(), SessionStoreError> {
        let mut tx = self.db.begin().await.map_err(unavailable)?;
        let held = service_sessions::lock(&mut tx, server)
            .await
            .map_err(unavailable)?;
        let stored = match &held {
            Some(row) => match self.open_service(server, row) {
                Ok(session) => Some(session),
                Err(e) => {
                    tracing::warn!(
                        server = server.as_str(),
                        error = e.kind(),
                        "replacing a stored service session that did not open"
                    );
                    None
                }
            },
            None => None,
        };
        let Some(merged) = merge(stored) else {
            return Ok(());
        };
        let (nonce, ciphertext) = self.seal_service(server, &merged)?;
        if held.is_some() {
            service_sessions::upsert(&mut tx, server, &nonce, &ciphertext).await
        } else {
            service_sessions::insert_new(&mut tx, server, &nonce, &ciphertext)
                .await
                .map(drop) // a seed that landed after the empty lock is newer
        }
        .map_err(unavailable)?;
        tx.commit().await.map_err(unavailable)
    }
}

#[async_trait]
impl SessionStore for Sessions {
    async fn load(&self, key: &SessionKey) -> Result<Option<AuthSession>, SessionStoreError> {
        match key {
            SessionKey::Player { uid, .. } => {
                Ok(self.cache.get_json(&session_key(uid.as_str())).await)
            }
            SessionKey::ServiceAccount { server } => self.service_session(*server).await,
        }
    }

    async fn save(&self, key: &SessionKey, session: &AuthSession) -> Result<(), SessionStoreError> {
        match key {
            SessionKey::Player { uid, .. } => {
                self.cache
                    .set_json(&session_key(uid.as_str()), session, SESSION_TTL)
                    .await;
                Ok(())
            }
            SessionKey::ServiceAccount { server } => self.persist_service(*server, session).await,
        }
    }

    async fn load_durable(
        &self,
        key: &SessionKey,
    ) -> Result<Option<DurableCredential>, SessionStoreError> {
        let (server, uid) = match key {
            SessionKey::Player { server, uid } => (*server, uid),
            SessionKey::ServiceAccount { server } => {
                let session = self.service_session(*server).await?;
                return Ok(session.as_ref().and_then(DurableCredential::of));
            }
        };
        let Some(user) = self.owner(server, uid).await.map_err(unavailable)? else {
            return Ok(None);
        };
        let mut conn = self.db.conn().await.map_err(unavailable)?;
        let Some(row) = credentials::load(&mut conn, user)
            .await
            .map_err(unavailable)?
        else {
            return Ok(None);
        };
        let stored = self
            .key
            .open(user.0, &row.nonce, &row.ciphertext)
            .ok()
            .and_then(|plaintext| serde_json::from_slice::<StoredCredential>(&plaintext).ok());
        if stored.is_none() {
            tracing::warn!(user = %user.0, "stored game credential did not open");
        }
        Ok(stored.map(|stored| DurableCredential {
            yostar_uid: stored.yostar_uid,
            yostar_token: stored.yostar_token,
        }))
    }

    async fn save_durable(
        &self,
        key: &SessionKey,
        credential: &DurableCredential,
    ) -> Result<(), SessionStoreError> {
        match key {
            SessionKey::Player { server, uid } => {
                let Some(user) = self.owner(*server, uid).await.map_err(unavailable)? else {
                    return Err(unavailable("no account for this uid"));
                };
                self.seal_player(user, credential).await
            }
            SessionKey::ServiceAccount { server } => {
                self.merge_service(*server, |stored| {
                    let mut session = stored.unwrap_or_default();
                    session.yostar_uid = credential.yostar_uid.as_str().into();
                    session.yostar_token = credential.yostar_token.as_str().into();
                    Some(session)
                })
                .await
            }
        }
    }

    async fn forget(&self, key: &SessionKey) -> Result<bool, SessionStoreError> {
        match key {
            SessionKey::Player { server, uid } => {
                self.cache.invalidate(&session_key(uid.as_str())).await;
                self.cache
                    .invalidate(&portal_session_key(uid.as_str()))
                    .await;
                let Some(user) = self.owner(*server, uid).await.map_err(unavailable)? else {
                    return Ok(false);
                };
                let mut conn = self.db.conn().await.map_err(unavailable)?;
                credentials::delete(&mut conn, user)
                    .await
                    .map_err(unavailable)
            }
            SessionKey::ServiceAccount { server } => {
                let mut conn = self.db.conn().await.map_err(unavailable)?;
                service_sessions::delete(&mut conn, *server)
                    .await
                    .map_err(unavailable)
            }
        }
    }
}
