use myrtle_types::Server;
use serde::{Deserialize, Serialize};

use crate::constants::ACCOUNT_PORTAL_BASE;
use crate::error::UpstreamError;
use crate::fetch::{FetchRequest, REDACTED, auth_request, fetch, parse_json, read_body};
use crate::session::AuthSession;
use crate::upstream::Upstream;

const SEND_CODE: &str = "yostar::yostar/send-code";
const GET_AUTH: &str = "yostar::yostar/get-auth";
const USER_LOGIN: &str = "yostar::user/login";
const PORTAL_LOGIN: &str = "yostar::api/user/login";
const SYNC_DATA: &str = "yostar::account/syncData";

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendCodeBody<'a> {
    account: &'a str,
    randstr: &'a str,
    ticket: &'a str,
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

#[derive(Default, Serialize)]
#[serde(rename_all = "PascalCase")]
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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountPortalSession {
    pub yssid: String,
    pub yssid_sig: String,
}

impl std::fmt::Debug for AccountPortalSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccountPortalSession")
            .field("yssid", &REDACTED)
            .field("yssid_sig", &REDACTED)
            .finish()
    }
}

fn sdk_base(upstream: &Upstream, server: Server) -> Result<String, UpstreamError> {
    upstream
        .config()
        .yostar_sdk(server)
        .map(str::to_owned)
        .ok_or(UpstreamError::Unsupported {
            server,
            what: "Yostar login",
        })
}

async fn post_sdk(
    upstream: &Upstream,
    server: Server,
    endpoint: &str,
    context: &'static str,
    body: &impl Serialize,
) -> Result<reqwest::Response, UpstreamError> {
    let base = sdk_base(upstream, server)?;
    let body =
        serde_json::to_value(body).map_err(|e| UpstreamError::parse(context, e.to_string()))?;
    fetch(
        upstream,
        &base,
        FetchRequest {
            endpoint: Some(endpoint),
            body: Some(&body),
            session: None,
            server,
            sign: true,
            timeout: upstream.request_timeout(),
        },
    )
    .await
}

pub async fn send_code(
    upstream: &Upstream,
    email: &str,
    server: Server,
) -> Result<serde_json::Value, UpstreamError> {
    let body = SendCodeBody {
        account: email,
        randstr: "",
        ticket: "",
    };
    let response = post_sdk(upstream, server, "yostar/send-code", SEND_CODE, &body).await?;
    parse_json(response, SEND_CODE).await
}

pub async fn submit_auth(
    upstream: &Upstream,
    email: &str,
    code: &str,
    server: Server,
) -> Result<AuthResponse, UpstreamError> {
    let body = AuthBody {
        account: email,
        code,
    };
    let response = post_sdk(upstream, server, "yostar/get-auth", GET_AUTH, &body).await?;
    let data: AuthFetchResponse = parse_json(response, GET_AUTH).await?;

    if data.code != 200 {
        return Err(UpstreamError::LoginRejected {
            context: GET_AUTH,
            code: data.code.to_string(),
            message: data.msg,
        });
    }
    let inner = data
        .data
        .ok_or_else(|| UpstreamError::parse(GET_AUTH, "missing auth data"))?;
    Ok(AuthResponse { token: inner.token })
}

pub async fn request_token(
    upstream: &Upstream,
    email: &str,
    email_token: &str,
    server: Server,
) -> Result<TokenResponse, UpstreamError> {
    let body = TokenBody {
        check_account: 0,
        geetest: Geetest::default(),
        open_id: email,
        secret: "",
        token: email_token,
        type_field: "yostar",
        user_name: email,
    };
    let response = post_sdk(upstream, server, "user/login", USER_LOGIN, &body).await?;
    let data: TokenFetchResponse = parse_json(response, USER_LOGIN).await?;
    Ok(TokenResponse {
        uid: data.data.user_info.id,
        token: data.data.user_info.token,
    })
}

/// The `YSSID` cookie pair for `account.yo-star.com`
pub async fn account_portal_login(
    upstream: &Upstream,
    email: &str,
    auth_token: &str,
) -> Result<AccountPortalSession, UpstreamError> {
    let body = PortalLoginBody {
        channel: "yostar",
        token: auth_token,
        open_id: email,
        account: email,
        check_account: false,
    };

    let base = upstream.config().hosts.account_portal.clone();
    let response = upstream
        .client()
        .post(format!("{base}/api/user/login"))
        .header("Content-Type", "application/json")
        .header("Lang", "en")
        .header("Accept", "application/json, text/plain, */*")
        .header("Origin", ACCOUNT_PORTAL_BASE)
        .header("Referer", format!("{ACCOUNT_PORTAL_BASE}/login"))
        .json(&body)
        .send()
        .await?;

    let mut yssid = None;
    let mut yssid_sig = None;
    for value in response.headers().get_all("set-cookie") {
        let cookie = value.to_str().unwrap_or("");
        if let Some(val) = cookie.strip_prefix("YSSID=") {
            yssid = val.split(';').next().map(str::to_owned);
        } else if let Some(val) = cookie.strip_prefix("YSSID.sig=") {
            yssid_sig = val.split(';').next().map(str::to_owned);
        }
    }
    let _ = response.text().await;

    match (yssid, yssid_sig) {
        (Some(yssid), Some(yssid_sig)) => Ok(AccountPortalSession { yssid, yssid_sig }),
        _ => Err(UpstreamError::parse(PORTAL_LOGIN, "missing YSSID cookies")),
    }
}

pub async fn sync_data_raw(
    upstream: &Upstream,
    session: &mut AuthSession,
    server: Server,
) -> Result<String, UpstreamError> {
    let body = serde_json::json!({ "platform": 1 });
    let response = auth_request(upstream, "account/syncData", Some(&body), session, server).await?;
    read_body(response, SYNC_DATA).await
}

pub async fn sync_data(
    upstream: &Upstream,
    session: &mut AuthSession,
    server: Server,
) -> Result<serde_json::Value, UpstreamError> {
    let text = sync_data_raw(upstream, session, server).await?;
    serde_json::from_str(&text).map_err(|e| UpstreamError::parse(SYNC_DATA, e.to_string()))
}
