use std::collections::HashMap;

use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use crate::core::hypergryph::{
    config::config,
    constants::{Domain, Server},
    loaders::CONFIG_TIMEOUT,
};

#[derive(Deserialize)]
struct NetworkConfigResponse {
    content: String,
}

#[derive(Deserialize)]
struct NetworkConfigContent {
    configs: HashMap<String, HashMap<String, Value>>,
    #[serde(rename = "funcVer")]
    func_ver: String,
}

async fn fetch_network(client: &Client, url: &str) -> Option<Vec<(Domain, String)>> {
    let response = client.get(url).timeout(CONFIG_TIMEOUT).send().await.ok()?;

    let outer: NetworkConfigResponse = response.json().await.ok()?;
    let inner: NetworkConfigContent = serde_json::from_str(&outer.content).ok()?;

    let network = inner
        .configs
        .get(&inner.func_ver)?
        .get("network")?
        .as_object()?;

    let domains = network
        .iter()
        .filter_map(|(k, v)| {
            let domain = parse_domain(k)?;
            let url = v.as_str()?;
            Some((domain, url.to_owned()))
        })
        .collect();

    Some(domains)
}

pub async fn load_network_config(client: &Client) {
    let mut set = tokio::task::JoinSet::new();
    for &server in Server::all() {
        let client = client.clone();
        set.spawn(async move { load_single_server(&client, server).await });
    }
    while set.join_next().await.is_some() {}
}

async fn load_single_server(client: &Client, server: Server) -> bool {
    let Some(url) = server.network_route() else {
        return false;
    };
    let Some(network) = fetch_network(client, url).await else {
        return false;
    };

    let mut cfg = config().write().await;
    for (domain, url) in network {
        cfg.set_domain(server, domain, url);
    }
    true
}

fn parse_domain(s: &str) -> Option<Domain> {
    serde_json::from_value(Value::String(s.to_lowercase())).ok()
}
