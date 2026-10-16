//! Cloudflare Turnstile on `POST /trevor/ask` (contract section 4).
//!
//! Skipped while `TURNSTILE_SECRET` is unset (the dev default). When it is
//! set, a missing or rejected token is a 403, and a siteverify that cannot be
//! reached fails CLOSED with a 503 "Trevor is busy". The verifier is a trait
//! object in [`super::runtime::TrevorRuntime`], so tests swap in a fake and
//! never touch the network.
use std::net::IpAddr;
use std::time::Duration;

use futures_util::future::BoxFuture;
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::state::AppState;

pub const SITEVERIFY_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";
/// siteverify answers in well under a second; past this it counts as down.
const SITEVERIFY_TIMEOUT: Duration = Duration::from_secs(5);

/// Checks one Turnstile token. `Ok(true)` passes, `Ok(false)` is a rejected
/// token, `Err` means the verdict could not be had (network, status, body).
pub trait TurnstileVerifier: Send + Sync {
    fn verify<'a>(
        &'a self,
        secret: &'a str,
        token: &'a str,
        remote_ip: Option<IpAddr>,
    ) -> BoxFuture<'a, Result<bool, String>>;
}

/// The real verifier: a form POST to Cloudflare's siteverify.
pub struct SiteVerify {
    client: reqwest::Client,
    url: String,
}

impl SiteVerify {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            client,
            url: SITEVERIFY_URL.to_owned(),
        }
    }
}

#[derive(Deserialize)]
struct SiteVerifyReply {
    success: bool,
}

/// `secret=..&response=..[&remoteip=..]`, form-encoded.
pub fn siteverify_form(secret: &str, token: &str, remote_ip: Option<IpAddr>) -> String {
    let mut body = format!(
        "secret={}&response={}",
        urlencoding::encode(secret),
        urlencoding::encode(token)
    );
    if let Some(ip) = remote_ip {
        body.push_str("&remoteip=");
        body.push_str(&urlencoding::encode(&ip.to_string()));
    }
    body
}

impl TurnstileVerifier for SiteVerify {
    fn verify<'a>(
        &'a self,
        secret: &'a str,
        token: &'a str,
        remote_ip: Option<IpAddr>,
    ) -> BoxFuture<'a, Result<bool, String>> {
        Box::pin(async move {
            let resp = self
                .client
                .post(&self.url)
                .header(
                    reqwest::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .body(siteverify_form(secret, token, remote_ip))
                .timeout(SITEVERIFY_TIMEOUT)
                .send()
                .await
                .map_err(|e| format!("siteverify unreachable: {e}"))?;
            if !resp.status().is_success() {
                return Err(format!("siteverify answered {}", resp.status()));
            }
            let reply: SiteVerifyReply = resp
                .json()
                .await
                .map_err(|e| format!("siteverify body unreadable: {e}"))?;
            Ok(reply.success)
        })
    }
}

/// The ask route's check: `Ok` to go on, or the error to answer with.
pub async fn check(
    state: &AppState,
    token: Option<&str>,
    remote_ip: Option<IpAddr>,
) -> Result<(), ApiError> {
    let cfg = state.trevor.config();
    let Some(secret) = cfg.turnstile_secret.as_deref() else {
        return Ok(());
    };
    let Some(token) = token.map(str::trim).filter(|t| !t.is_empty()) else {
        return Err(ApiError::Forbidden);
    };
    if token.len() > 2048 {
        return Err(ApiError::BadRequest("turnstileToken is too long".into()));
    }
    match state
        .trevor
        .verifier()
        .verify(secret, token, remote_ip)
        .await
    {
        Ok(true) => Ok(()),
        Ok(false) => Err(ApiError::Forbidden),
        Err(e) => {
            tracing::warn!(error = %e, "trevor: Turnstile verification failed closed");
            Err(ApiError::ServiceUnavailableMessage("Trevor is busy".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_form_carries_secret_response_and_remoteip_encoded() {
        let ip: IpAddr = "203.0.113.7".parse().unwrap();
        assert_eq!(
            siteverify_form("s&k", "t=1", Some(ip)),
            "secret=s%26k&response=t%3D1&remoteip=203.0.113.7"
        );
        assert_eq!(siteverify_form("s", "t", None), "secret=s&response=t");
    }
}
