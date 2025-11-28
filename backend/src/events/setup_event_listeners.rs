use crate::events::{ConfigEvent, EventEmitter};
use std::sync::Arc;

pub fn setup_event_listeners(events: &Arc<EventEmitter>) {
    let mut rx = events.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                ConfigEvent::NetworkLoaded(server) => {
                    println!("Network config loaded for {:?}", server);
                }
                ConfigEvent::NetworkInitiated => {
                    println!("All network configs loaded!");
                }
                ConfigEvent::VersionLoaded(server) => {
                    println!("Version config loaded for {:?}", server);
                }
                ConfigEvent::VersionInitiated => {
                    println!("All version configs loaded!");
                }
                ConfigEvent::DeviceIdsGenerated => {
                    println!("Device IDs generated!");
                }
                ConfigEvent::AuthLoginSuccess(session) => {
                    println!("Login successful for uid: {}", session.uid);
                }
                ConfigEvent::AuthLoginError(error) => {
                    eprintln!("Login failed: {}", error);
                }
            }
        }
    });
}
