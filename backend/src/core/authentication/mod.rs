pub mod config;
pub mod constants;
pub mod generate;
pub mod loaders;
pub mod yostar;

use std::time::Duration;

use reqwest::{Client, Response};

use crate::core::authentication::constants::{AuthSession, DEFAULT_HEADERS, FetchError};

pub async fn fetch(
    client: &Client,
    url: &str,
    endpoint: Option<&str>,
    body: Option<serde_json::Value>,
    session: Option<&AuthSession>,
) -> Result<Response, FetchError> {
    let full_url = match endpoint {
        Some(ep) => format!("{}/{}", url, ep),
        None => url.to_string(),
    };

    let mut request = if body.is_some() {
        client.post(&full_url)
    } else {
        client.get(&full_url)
    };

    for (key, value) in DEFAULT_HEADERS.iter() {
        request = request.header(*key, *value)
    }

    if let Some(sess) = session {
        request = request
            .header("uid", &sess.uid)
            .header("seqnum", sess.seqnum.to_string())
            .header("secret", &sess.secret);
    }

    if let Some(b) = body {
        request = request.json(&b);
    }

    let response = request
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map_err(FetchError::RequestFailed)?;
    Ok(response)
}
