//! An in-memory `myrtle_hypergryph::SessionStore` with injectable failures.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use async_trait::async_trait;
use myrtle_hypergryph::{
    AuthSession, DurableCredential, SessionKey, SessionStore, SessionStoreError,
};

#[derive(Debug, Default)]
pub struct MemorySessions {
    state: Mutex<State>,
    pub fail_load: AtomicBool,
    pub fail_save: AtomicBool,
    pub fail_load_durable: AtomicBool,
}

#[derive(Debug, Default)]
struct State {
    sessions: HashMap<SessionKey, AuthSession>,
    durable: HashMap<SessionKey, DurableCredential>,
    saves: Vec<AuthSession>,
}

fn fails(flag: &AtomicBool) -> Result<(), SessionStoreError> {
    if flag.load(Ordering::SeqCst) {
        Err(SessionStoreError::Unavailable("injected".into()))
    } else {
        Ok(())
    }
}

impl MemorySessions {
    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn put_session(&self, key: &SessionKey, session: AuthSession) {
        self.state().sessions.insert(key.clone(), session);
    }

    pub fn put_durable(&self, key: &SessionKey, credential: DurableCredential) {
        self.state().durable.insert(key.clone(), credential);
    }

    #[must_use]
    pub fn session(&self, key: &SessionKey) -> Option<AuthSession> {
        self.state().sessions.get(key).cloned()
    }

    /// Every session `save` stored, oldest first
    #[must_use]
    pub fn saves(&self) -> Vec<AuthSession> {
        self.state().saves.clone()
    }
}

#[async_trait]
impl SessionStore for MemorySessions {
    async fn load(&self, key: &SessionKey) -> Result<Option<AuthSession>, SessionStoreError> {
        fails(&self.fail_load)?;
        Ok(self.session(key))
    }

    async fn save(&self, key: &SessionKey, session: &AuthSession) -> Result<(), SessionStoreError> {
        fails(&self.fail_save)?;
        self.state().saves.push(session.clone());
        self.put_session(key, session.clone());
        Ok(())
    }

    async fn load_durable(
        &self,
        key: &SessionKey,
    ) -> Result<Option<DurableCredential>, SessionStoreError> {
        fails(&self.fail_load_durable)?;
        Ok(self.state().durable.get(key).cloned())
    }

    async fn save_durable(
        &self,
        key: &SessionKey,
        credential: &DurableCredential,
    ) -> Result<(), SessionStoreError> {
        self.put_durable(key, credential.clone());
        Ok(())
    }

    async fn forget(&self, key: &SessionKey) -> Result<bool, SessionStoreError> {
        let session = self.state().sessions.remove(key).is_some();
        let durable = self.state().durable.remove(key).is_some();
        Ok(session || durable)
    }
}
