//! Experimental: official CN (Hypergryph) account login.
//!
//! No open-source project has a verified implementation of the CN game client's
//! login. What IS documented (ArkMowers/arknights-mower, MIT; the archived
//! `ProbiusOfficial/Skland_API` docs) is the passport login the Skland app uses,
//! on the same account system (`as.hypergryph.com`) the client almost certainly
//! shares.
//!
//! Hypothesis: the client runs the same phone/password or phone/SMS login and
//! the same `user/oauth2/v2/grant` exchange, with its own `appCode`. That code
//! hasn't surfaced; [`app_code`] defaults to Skland's as a placeholder,
//! overridable via `HYPERGRYPH_CN_APP_CODE` once captured (e.g. MITM on a rooted
//! device, see djpadbit/Arknights-RE). Until then expect [`oauth2_grant`] to be
//! rejected or scoped to the wrong app, and `session::login_cn`'s u8 exchange to
//! fail. Shipped so testing it costs one constant.
//!
//! The introspection endpoint (`user/info/v1/basic`, not called here) returns
//! real-name PII (ID card number, legal name), since CN mandates real-name
//! binding. Treat a passport token as sensitive for that reason.

use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, de::DeserializeOwned};

use crate::core::hypergryph::fetch::FetchError;
use crate::utils::redact::redacted_body;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const BASE_URL: &str = "https://as.hypergryph.com";

/// Skland's own appCode, used as an unverified placeholder. See module docs.
const DEFAULT_APP_CODE: &str = "4ca99fa6b56cc2ba";

fn app_code() -> String {
    std::env::var("HYPERGRYPH_CN_APP_CODE").unwrap_or_else(|_| DEFAULT_APP_CODE.to_owned())
}

#[derive(Deserialize)]
struct Envelope<T> {
    status: i32,
    #[serde(default)]
    msg: String,
    // No #[serde(default)]: a missing Option<T> is already None, and the attribute
    // makes serde_derive infer a spurious `T: Default` bound.
    data: Option<T>,
}

#[derive(Deserialize)]
struct StatusOnly {
    status: i32,
    #[serde(default)]
    msg: String,
}

async fn post(client: &Client, path: &str, body: &serde_json::Value) -> Result<String, FetchError> {
    let response = client
        .post(format!("{BASE_URL}/{path}"))
        .json(body)
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| FetchError::ParseError(format!("passport::{path}: read body: {e}")))?;

    if !status.is_success() {
        return Err(FetchError::ParseError(format!(
            "passport::{path}: upstream non-success (status={status}): {text}"
        )));
    }

    Ok(text)
}

async fn post_status_only(
    client: &Client,
    path: &str,
    body: &serde_json::Value,
) -> Result<(), FetchError> {
    let text = post(client, path, body).await?;

    let parsed: StatusOnly = serde_json::from_str(&text).map_err(|e| {
        tracing::warn!(path, body = %redacted_body(&text), error = %e, "passport: failed to parse status envelope");
        FetchError::ParseError(format!("passport::{path}: invalid upstream response"))
    })?;

    if parsed.status != 0 {
        return Err(FetchError::ParseError(format!(
            "passport::{path}: status={} msg={}",
            parsed.status, parsed.msg
        )));
    }

    Ok(())
}

async fn post_json<T: DeserializeOwned>(
    client: &Client,
    path: &str,
    body: &serde_json::Value,
) -> Result<T, FetchError> {
    let text = post(client, path, body).await?;

    let envelope: Envelope<T> = serde_json::from_str(&text).map_err(|e| {
        tracing::warn!(path, body = %redacted_body(&text), error = %e, "passport: failed to parse envelope");
        FetchError::ParseError(format!("passport::{path}: invalid upstream response"))
    })?;

    if envelope.status != 0 {
        return Err(FetchError::ParseError(format!(
            "passport::{path}: status={} msg={}",
            envelope.status, envelope.msg
        )));
    }

    envelope
        .data
        .ok_or_else(|| FetchError::ParseError(format!("passport::{path}: missing data")))
}

/// Requests an SMS code for `phone`, ahead of [`PassportCredential::SmsCode`].
pub async fn send_phone_code(client: &Client, phone: &str) -> Result<(), FetchError> {
    let body = serde_json::json!({ "phone": phone, "type": 2 });
    post_status_only(client, "general/v1/send_phone_code", &body).await
}

pub enum PassportCredential<'a> {
    Password { phone: &'a str, password: &'a str },
    SmsCode { phone: &'a str, code: &'a str },
}

#[derive(Deserialize)]
struct TokenData {
    token: String,
}

/// Phone + password or phone + SMS code -> passport token.
pub async fn login(
    client: &Client,
    credential: PassportCredential<'_>,
) -> Result<String, FetchError> {
    let (path, body) = match credential {
        PassportCredential::Password { phone, password } => (
            "user/auth/v1/token_by_phone_password",
            serde_json::json!({ "phone": phone, "password": password }),
        ),
        PassportCredential::SmsCode { phone, code } => (
            "user/auth/v2/token_by_phone_code",
            serde_json::json!({ "phone": phone, "code": code }),
        ),
    };

    let data: TokenData = post_json(client, path, &body).await?;
    Ok(data.token)
}

#[derive(Deserialize)]
pub struct OAuthGrant {
    pub code: String,
    pub uid: String,
}

/// Passport token -> app-scoped `OAuth2` grant. The appCode is an unverified
/// placeholder (see module docs).
pub async fn oauth2_grant(client: &Client, passport_token: &str) -> Result<OAuthGrant, FetchError> {
    let body = serde_json::json!({
        "token": passport_token,
        "appCode": app_code(),
        "type": 0,
    });

    post_json(client, "user/oauth2/v2/grant", &body).await
}
