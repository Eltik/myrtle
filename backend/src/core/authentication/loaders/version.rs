use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;

use crate::core::authentication::config::{GlobalConfig, VersionInfo};
use crate::core::authentication::constants::{Domain, Server};
use crate::events::{ConfigEvent, EventEmitter};

const CONFIG_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn load_version_config(
    client: &Client,
    config: &Arc<RwLock<GlobalConfig>>,
    events: &Arc<EventEmitter>,
    server: Option<Server>,
) {
    match server {
        Some(s) => {
            load_single_server(client, config, s).await;
            events.emit(ConfigEvent::VersionLoaded(s));
        }
        None => {
            for &s in Server::all() {
                load_single_server(client, config, s).await;
                events.emit(ConfigEvent::VersionLoaded(s));
            }
            events.emit(ConfigEvent::VersionInitiated);
        }
    }
}

async fn load_single_server(client: &Client, config: &Arc<RwLock<GlobalConfig>>, server: Server) {
    let hv_url = {
        let config = config.read().await;
        config
            .domains
            .get(&server)
            .and_then(|domains| domains.get(&Domain::HV))
            .cloned()
    };

    let Some(url) = hv_url else {
        eprintln!("No HV domain found for {:?}, skipping version load", server);
        return;
    };

    let result = timeout(CONFIG_TIMEOUT, async {
        let response = client.get(&url).timeout(CONFIG_TIMEOUT).send().await?;

        let version_info: VersionInfo = response.json().await?;
        Ok::<_, reqwest::Error>(version_info)
    })
    .await;

    match result {
        Ok(Ok(version_info)) => {
            let mut config = config.write().await;
            config.versions.insert(server, version_info);
            println!("Version config loaded for {:?}", server);
        }
        Ok(Err(e)) => {
            eprintln!("Error loading version config for {:?}: {}", server, e);
        }
        Err(_) => {
            eprintln!("Timeout loading version config for {:?}", server);
        }
    }
}

pub async fn reset_version_config(config: &Arc<RwLock<GlobalConfig>>) {
    let mut config = config.write().await;
    config.reset_versions();
}
