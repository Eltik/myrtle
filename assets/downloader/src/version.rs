use crate::server::Server;
use crate::types::VersionResponse;

pub async fn fetch_version(
    client: &reqwest::Client,
    server: Server,
) -> anyhow::Result<VersionResponse> {
    let resp = client
        .get(server.version_url())
        .send()
        .await?
        .json()
        .await?;
    Ok(resp)
}
