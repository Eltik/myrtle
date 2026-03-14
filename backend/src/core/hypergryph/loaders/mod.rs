use std::time::Duration;

use reqwest::Client;

use crate::core::hypergryph::config::{DeviceIds, config};

pub mod device_ids;
pub mod network;
pub mod version;

pub const CONFIG_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn init(client: &Client) {
    {
        let mut cfg = config().write().await;
        cfg.reset_network();
        cfg.reset_versions();
        cfg.device_ids = DeviceIds::generate();
    }

    network::load_network_config(client).await;
    version::load_version_config(client).await;

    // TODO: Emit event for when everything is initialized
}

pub async fn reload_all(client: &Client) {
    network::load_network_config(client).await;
    version::load_version_config(client).await;

    // TODO: Emit event for when everything is reloaded
}
