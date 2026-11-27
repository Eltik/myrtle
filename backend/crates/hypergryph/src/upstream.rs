use std::sync::{Arc, Weak};
use std::time::Duration;

use arc_swap::ArcSwap;
use myrtle_types::Server;
use reqwest::Client;
use tokio::task::{JoinHandle, JoinSet};

use crate::config::network::{NetworkConfig, VersionInfo, fetch_network, fetch_version};
use crate::constants::Domain;
use crate::fetch::REQUEST_TIMEOUT;
use crate::login::bilibili::random_bd_id;

const NETWORK_RETRY_MIN: Duration = Duration::from_secs(15);
const NETWORK_RETRY_MAX: Duration = Duration::from_mins(10);

pub struct Upstream {
    client: Client,
    config: ArcSwap<NetworkConfig>,
    bilibili_bd_id: String,
    request_timeout: Duration,
    network_retry: (Duration, Duration),
}

impl Upstream {
    #[must_use]
    pub fn new(client: Client, config: NetworkConfig) -> Self {
        Self {
            client,
            config: ArcSwap::from_pointee(config),
            bilibili_bd_id: random_bd_id(),
            request_timeout: REQUEST_TIMEOUT,
            network_retry: (NETWORK_RETRY_MIN, NETWORK_RETRY_MAX),
        }
    }

    #[must_use]
    pub const fn with_network_retry(mut self, first: Duration, max: Duration) -> Self {
        self.network_retry = (first, max);
        self
    }

    #[must_use]
    pub fn network_retry_delay(&self, attempt: u32) -> Duration {
        let (first, max) = self.network_retry;
        first.saturating_mul(2u32.saturating_pow(attempt)).min(max)
    }

    #[must_use]
    pub const fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    pub const fn request_timeout(&self) -> Duration {
        self.request_timeout
    }

    pub const fn client(&self) -> &Client {
        &self.client
    }

    pub fn config(&self) -> Arc<NetworkConfig> {
        self.config.load_full()
    }

    pub fn update(&self, edit: impl Fn(&mut NetworkConfig)) {
        self.config.rcu(|current| {
            let mut next = NetworkConfig::clone(current);
            edit(&mut next);
            next
        });
    }

    pub fn bilibili_bd_id(&self) -> &str {
        &self.bilibili_bd_id
    }

    pub async fn reload(&self) {
        self.reload_network().await;
        self.reload_versions().await;
    }

    pub async fn reload_network(&self) -> usize {
        self.reload_network_for(&Server::ALL).await
    }

    pub async fn reload_network_for(&self, servers: &[Server]) -> usize {
        let snapshot = self.config();
        let mut set = JoinSet::new();
        for &server in servers {
            let client = self.client.clone();
            let url = snapshot.network_route(server).to_owned();
            let timeout = self.request_timeout;
            set.spawn(async move {
                let fetched = fetch_network(&client, &url, timeout).await;
                (server, url, fetched)
            });
        }

        let mut loaded: Vec<(Server, Vec<(Domain, String)>)> = Vec::new();
        while let Some(joined) = set.join_next().await {
            match joined {
                Ok((server, _, Ok(routes))) => loaded.push((server, routes)),
                Ok((server, url, Err(e))) => {
                    tracing::warn!(server = server.as_str(), url, error = %e, "network_config load failed; this server's logins and syncs fail until it loads");
                }
                Err(e) => tracing::warn!(error = %e, "network_config task failed"),
            }
        }

        self.update(|next| {
            for (server, routes) in &loaded {
                for (domain, url) in routes {
                    next.set_domain(*server, *domain, url.clone());
                }
            }
        });
        loaded.len()
    }

    #[must_use]
    pub fn missing_networks(&self, servers: &[Server]) -> Vec<Server> {
        let config = self.config();
        servers
            .iter()
            .copied()
            .filter(|&server| !config.has_network(server))
            .collect()
    }

    pub fn spawn_network_retry(self: &Arc<Self>, servers: Vec<Server>) -> JoinHandle<()> {
        tokio::spawn(retry_networks(Arc::downgrade(self), servers))
    }

    pub async fn reload_versions(&self) -> usize {
        let snapshot = self.config();
        let mut set = JoinSet::new();
        for server in Server::ALL {
            let Some(url) = snapshot.version_url(server) else {
                continue;
            };
            let client = self.client.clone();
            let timeout = self.request_timeout;
            set.spawn(async move { (server, fetch_version(&client, &url, timeout).await) });
        }

        let mut loaded: Vec<(Server, VersionInfo)> = Vec::new();
        while let Some(joined) = set.join_next().await {
            match joined {
                Ok((server, Ok(info))) => loaded.push((server, info)),
                Ok((server, Err(e))) => {
                    tracing::debug!(server = server.as_str(), error = %e, "version fetch failed");
                }
                Err(e) => tracing::warn!(error = %e, "version task failed"),
            }
        }

        self.update(|next| {
            for (server, info) in &loaded {
                next.set_version(*server, info.clone());
            }
        });
        loaded.len()
    }
}

async fn retry_networks(upstream: Weak<Upstream>, servers: Vec<Server>) {
    let mut attempt = 0;
    loop {
        let Some(up) = upstream.upgrade() else { return };
        let missing = up.missing_networks(&servers);
        if missing.is_empty() {
            if attempt > 0 {
                tracing::info!(
                    attempts = attempt,
                    "network_config loaded for every configured server"
                );
            }
            return;
        }
        let delay = up.network_retry_delay(attempt);
        let names: Vec<&str> = missing.iter().copied().map(Server::as_str).collect();
        tracing::warn!(servers = ?names, retry_in_secs = delay.as_secs(), "network_config missing, retrying");
        drop(up);
        tokio::time::sleep(delay).await;

        let Some(up) = upstream.upgrade() else { return };
        if up.reload_network_for(&missing).await > 0 {
            up.reload_versions().await;
        }
        attempt += 1;
    }
}
