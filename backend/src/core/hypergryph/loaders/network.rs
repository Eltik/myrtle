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

async fn fetch_network(client: &Client, url: &str) -> Result<Vec<(Domain, String)>, String> {
    let response = client
        .get(url)
        .timeout(CONFIG_TIMEOUT)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP {status}"));
    }
    let body = response
        .text()
        .await
        .map_err(|e| format!("reading the response failed: {e}"))?;
    parse_network(&body)
}

/// The `(domain, url)` pairs of a `network_config` response, or what is wrong
/// with it. A response that parses but names no domain this backend knows is an
/// error too: loading it would leave the server exactly as unusable as not
/// loading anything.
fn parse_network(body: &str) -> Result<Vec<(Domain, String)>, String> {
    let outer: NetworkConfigResponse =
        serde_json::from_str(body).map_err(|e| format!("unexpected response: {e}"))?;
    let inner: NetworkConfigContent =
        serde_json::from_str(&outer.content).map_err(|e| format!("unexpected content: {e}"))?;

    let network = inner
        .configs
        .get(&inner.func_ver)
        .ok_or_else(|| format!("no config for funcVer {}", inner.func_ver))?
        .get("network")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("no network table under funcVer {}", inner.func_ver))?;

    let domains: Vec<(Domain, String)> = network
        .iter()
        .filter_map(|(k, v)| {
            let domain = parse_domain(k)?;
            let url = v.as_str()?;
            Some((domain, url.to_owned()))
        })
        .collect();

    if domains.is_empty() {
        return Err("network table names no known domain".to_owned());
    }
    Ok(domains)
}

pub async fn load_network_config(client: &Client) {
    load_network_config_for(client, Server::all()).await;
}

/// Load the network config of `servers`, each on its own request. A server that
/// fails keeps whatever it had and is logged with the reason: until
/// 2026-11-13 a failure here was silent, and one timed-out fetch at startup left
/// every EN login failing with `DomainNotFound` until the next restart.
pub async fn load_network_config_for(client: &Client, servers: &[Server]) {
    let mut set = tokio::task::JoinSet::new();
    for &server in servers {
        let client = client.clone();
        set.spawn(async move { load_single_server(&client, server).await });
    }
    while set.join_next().await.is_some() {}
}

/// The servers of `wanted` whose network config has not loaded.
pub async fn unloaded_servers(wanted: &[Server]) -> Vec<Server> {
    let cfg = config().read().await;
    wanted
        .iter()
        .copied()
        .filter(|&server| server.network_route().is_some() && !cfg.has_network(server))
        .collect()
}

async fn load_single_server(client: &Client, server: Server) -> bool {
    let Some(url) = server.network_route() else {
        return false;
    };
    let network = match fetch_network(client, url).await {
        Ok(network) => network,
        Err(error) => {
            tracing::warn!(
                server = server.as_str(),
                url,
                error,
                "network config load failed; requests that need this server's domains fail until it loads"
            );
            return false;
        }
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

#[cfg(test)]
mod tests {
    use super::parse_network;
    use crate::core::hypergryph::constants::Domain;

    /// A `network_config` response: the outer envelope with `content` as a JSON string.
    fn response(content: &str) -> String {
        serde_json::json!({ "sign": "x", "content": content }).to_string()
    }

    #[test]
    fn parses_the_domains_of_the_current_func_ver() {
        let body = response(
            r#"{"configVer":"5","funcVer":"V058","configs":{
                "V057":{"network":{"gs":"https://old.example/gs"}},
                "V058":{"override":true,"network":{
                    "gs":"https://gs.example:8443",
                    "as":"https://as.example",
                    "u8":"https://as.example/u8",
                    "hv":"https://conf.example/{0}/version",
                    "somethingNew":"https://ignored.example",
                    "notAString":7}}}}"#,
        );
        let mut domains = parse_network(&body).expect("parses");
        domains.sort_by(|a, b| a.1.cmp(&b.1));
        assert_eq!(
            domains,
            vec![
                (Domain::AS, "https://as.example".to_owned()),
                (Domain::U8, "https://as.example/u8".to_owned()),
                (Domain::HV, "https://conf.example/{0}/version".to_owned()),
                (Domain::GS, "https://gs.example:8443".to_owned()),
            ]
        );
    }

    #[test]
    fn says_what_is_wrong_with_a_bad_response() {
        let cases = [
            ("<html>blocked</html>".to_owned(), "unexpected response"),
            (response("not json"), "unexpected content"),
            (
                response(r#"{"funcVer":"V058","configs":{"V057":{"network":{}}}}"#),
                "no config for funcVer V058",
            ),
            (
                response(r#"{"funcVer":"V058","configs":{"V058":{"override":true}}}"#),
                "no network table under funcVer V058",
            ),
            (
                response(r#"{"funcVer":"V058","configs":{"V058":{"network":{"mystery":"x"}}}}"#),
                "names no known domain",
            ),
        ];
        for (body, expected) in cases {
            let error = parse_network(&body).expect_err("rejected");
            assert!(
                error.contains(expected),
                "{error:?} should mention {expected:?}"
            );
        }
    }
}
