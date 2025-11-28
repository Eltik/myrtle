use tokio::sync::broadcast;

use crate::core::authentication::constants::{AuthSession, Server};

pub mod setup_event_listeners;

#[derive(Debug, Clone)]
pub enum ConfigEvent {
    NetworkLoaded(Server),
    NetworkInitiated,
    VersionLoaded(Server),
    VersionInitiated,
    DeviceIdsGenerated,
    AuthLoginSuccess(AuthSession),
    AuthLoginError(String),
}

pub struct EventEmitter {
    sender: broadcast::Sender<ConfigEvent>,
}

impl EventEmitter {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(32);
        Self { sender }
    }

    pub fn emit(&self, event: ConfigEvent) {
        let _ = self.sender.send(event); // Ignore if no receivers
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ConfigEvent> {
        self.sender.subscribe()
    }
}
