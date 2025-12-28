pub mod config;
pub mod constants;
pub mod generate;
pub mod get;
pub mod jwt;
pub mod loaders;
pub mod login;
pub mod yostar;

use std::{sync::Arc, time::Duration};

use reqwest::{Client, Response};
use tokio::sync::RwLock;

use crate::core::authentication::{
    config::GlobalConfig,
    constants::{AuthSession, DEFAULT_HEADERS, Domain, FetchError, Server},
    generate::generate_headers,
};

#[allow(clippy::too_many_arguments)]
pub async fn fetch(
    client: &Client,
    config: &Arc<RwLock<GlobalConfig>>,
    domain: Domain,
    server: Server,
    endpoint: Option<&str>,
    body: Option<serde_json::Value>,
    session: Option<&AuthSession>,
    assign_headers: bool,
) -> Result<Response, FetchError> {
    let mut url = {
        let config = config.read().await;
        config
            .domains
            .get(&server)
            .and_then(|d| d.get(&domain))
            .ok_or(FetchError::InvalidServer(server))?
            .clone()
    };

    if url.contains("{0}") {
        url = url.replace("{0}", "Android");
    }

    if let Some(ep) = endpoint {
        url = format!("{url}/{ep}");
    }

    let mut request = if body.is_some() {
        client.post(&url)
    } else {
        client.get(&url)
    };

    for (key, value) in DEFAULT_HEADERS.iter() {
        request = request.header(*key, *value);
    }

    if let Some(sess) = session {
        request = request
            .header("uid", &sess.uid)
            .header("seqnum", sess.seqnum.to_string())
            .header("secret", &sess.secret);
    }

    if assign_headers {
        let body_str = body
            .as_ref()
            .map(|b| serde_json::to_string(b).unwrap_or_default())
            .unwrap_or_default();

        let device_id = {
            let config = config.read().await;
            Some(config.device_ids.device_id.clone())
        };

        let headers = generate_headers(
            &body_str,
            server,
            session.map(|s| s.uid.as_str()),
            session.map(|s| s.secret.as_str()),
            device_id.as_deref(),
        );

        for (key, value) in headers {
            request = request.header(key, value);
        }
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

/// Authenticated request to the game server (GS domain).
/// Automatically increments session seqnum before each request.
pub async fn auth_request(
    client: &Client,
    config: &Arc<RwLock<GlobalConfig>>,
    endpoint: &str,
    body: Option<serde_json::Value>,
    session: &mut AuthSession,
    server: Server,
) -> Result<Response, FetchError> {
    // Validate session
    if session.uid.is_empty() {
        return Err(FetchError::ParseError("Not logged in.".into()));
    }

    // Increment seqnum before request
    session.seqnum += 1;

    // Use fetch with GS domain, no Authorization headers (assign_headers = false)
    fetch(
        client,
        config,
        Domain::GS,
        server,
        Some(endpoint),
        body,
        Some(session),
        false, // Don't generate Authorization header, just use session headers
    )
    .await
}

pub async fn fetch_url(
    client: &Client,
    url: &str,
    endpoint: Option<&str>,
    body: Option<serde_json::Value>,
    session: Option<&AuthSession>,
    server: Server,
    assign_headers: bool,
) -> Result<Response, FetchError> {
    let full_url = match endpoint {
        Some(ep) => format!("{url}/{ep}"),
        None => url.to_string(),
    };

    let mut request = if body.is_some() {
        client.post(&full_url)
    } else {
        client.get(&full_url)
    };

    for (key, value) in DEFAULT_HEADERS.iter() {
        request = request.header(*key, *value);
    }

    if let Some(sess) = session {
        request = request
            .header("uid", &sess.uid)
            .header("seqnum", sess.seqnum.to_string())
            .header("secret", &sess.secret);
    }

    if assign_headers {
        let body_str = body
            .as_ref()
            .map(|b| serde_json::to_string(b).unwrap_or_default())
            .unwrap_or_default();

        let headers = generate_headers(
            &body_str,
            server,
            session.map(|s| s.uid.as_str()),
            session.map(|s| s.secret.as_str()),
            None, // device_id - will generate random UUID
        );

        for (key, value) in headers {
            request = request.header(key, value);
        }
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
