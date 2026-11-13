use std::time::Duration;

use reqwest::Client;

use self::device_ids::resolve_device_ids;
use self::network::{load_network_config, load_network_config_for, unloaded_servers};
use self::version::load_version_config;
use crate::core::hypergryph::config::config;
use crate::core::hypergryph::constants::Server;

pub mod device_ids;
pub mod network;
pub mod version;

pub const CONFIG_TIMEOUT: Duration = Duration::from_secs(5);

/// First wait before [`spawn_retry`] tries again, doubling up to [`RETRY_MAX`].
const RETRY_MIN: Duration = Duration::from_secs(15);
const RETRY_MAX: Duration = Duration::from_mins(10);

pub async fn init(client: &Client) {
    // Resolved before taking the lock to keep file I/O off the critical section.
    crate::core::startup::step("device ids");
    let device_ids = resolve_device_ids();
    {
        let mut cfg = config().write().await;
        cfg.reset_network();
        cfg.reset_versions();
        cfg.device_ids = device_ids;
    }

    // One fetch per server on a 5s timeout; an unreachable Hypergryph stalls this
    // step that long.
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

/// How long to wait before retry number `attempt` (0-based): [`RETRY_MIN`],
/// doubling, capped at [`RETRY_MAX`].
fn retry_delay(attempt: u32) -> Duration {
    RETRY_MIN
        .saturating_mul(2u32.saturating_pow(attempt))
        .min(RETRY_MAX)
}

/// Keep loading the network config of every server in `servers` that came up
/// without one, until all of them have it.
///
/// [`init`] makes one attempt per server on a 5s timeout and carries on, so that
/// an unreachable Hypergryph cannot hold the boot. Nothing used to make a second
/// attempt: on 2026-11-13 the backend restarted while its route to the config
/// hosts was stalling past that timeout, EN and JP came up with no domains, and
/// every login failed with `DomainNotFound(EN, U8)` for over two hours, until
/// someone restarted it. This task is that second attempt, and the third.
///
/// Only the configured servers are retried: [`init`] also tries the ones this
/// deployment does not serve, and those are allowed to stay missing. The task
/// ends once nothing is missing, because a loaded config is never unloaded
/// ([`reload`] only overwrites).
pub fn spawn_retry(client: Client, servers: Vec<Server>) {
    tokio::spawn(async move {
        let mut attempt = 0;
        loop {
            let missing = unloaded_servers(&servers).await;
            if missing.is_empty() {
                if attempt > 0 {
                    tracing::info!(
                        attempts = attempt,
                        "network config loaded for every configured server"
                    );
                }
                return;
            }

            let delay = retry_delay(attempt);
            let names: Vec<&str> = missing.iter().map(Server::as_str).collect();
            tracing::warn!(
                servers = ?names,
                retry_in_secs = delay.as_secs(),
                "network config missing; logins and syncs on these servers fail until it loads"
            );
            tokio::time::sleep(delay).await;

            load_network_config_for(&client, &missing).await;
            // The version check reads its URL from the network config, so the
            // servers that just loaded could not have had one either.
            load_version_config(&client).await;
            attempt += 1;
        }
    });
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{RETRY_MAX, RETRY_MIN, retry_delay};
    use crate::core::hypergryph::config::GlobalConfig;
    use crate::core::hypergryph::constants::{Domain, Server};

    #[test]
    fn retry_delay_doubles_from_the_minimum_and_stops_at_the_cap() {
        assert_eq!(retry_delay(0), RETRY_MIN);
        assert_eq!(retry_delay(1), RETRY_MIN * 2);
        assert_eq!(retry_delay(3), Duration::from_secs(120));
        assert_eq!(retry_delay(6), RETRY_MAX);
        assert_eq!(retry_delay(u32::MAX), RETRY_MAX);
    }

    #[test]
    fn a_server_has_its_network_once_any_domain_is_set() {
        let mut cfg = GlobalConfig::new();
        assert!(Server::all().iter().all(|&s| !cfg.has_network(s)));

        cfg.set_domain(Server::EN, Domain::U8, "https://as.example/u8".to_owned());
        assert!(cfg.has_network(Server::EN));
        assert!(!cfg.has_network(Server::JP));

        cfg.reset_network();
        assert!(!cfg.has_network(Server::EN));
    }
}
