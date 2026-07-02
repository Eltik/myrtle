use crate::types::VersionResponse;

/// # Errors
///
/// Returns an error if the HTTP request fails or the response body cannot be
/// deserialized into a `VersionResponse`.
pub async fn fetch_version(client: &reqwest::Client, url: &str) -> anyhow::Result<VersionResponse> {
    let resp = client.get(url).send().await?.json().await?;
    Ok(resp)
}
