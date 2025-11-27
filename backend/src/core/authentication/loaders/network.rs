use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;

use crate::core::authentication::config::GlobalConfig;
use crate::core::authentication::constants::{Domain, Server};
use crate::events::{ConfigEvent, EventEmitter};

const CONFIG_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Deserialize)]
struct NetworkConfigResponse {
    content: String,
}

#[derive(Deserialize)]
struct NetworkConfigContent {
    configs: HashMap<String, HashMap<String, HashMap<String, String>>>,
    #[serde(rename = "funcVer")]
    func_ver: String,
}

pub async fn load_network_config(
    client: &Client,
    config: &Arc<RwLock<GlobalConfig>>,
    events: &Arc<EventEmitter>,
    server: Option<Server>,
) {
    match server {
        Some(s) => {
            load_single_server(client, config, s).await;
            events.emit(ConfigEvent::NetworkLoaded(s));
        }
        None => {
            for &s in Server::all() {
                load_single_server(client, config, s).await;
                events.emit(ConfigEvent::NetworkLoaded(s));
            }
            events.emit(ConfigEvent::NetworkInitiated);
        }
    }
}

async fn load_single_server(client: &Client, config: &Arc<RwLock<GlobalConfig>>, server: Server) {
    let Some(url) = server.network_route() else {
        eprintln!("No network route for {:?}", server);
        return;
    };

    let result = timeout(CONFIG_TIMEOUT, async {
        let response = client
            .get(url)
            .timeout(CONFIG_TIMEOUT)
            .send()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let outer: NetworkConfigResponse = response
            .json()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let inner: NetworkConfigContent = serde_json::from_str(&outer.content)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let network = inner
            .configs
            .get(&inner.func_ver)
            .and_then(|c| c.get("network"))
            .cloned()
            .unwrap_or_default();

        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(network)
    })
    .await;

    match result {
        Ok(Ok(network)) => {
            let mut config = config.write().await;
            let domains = config.domains.entry(server).or_default();

            // Parse domain strings to Domain enum and store URLs
            for (key, url) in network {
                if let Some(domain) = parse_domain(&key) {
                    domains.insert(domain, url);
                }
            }
            println!("Network config loaded for {:?}", server);
        }
        Ok(Err(e)) => {
            eprintln!("Error loading network config for {:?}: {}", server, e);
        }
        Err(_) => {
            eprintln!("Timeout loading network config for {:?}", server);
        }
    }
}

fn parse_domain(s: &str) -> Option<Domain> {
    match s.to_lowercase().as_str() {
        "gs" => Some(Domain::GS),
        "as" => Some(Domain::AS),
        "u8" => Some(Domain::U8),
        "hu" => Some(Domain::HU),
        "hv" => Some(Domain::HV),
        "rc" => Some(Domain::RC),
        "an" => Some(Domain::AN),
        "prean" => Some(Domain::PREAN),
        "sl" => Some(Domain::SL),
        "of" => Some(Domain::OF),
        "pkgad" => Some(Domain::PkgAd),
        "pkgios" => Some(Domain::PkgIos),
        _ => None, // Unknown domain, skip it
    }
}

pub async fn reset_network_config(config: &Arc<RwLock<GlobalConfig>>) {
    let mut config = config.write().await;
    config.reset_network();
}
