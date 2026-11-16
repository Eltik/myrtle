use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::UpstreamError;
use crate::fetch::{read_text, redacted_body, shape_error};
use crate::upstream::Upstream;

#[derive(Deserialize)]
struct Envelope<T> {
    status: i32,
    #[serde(default)]
    msg: String,
    data: Option<T>,
}

#[derive(Deserialize)]
struct StatusOnly {
    status: i32,
    #[serde(default)]
    msg: String,
}

pub enum PassportCredential<'a> {
    Password { phone: &'a str, password: &'a str },
    SmsCode { phone: &'a str, code: &'a str },
}

#[derive(Deserialize)]
struct TokenData {
    token: String,
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct OAuthGrant {
    pub code: String,
    pub uid: String,
}

#[derive(Clone, Copy)]
struct Route {
    path: &'static str,
    context: &'static str,
}

const SEND_PHONE_CODE: Route = Route {
    path: "general/v1/send_phone_code",
    context: "passport::general/v1/send_phone_code",
};
const TOKEN_BY_PASSWORD: Route = Route {
    path: "user/auth/v1/token_by_phone_password",
    context: "passport::user/auth/v1/token_by_phone_password",
};
const TOKEN_BY_CODE: Route = Route {
    path: "user/auth/v2/token_by_phone_code",
    context: "passport::user/auth/v2/token_by_phone_code",
};
const GRANT: Route = Route {
    path: "user/oauth2/v2/grant",
    context: "passport::user/oauth2/v2/grant",
};

async fn post(
    upstream: &Upstream,
    route: Route,
    body: &serde_json::Value,
) -> Result<String, UpstreamError> {
    let base = upstream.config().hosts.passport.clone();
    let response = upstream
        .client()
        .post(format!("{base}/{}", route.path))
        .json(body)
        .timeout(upstream.request_timeout())
        .send()
        .await?;

    let status = response.status();
    let text = read_text(response, route.context).await?;
    if !status.is_success() {
        return Err(UpstreamError::parse(
            route.context,
            format!(
                "upstream non-success (status={status}): {}",
                redacted_body(&text)
            ),
        ));
    }
    Ok(text)
}

fn decode<T: DeserializeOwned>(route: Route, text: &str) -> Result<T, UpstreamError> {
    serde_json::from_str(text).map_err(|e| {
        tracing::warn!(context = route.context, body = %redacted_body(text), error = %shape_error(&e), "passport: failed to parse envelope");
        UpstreamError::parse(route.context, "invalid upstream response")
    })
}

fn refused(route: Route, status: i32, msg: String) -> UpstreamError {
    UpstreamError::LoginRejected {
        context: route.context,
        code: status.to_string(),
        message: msg,
    }
}

async fn post_status_only(
    upstream: &Upstream,
    route: Route,
    body: &serde_json::Value,
) -> Result<(), UpstreamError> {
    let text = post(upstream, route, body).await?;
    let parsed: StatusOnly = decode(route, &text)?;
    if parsed.status != 0 {
        return Err(refused(route, parsed.status, parsed.msg));
    }
    Ok(())
}

async fn post_json<T: DeserializeOwned>(
    upstream: &Upstream,
    route: Route,
    body: &serde_json::Value,
) -> Result<T, UpstreamError> {
    let text = post(upstream, route, body).await?;
    let envelope: Envelope<T> = decode(route, &text)?;
    if envelope.status != 0 {
        return Err(refused(route, envelope.status, envelope.msg));
    }
    envelope
        .data
        .ok_or_else(|| UpstreamError::parse(route.context, "missing data"))
}

pub async fn send_phone_code(upstream: &Upstream, phone: &str) -> Result<(), UpstreamError> {
    let body = serde_json::json!({ "phone": phone, "type": 2 });
    post_status_only(upstream, SEND_PHONE_CODE, &body).await
}

pub async fn login(
    upstream: &Upstream,
    credential: PassportCredential<'_>,
) -> Result<String, UpstreamError> {
    let (route, body) = match credential {
        PassportCredential::Password { phone, password } => (
            TOKEN_BY_PASSWORD,
            serde_json::json!({ "phone": phone, "password": password }),
        ),
        PassportCredential::SmsCode { phone, code } => (
            TOKEN_BY_CODE,
            serde_json::json!({ "phone": phone, "code": code }),
        ),
    };
    let data: TokenData = post_json(upstream, route, &body).await?;
    Ok(data.token)
}

pub async fn oauth2_grant(
    upstream: &Upstream,
    passport_token: &str,
) -> Result<OAuthGrant, UpstreamError> {
    let body = serde_json::json!({
        "token": passport_token,
        "appCode": upstream.config().cn_app_code,
        "type": 0,
    });
    post_json(upstream, GRANT, &body).await
}
