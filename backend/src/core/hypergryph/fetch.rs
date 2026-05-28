use std::time::Duration;

use reqwest::{Client, Response};
use serde::Deserialize;

use crate::core::hypergryph::{
    config::config,
    constants::{AuthSession, DEFAULT_HEADERS, Domain, Server},
    crypto::generate_auth_header,
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Deserialize)]
pub struct UpstreamError {
    #[serde(default, rename = "statusCode")]
    pub status_code: i64,
    #[serde(default)]
    pub code: i64,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub msg: String,
    #[serde(default)]
    pub info: String,
}

impl std::fmt::Display for UpstreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "upstream {} (code={}) {}: {} {}",
            self.status_code, self.code, self.error, self.msg, self.info
        )
    }
}

pub mod upstream_code {
    pub const STAGE_NO_REPLAY: i64 = 5516;
}

#[derive(Debug)]
pub enum FetchError {
    DomainNotFound(Server, Domain),
    RequestFailed(reqwest::Error),
    NotLoggedIn,
    ParseError(String),
    Upstream(UpstreamError),
}

impl From<reqwest::Error> for FetchError {
    fn from(e: reqwest::Error) -> Self {
        Self::RequestFailed(e)
    }
}

pub struct FetchRequest<'a> {
    pub endpoint: Option<&'a str>,
    pub body: Option<&'a serde_json::Value>,
    pub session: Option<&'a AuthSession>,
    pub server: Server,
    pub sign: bool,
}
pub async fn fetch(
    client: &Client,
    url: &str,
    req: FetchRequest<'_>,
) -> Result<Response, FetchError> {
    let full_url = match req.endpoint {
        Some(ep) => format!("{url}/{ep}"),
        None => url.to_owned(),
    };

    let has_body = req.body.is_some();
    let mut builder = if has_body {
        client.post(&full_url)
    } else {
        client.get(&full_url)
    };

    for &(k, v) in DEFAULT_HEADERS {
        builder = builder.header(k, v);
    }

    if let Some(sess) = req.session {
        builder = builder
            .header("uid", &*sess.uid)
            .header("secret", &*sess.secret)
            .header("seqnum", sess.seqnum.to_string());
    }

    if req.sign {
        let body_str = req
            .body
            .map(|b| serde_json::to_string(b).unwrap_or_default())
            .unwrap_or_default();

        let device_id = config().read().await.device_ids.device_id.clone();

        let auth = generate_auth_header(
            &body_str,
            req.server,
            req.session.map(|s| &*s.uid),
            req.session.map(|s| &*s.secret),
            &device_id,
        );
        builder = builder.header("Authorization", auth);
    }

    if let Some(b) = req.body {
        builder = builder.json(b);
    }

    Ok(builder.timeout(REQUEST_TIMEOUT).send().await?)
}

pub async fn fetch_domain(
    client: &Client,
    domain: Domain,
    req: FetchRequest<'_>,
) -> Result<Response, FetchError> {
    let raw_url = config()
        .read()
        .await
        .domain(req.server, domain)
        .map(|u| u.replace("{0}", "Android"))
        .ok_or(FetchError::DomainNotFound(req.server, domain))?;

    fetch(client, &raw_url, req).await
}

/// Read the response body. Non-2xx statuses are parsed as the Arknights error
/// envelope (`FetchError::Upstream`) or, failing that, surfaced as `ParseError`.
pub async fn read_body(response: Response, context: &str) -> Result<String, FetchError> {
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| FetchError::ParseError(format!("{context}: read body: {e}")))?;

    if status.is_success() {
        return Ok(text);
    }

    if let Ok(err) = serde_json::from_str::<UpstreamError>(&text) {
        tracing::warn!(context, %status, code = err.code, msg = %err.msg, "upstream error");
        return Err(FetchError::Upstream(err));
    }

    tracing::warn!(context, %status, body = %text, "non-success without error envelope");
    Err(FetchError::ParseError(format!(
        "{context}: upstream non-success (status={status})"
    )))
}

pub async fn parse_json<T: serde::de::DeserializeOwned>(
    response: Response,
    context: &str,
) -> Result<T, FetchError> {
    let text = read_body(response, context).await?;
    serde_json::from_str::<T>(&text).map_err(|e| {
        // Body and serde detail stay in logs; the surfaced error is intentionally generic
        // so we don't leak the upstream schema to API clients.
        tracing::warn!(context, body = %text, error = %e, "failed to parse response body");
        FetchError::ParseError(format!("{context}: invalid upstream response"))
    })
}

pub async fn auth_request(
    client: &Client,
    endpoint: &str,
    body: Option<&serde_json::Value>,
    session: &mut AuthSession,
    server: Server,
) -> Result<Response, FetchError> {
    if session.uid.is_empty() {
        return Err(FetchError::NotLoggedIn);
    }

    session.seqnum += 1;

    fetch_domain(
        client,
        Domain::GS,
        FetchRequest {
            endpoint: Some(endpoint),
            body,
            session: Some(session),
            server,
            sign: false,
        },
    )
    .await
}
