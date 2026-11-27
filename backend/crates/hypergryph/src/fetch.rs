use std::sync::LazyLock;
use std::time::Duration;

use myrtle_types::Server;
use regex::Regex;
use reqwest::Response;
use serde::de::DeserializeOwned;

use crate::constants::{DEFAULT_HEADERS, Domain};
use crate::crypto::generate_auth_header;
use crate::error::{UpstreamEnvelope, UpstreamError};
use crate::session::AuthSession;
use crate::upstream::Upstream;

pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_LOGGED_BODY_CHARS: usize = 300;
pub(crate) const REDACTED: &str = "<redacted>";

static SECRET_FIELD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)"([^"]*(?:token|secret|key|passw|pwd|ticket|cookie|sign|session|auth|cred|transcode)[^"]*)"\s*:\s*"[^"]*""#,
    )
    .expect("static redaction regex is valid")
});

pub struct FetchRequest<'a> {
    pub endpoint: Option<&'a str>,
    pub body: Option<&'a serde_json::Value>,
    pub session: Option<&'a AuthSession>,
    pub server: Server,
    pub sign: bool,
    pub timeout: Duration,
}

pub async fn fetch(
    upstream: &Upstream,
    url: &str,
    req: FetchRequest<'_>,
) -> Result<Response, UpstreamError> {
    let full_url = match req.endpoint {
        Some(ep) => format!("{url}/{ep}"),
        None => url.to_owned(),
    };

    let client = upstream.client();
    let mut builder = if req.body.is_some() {
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
        let body = req
            .body
            .map(|b| serde_json::to_string(b).unwrap_or_default())
            .unwrap_or_default();
        let auth = generate_auth_header(
            &body,
            req.server,
            req.session.map(|s| &*s.uid),
            req.session.map(|s| &*s.secret),
            &upstream.config().device_ids.device_id,
        );
        builder = builder.header("Authorization", auth);
    }

    if let Some(b) = req.body {
        builder = builder.json(b);
    }

    Ok(builder.timeout(req.timeout).send().await?)
}

pub async fn fetch_domain(
    upstream: &Upstream,
    domain: Domain,
    req: FetchRequest<'_>,
) -> Result<Response, UpstreamError> {
    let url = upstream
        .config()
        .domain(req.server, domain)
        .map(|u| u.replace("{0}", "Android"))
        .ok_or(UpstreamError::DomainNotFound {
            server: req.server,
            domain,
        })?;
    fetch(upstream, &url, req).await
}

pub async fn read_body(response: Response, context: &str) -> Result<String, UpstreamError> {
    read_body_tolerating(response, context, &[]).await
}

pub async fn read_body_tolerating(
    response: Response,
    context: &str,
    tolerated: &[i64],
) -> Result<String, UpstreamError> {
    let status = response.status();
    let text = read_text(response, context).await?;

    if status.is_success() {
        return Ok(text);
    }

    if let Ok(envelope) = serde_json::from_str::<UpstreamEnvelope>(&text) {
        if tolerated.contains(&envelope.code) {
            tracing::debug!(context, %status, code = envelope.code, msg = %envelope.msg, "upstream error (expected)");
        } else {
            tracing::warn!(context, %status, code = envelope.code, msg = %envelope.msg, "upstream error");
        }
        return Err(UpstreamError::from_envelope(envelope));
    }

    tracing::warn!(context, %status, body = %redacted_body(&text), "non-success without error envelope");
    Err(UpstreamError::parse(
        context,
        format!("upstream non-success (status={status})"),
    ))
}

pub async fn parse_json<T: DeserializeOwned>(
    response: Response,
    context: &str,
) -> Result<T, UpstreamError> {
    parse_json_tolerating(response, context, &[]).await
}

pub async fn parse_json_tolerating<T: DeserializeOwned>(
    response: Response,
    context: &str,
    tolerated: &[i64],
) -> Result<T, UpstreamError> {
    let text = read_body_tolerating(response, context, tolerated).await?;
    serde_json::from_str(&text).map_err(|e| {
        tracing::warn!(context, body = %redacted_body(&text), error = %shape_error(&e), "failed to parse response body");
        UpstreamError::parse(context, "invalid upstream response")
    })
}

pub async fn auth_request(
    upstream: &Upstream,
    endpoint: &str,
    body: Option<&serde_json::Value>,
    session: &mut AuthSession,
    server: Server,
) -> Result<Response, UpstreamError> {
    let timeout = upstream.request_timeout();
    auth_request_with_timeout(upstream, endpoint, body, session, server, timeout).await
}

pub async fn auth_request_with_timeout(
    upstream: &Upstream,
    endpoint: &str,
    body: Option<&serde_json::Value>,
    session: &mut AuthSession,
    server: Server,
    timeout: Duration,
) -> Result<Response, UpstreamError> {
    if session.uid.is_empty() {
        return Err(UpstreamError::NotLoggedIn);
    }

    session.seqnum += 1;

    fetch_domain(
        upstream,
        Domain::GS,
        FetchRequest {
            endpoint: Some(endpoint),
            body,
            session: Some(session),
            server,
            sign: false,
            timeout,
        },
    )
    .await
}

pub(crate) fn shape_error(e: &serde_json::Error) -> String {
    format!("{:?} at {}:{}", e.classify(), e.line(), e.column())
}

pub(crate) async fn read_bytes(
    response: Response,
    context: &str,
) -> Result<Vec<u8>, UpstreamError> {
    response.bytes().await.map(|b| b.to_vec()).map_err(|e| {
        if e.is_timeout() {
            UpstreamError::Timeout(e)
        } else {
            UpstreamError::parse(context, format!("read body: {e}"))
        }
    })
}

pub(crate) async fn read_text(response: Response, context: &str) -> Result<String, UpstreamError> {
    response.text().await.map_err(|e| {
        if e.is_timeout() {
            UpstreamError::Timeout(e)
        } else {
            UpstreamError::parse(context, format!("read body: {e}"))
        }
    })
}

/// Credential values blanked, cut to 300 chars
pub fn redacted_body(text: &str) -> String {
    let cleaned = SECRET_FIELD.replace_all(text, r#""${1}":"<redacted>""#);
    if cleaned.chars().count() <= MAX_LOGGED_BODY_CHARS {
        return cleaned.into_owned();
    }
    let head: String = cleaned.chars().take(MAX_LOGGED_BODY_CHARS).collect();
    format!("{head}… (+{} bytes)", cleaned.len() - head.len())
}
