use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;
use myrtle_types::Server;
use reqwest::Client;
use tokio::task::JoinSet;

use crate::config::network::{NetworkConfig, VersionInfo, fetch_network, fetch_version};
use crate::constants::Domain;
use crate::fetch::REQUEST_TIMEOUT;
use crate::login::bilibili::random_bd_id;

pub struct Upstream {
    client: Client,
    config: ArcSwap<NetworkConfig>,
    bilibili_bd_id: String,
    request_timeout: Duration,
}

impl Upstream {
    #[must_use]
    pub fn new(client: Client, config: NetworkConfig) -> Self {
        Self {
            client,
            config: ArcSwap::from_pointee(config),
            bilibili_bd_id: random_bd_id(),
            request_timeout: REQUEST_TIMEOUT,
        }
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
        let snapshot = self.config();
        let mut set = JoinSet::new();
        for server in Server::ALL {
            let client = self.client.clone();
            let url = snapshot.network_route(server).to_owned();
            let timeout = self.request_timeout;
            set.spawn(async move { (server, fetch_network(&client, &url, timeout).await) });
        }

        let mut loaded: Vec<(Server, Vec<(Domain, String)>)> = Vec::new();
        while let Some(joined) = set.join_next().await {
            match joined {
                Ok((server, Ok(routes))) => loaded.push((server, routes)),
                Ok((server, Err(e))) => {
                    tracing::debug!(server = server.as_str(), error = %e, "network_config fetch failed");
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
