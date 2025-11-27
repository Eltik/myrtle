use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::authentication::config::GlobalConfig;
use crate::events::EventEmitter;

pub mod device_ids;
pub mod network;
pub mod version;

pub async fn init_all(
    client: &Client,
    config: &Arc<RwLock<GlobalConfig>>,
    events: &Arc<EventEmitter>,
) {
    network::reset_network_config(config).await;
    version::reset_version_config(config).await;
    device_ids::load_device_ids(config).await;
    network::load_network_config(client, config, events, None).await;
    version::load_version_config(client, config, events, None).await;
}

pub async fn reload_all(
    client: &Client,
    config: &Arc<RwLock<GlobalConfig>>,
    events: &Arc<EventEmitter>,
) {
    network::load_network_config(client, config, events, None).await;
    version::load_version_config(client, config, events, None).await;
}
