#![allow(dead_code)]

use crate::core::authentication::{FetchError, constants::Server, fetch_url};
use reqwest::Client;
use serde::{Deserialize, Serialize};

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
    let body = SendCodeBody {
        account: email,
        randstr: "",
        ticket: "",
    };

    let base_url = server.yostar_domain().expect("Server not supported");

    let response = fetch_url(
        client,
        base_url,
        Some("yostar/send-code"),
        Some(serde_json::to_value(&body).unwrap()),
        None,
        server,
        true, // assign_headers for Authorization
    )
    .await?;

    let data: serde_json::Value = response.json().await.map_err(FetchError::RequestFailed)?;
    Ok(data)
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Geetest {
    captcha_id: Option<String>,
    captcha_output: Option<String>,
    gen_time: Option<String>,
    lot_number: Option<String>,
    pass_token: Option<String>,
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

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TokenUserInfo {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "UID2")]
    uid2: i64,
    #[serde(rename = "PID")]
    pid: String,
    token: String,
    birthday: String,
    reg_channel: String,
    trans_code: String,
    state: i64,
    #[serde(rename = "DeviceID")]
    device_id: String,
    created_at: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TokenYostar {
    #[serde(rename = "ID")]
    id: String,
    country: String,
    nickname: String,
    picture: String,
    state: i32,
    agree_ad: i32,
    created_at: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TokenData {
    age_verify_method: i64,
    is_new: i32,
    user_info: TokenUserInfo,
    yostar: TokenYostar,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TokenFetchResponse {
    code: i32,
    data: TokenData,
    msg: String,
}

#[derive(Deserialize)]
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
    let body = TokenBody {
        check_account: 0,
        geetest: Geetest {
            captcha_id: None,
            captcha_output: None,
            gen_time: None,
            lot_number: None,
            pass_token: None,
        },
        open_id: email,
        secret: "",
        token: email_token,
        type_field: "yostar",
        user_name: email,
    };

    let base_url = server.yostar_domain().expect("Server not supported");

    let response = fetch_url(
        client,
        base_url,
        Some("user/login"),
        Some(serde_json::to_value(&body).unwrap()),
        None,
        server,
        true, // assign_headers for Authorization
    )
    .await?;

    let token_response: TokenFetchResponse =
        response.json().await.map_err(FetchError::RequestFailed)?;

    let data = TokenResponse {
        uid: token_response.data.user_info.id,
        token: token_response.data.user_info.token,
    };
    Ok(data)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AuthBody<'a> {
    account: &'a str,
    code: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AuthFetchData {
    #[serde(rename = "UID")]
    uid: String,
    token: String,
    account: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AuthFetchResponse {
    code: i64,
    data: Option<AuthFetchData>,
    msg: String,
}

#[derive(Deserialize)]
pub struct AuthResponse {
    pub token: String,
}
pub async fn submit_auth(
    client: &Client,
    email: &str,
    code: &str,
    server: Server,
) -> Result<AuthResponse, FetchError> {
    let body = AuthBody {
        account: email,
        code,
    };

    let base_url = server.yostar_domain().expect("Server not supported");

    let response = fetch_url(
        client,
        base_url,
        Some("yostar/get-auth"),
        Some(serde_json::to_value(&body).unwrap()),
        None,
        server,
        true, // assign_headers for Authorization
    )
    .await?;

    let auth_response: AuthFetchResponse =
        response.json().await.map_err(FetchError::RequestFailed)?;

    if auth_response.code != 200 {
        return Err(FetchError::ParseError(format!(
            "Auth failed: {} (code: {})",
            auth_response.msg, auth_response.code
        )));
    }

    let data = auth_response
        .data
        .ok_or(FetchError::ParseError("Missing data".into()))?;

    Ok(AuthResponse { token: data.token })
}
