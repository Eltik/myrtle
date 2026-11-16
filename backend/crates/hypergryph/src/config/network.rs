use std::collections::HashMap;
use std::time::Duration;

use myrtle_types::Server;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::device_ids::DeviceIds;
use crate::constants::{
    ACCOUNT_PORTAL_BASE, BILIBILI_BASE, Domain, PASSPORT_BASE, network_route, yostar_domain,
};
use crate::error::UpstreamError;
use crate::fetch::read_bytes;

/// Skland's appCode
pub const DEFAULT_CN_APP_CODE: &str = "4ca99fa6b56cc2ba";

const SERVERS: usize = Server::ALL.len();

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub res_version: String,
    pub client_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hosts {
    pub network_routes: [String; SERVERS],
    pub yostar_sdk: [Option<String>; SERVERS],
    pub account_portal: String,
    pub passport: String,
    pub bilibili: String,
}

impl Default for Hosts {
    fn default() -> Self {
        Self {
            network_routes: Server::ALL.map(|s| network_route(s).to_owned()),
            yostar_sdk: Server::ALL.map(|s| yostar_domain(s).map(str::to_owned)),
            account_portal: ACCOUNT_PORTAL_BASE.to_owned(),
            passport: PASSPORT_BASE.to_owned(),
            bilibili: BILIBILI_BASE.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkConfig {
    pub device_ids: DeviceIds,
    pub hosts: Hosts,
    /// Sent to `account/login` in place of per-server value
    pub network_version_override: Option<String>,
    pub cn_app_code: String,
    domains: [[Option<String>; Domain::ALL.len()]; SERVERS],
    versions: [VersionInfo; SERVERS],
}

impl NetworkConfig {
    #[must_use]
    pub fn new(device_ids: DeviceIds) -> Self {
        Self {
            device_ids,
            hosts: Hosts::default(),
            network_version_override: None,
            cn_app_code: DEFAULT_CN_APP_CODE.to_owned(),
            domains: Default::default(),
            versions: Default::default(),
        }
    }

    #[must_use]
    pub fn domain(&self, server: Server, domain: Domain) -> Option<&str> {
        self.domains[server_slot(server)][domain.index()].as_deref()
    }

    pub fn set_domain(&mut self, server: Server, domain: Domain, url: impl Into<String>) {
        self.domains[server_slot(server)][domain.index()] = Some(url.into());
    }

    #[must_use]
    pub const fn version(&self, server: Server) -> &VersionInfo {
        &self.versions[server_slot(server)]
    }

    pub fn set_version(&mut self, server: Server, info: VersionInfo) {
        self.versions[server_slot(server)] = info;
    }

    #[must_use]
    pub fn network_route(&self, server: Server) -> &str {
        &self.hosts.network_routes[server_slot(server)]
    }

    #[must_use]
    pub fn yostar_sdk(&self, server: Server) -> Option<&str> {
        self.hosts.yostar_sdk[server_slot(server)].as_deref()
    }

    #[must_use]
    pub fn version_url(&self, server: Server) -> Option<String> {
        self.domain(server, Domain::HV)
            .map(|u| u.replace("{0}", "Android"))
    }
}

const fn server_slot(server: Server) -> usize {
    server.index() as usize
}

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

const NETWORK_CONFIG: &str = "network_config::network_config";
const VERSION: &str = "network_config::version";

/// The routes under a `network_config` body's current `funcVer`
pub fn parse_network(body: &[u8]) -> Result<Vec<(Domain, String)>, UpstreamError> {
    let outer: NetworkConfigResponse = serde_json::from_slice(body)
        .map_err(|e| UpstreamError::parse(NETWORK_CONFIG, e.to_string()))?;
    let inner: NetworkConfigContent = serde_json::from_str(&outer.content)
        .map_err(|e| UpstreamError::parse(NETWORK_CONFIG, format!("content: {e}")))?;

    let network = inner
        .configs
        .get(&inner.func_ver)
        .and_then(|c| c.get("network"))
        .and_then(Value::as_object)
        .ok_or_else(|| UpstreamError::parse(NETWORK_CONFIG, "no network for funcVer"))?;

    Ok(network
        .iter()
        .filter_map(|(k, v)| Some((Domain::parse(k)?, v.as_str()?.to_owned())))
        .collect())
}

pub(crate) async fn fetch_network(
    client: &Client,
    url: &str,
    timeout: Duration,
) -> Result<Vec<(Domain, String)>, UpstreamError> {
    let response = client.get(url).timeout(timeout).send().await?;
    parse_network(&read_bytes(response, NETWORK_CONFIG).await?)
}

pub(crate) async fn fetch_version(
    client: &Client,
    url: &str,
    timeout: Duration,
) -> Result<VersionInfo, UpstreamError> {
    let response = client.get(url).timeout(timeout).send().await?;
    serde_json::from_slice(&read_bytes(response, VERSION).await?)
        .map_err(|e| UpstreamError::parse(VERSION, e.to_string()))
}
