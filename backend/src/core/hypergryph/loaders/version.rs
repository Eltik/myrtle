use reqwest::Client;

use crate::core::hypergryph::{
    config::{VersionInfo, config},
    constants::{Domain, Server},
    loaders::CONFIG_TIMEOUT,
};

pub async fn load_version_config(client: &Client) {
    let mut set = tokio::task::JoinSet::new();
    for &server in Server::all() {
        let client = client.clone();
        set.spawn(async move { load_single_server(&client, server).await });
    }
    while set.join_next().await.is_some() {}
}

async fn load_single_server(client: &Client, server: Server) -> bool {
    let hv_url = config()
        .read()
        .await
        .domain(server, Domain::HV)
        .map(str::to_owned);
    let Some(url) = hv_url else { return false };

    let url = url.replace("{0}", "Android");

    let Ok(response) = client.get(&url).timeout(CONFIG_TIMEOUT).send().await else {
        return false;
    };
    let Ok(version_info) = response.json::<VersionInfo>().await else {
        return false;
    };

    config().write().await.set_version(server, version_info);
    true
}
