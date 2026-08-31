use std::time::Duration;

use reqwest::Client;

use self::device_ids::resolve_device_ids;
use self::network::load_network_config;
use self::version::load_version_config;
use crate::core::hypergryph::config::config;

pub mod device_ids;
pub mod network;
pub mod version;

pub const CONFIG_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn init(client: &Client) {
    // Resolve (load-or-generate) before taking the lock to keep file I/O off the
    // critical section. Persisted so the device identity survives restarts.
    crate::core::startup::step("device ids");
    let device_ids = resolve_device_ids();
    {
        let mut cfg = config().write().await;
        cfg.reset_network();
        cfg.reset_versions();
        cfg.device_ids = device_ids;
    }

    // One fetch per server on a 5s timeout, so an unreachable Hypergryph stalls
    // this step for the full timeout.
    crate::core::startup::step("network config");
    load_network_config(client).await;
    crate::core::startup::step("version config");
    load_version_config(client).await;

    // TODO: Emit event for when everything is initialized
}

pub async fn reload(client: &Client) {
    load_network_config(client).await;
    load_version_config(client).await;

    // TODO: Emit event for when everything is reloaded
}
