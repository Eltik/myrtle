use crate::types::VersionResponse;

pub async fn fetch_version(client: &reqwest::Client, url: &str) -> anyhow::Result<VersionResponse> {
    let resp = client.get(url).send().await?.json().await?;
    Ok(resp)
}
