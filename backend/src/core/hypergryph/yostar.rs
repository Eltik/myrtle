use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::core::hypergryph::{
    constants::Server,
    fetch::{FetchError, FetchRequest, fetch},
};

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendCodeBody<'a> {
    account: &'a str,
    randstr: &'a str,
    ticket: &'a str,
}

pub async fn send_code(
    client: &Client,
    email: &str,
    server: Server,
) -> Result<serde_json::Value, FetchError> {
    let base_url = server.yostar_domain().ok_or(FetchError::ParseError(
        "Server not supported for Yostar".into(),
    ))?;

    let body = serde_json::to_value(SendCodeBody {
        account: email,
        randstr: "",
        ticket: "",
    })
    .map_err(|e| FetchError::ParseError(e.to_string()))?;

    let response = fetch(
        client,
        base_url,
        FetchRequest {
            endpoint: Some("yostar/send-code"),
            body: Some(&body),
            session: None,
            server,
            sign: true,
        },
    )
    .await?;

    response
        .json()
        .await
        .map_err(|e| FetchError::ParseError(e.to_string()))
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct AuthBody<'a> {
    account: &'a str,
    code: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AuthFetchResponse {
    code: i64,
    data: Option<AuthFetchData>,
    msg: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AuthFetchData {
    token: String,
}

pub struct AuthResponse {
    pub token: String,
}

pub async fn submit_auth(
    client: &Client,
    email: &str,
    code: &str,
    server: Server,
) -> Result<AuthResponse, FetchError> {
    let base_url = server.yostar_domain().ok_or(FetchError::ParseError(
        "Server not supported for Yostar".into(),
    ))?;

    let body = serde_json::to_value(AuthBody {
        account: email,
        code,
    })
    .map_err(|e| FetchError::ParseError(e.to_string()))?;

    let response = fetch(
        client,
        base_url,
        FetchRequest {
            endpoint: Some("yostar/get-auth"),
            body: Some(&body),
            session: None,
            server,
            sign: true,
        },
    )
    .await?;

    let data: AuthFetchResponse = response
        .json()
        .await
        .map_err(|e| FetchError::ParseError(e.to_string()))?;

    if data.code != 200 {
        return Err(FetchError::ParseError(format!(
            "Auth failed: {} (code: {})",
            data.msg, data.code
        )));
    }

    let inner = data
        .data
        .ok_or(FetchError::ParseError("Missing auth data".into()))?;

    Ok(AuthResponse { token: inner.token })
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct TokenBody<'a> {
    check_account: i32,
    geetest: Geetest,
    #[serde(rename = "OpenID")]
    open_id: &'a str,
    secret: &'a str,
    token: &'a str,
    #[serde(rename = "Type")]
    type_field: &'a str,
    user_name: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
#[derive(Default)]
struct Geetest {
    captcha_id: Option<String>,
    captcha_output: Option<String>,
    gen_time: Option<String>,
    lot_number: Option<String>,
    pass_token: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TokenFetchResponse {
    data: TokenData,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TokenData {
    user_info: TokenUserInfo,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TokenUserInfo {
    #[serde(rename = "ID")]
    id: String,
    token: String,
}

pub struct TokenResponse {
    pub uid: String,
    pub token: String,
}

pub async fn request_token(
    client: &Client,
    email: &str,
    email_token: &str,
    server: Server,
) -> Result<TokenResponse, FetchError> {
    let base_url = server.yostar_domain().ok_or(FetchError::ParseError(
        "Server not supported for Yostar".into(),
    ))?;

    let body = serde_json::to_value(TokenBody {
        check_account: 0,
        geetest: Geetest::default(),
        open_id: email,
        secret: "",
        token: email_token,
        type_field: "yostar",
        user_name: email,
    })
    .map_err(|e| FetchError::ParseError(e.to_string()))?;

    let response = fetch(
        client,
        base_url,
        FetchRequest {
            endpoint: Some("user/login"),
            body: Some(&body),
            session: None,
            server,
            sign: true,
        },
    )
    .await?;

    let data: TokenFetchResponse = response
        .json()
        .await
        .map_err(|e| FetchError::ParseError(e.to_string()))?;

    Ok(TokenResponse {
        uid: data.data.user_info.id,
        token: data.data.user_info.token,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PortalLoginBody<'a> {
    channel: &'a str,
    token: &'a str,
    #[serde(rename = "openId")]
    open_id: &'a str,
    account: &'a str,
    check_account: bool,
}

#[derive(Debug)]
pub struct AccountPortalSession {
    pub yssid: String,
    pub yssid_sig: String,
}

pub async fn account_portal_login(
    client: &Client,
    email: &str,
    auth_token: &str,
) -> Result<AccountPortalSession, FetchError> {
    let body = PortalLoginBody {
        channel: "yostar",
        token: auth_token,
        open_id: email,
        account: email,
        check_account: false,
    };

    let response = client
        .post("https://account.yo-star.com/api/user/login")
        .header("Content-Type", "application/json")
        .header("Lang", "en")
        .header("Accept", "application/json, text/plain, */*")
        .header("Origin", "https://account.yo-star.com")
        .header("Referer", "https://account.yo-star.com/login")
        .json(&body)
        .send()
        .await
        .map_err(FetchError::RequestFailed)?;

    let mut yssid = None;
    let mut yssid_sig = None;

    for value in response.headers().get_all("set-cookie") {
        let cookie = value.to_str().unwrap_or("");
        if let Some(val) = cookie.strip_prefix("YSSID=") {
            if !cookie.starts_with("YSSID.sig=") {
                yssid = val.split(';').next().map(str::to_owned);
            }
        } else if let Some(val) = cookie.strip_prefix("YSSID.sig=") {
            yssid_sig = val.split(';').next().map(str::to_owned);
        }
    }

    let _ = response.text().await;

    match (yssid, yssid_sig) {
        (Some(yssid), Some(yssid_sig)) => Ok(AccountPortalSession { yssid, yssid_sig }),
        _ => Err(FetchError::ParseError("Missing YSSID cookies".into())),
    }
}
