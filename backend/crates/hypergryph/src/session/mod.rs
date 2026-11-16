use std::fmt;

use myrtle_types::Server;
use serde::{Deserialize, Serialize};

use crate::config::network::NetworkConfig;
use crate::constants::Domain;
use crate::crypto::generate_u8_sign;
use crate::error::{RefreshStep, UpstreamError};
use crate::fetch::{FetchRequest, REDACTED, fetch_domain, parse_json};
use crate::login::bilibili::{self, OtpChallenge};
use crate::login::passport::{self, PassportCredential};
use crate::login::yostar::{
    AccountPortalSession, account_portal_login, request_token, submit_auth,
};
use crate::session::store::{DurableCredential, SessionKey, SessionStore};
use crate::upstream::Upstream;

pub mod account;
pub mod health;
pub mod store;

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSession {
    pub uid: Box<str>,
    pub secret: Box<str>,
    pub seqnum: u32,
    pub token: Box<str>,
    #[serde(default)]
    pub yostar_uid: Box<str>,
    #[serde(default)]
    pub yostar_token: Box<str>,
}

impl fmt::Debug for AuthSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthSession")
            .field("uid", &self.uid)
            .field("secret", &REDACTED)
            .field("seqnum", &self.seqnum)
            .field("token", &REDACTED)
            .field("yostar_uid", &self.yostar_uid)
            .field("yostar_token", &REDACTED)
            .finish()
    }
}

impl AuthSession {
    #[must_use]
    pub fn new(
        uid: Option<&str>,
        secret: Option<&str>,
        seqnum: Option<u32>,
        token: Option<&str>,
    ) -> Self {
        Self {
            uid: uid.unwrap_or_default().into(),
            secret: secret.unwrap_or_default().into(),
            seqnum: seqnum.unwrap_or(1),
            token: token.unwrap_or_default().into(),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn from_durable(uid: &str, credential: &DurableCredential) -> Self {
        Self {
            uid: uid.into(),
            yostar_uid: credential.yostar_uid.as_str().into(),
            yostar_token: credential.yostar_token.as_str().into(),
            ..Default::default()
        }
    }
}

pub struct LoginResult {
    pub session: AuthSession,
    pub yostar_email: String,
    pub portal_session: Option<AccountPortalSession>,
}

const ACCOUNT_LOGIN: &str = "session::account/login";
const GET_TOKEN: &str = "session::user/v1/getToken";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetSecretBody<'a> {
    platform: u32,
    network_version: &'a str,
    assets_version: &'a str,
    client_version: &'a str,
    token: &'a str,
    uid: &'a str,
    device_id: &'a str,
    device_id2: &'a str,
    device_id3: &'a str,
}

#[derive(Deserialize)]
struct GetSecretResponse {
    result: i32,
    #[serde(default)]
    secret: String,
}

#[derive(Deserialize)]
struct U8TokenResponse {
    result: i32,
    #[serde(default)]
    uid: String,
    #[serde(default)]
    token: String,
}

fn network_version(config: &NetworkConfig, server: Server) -> Result<String, UpstreamError> {
    if let Some(v) = config.network_version_override.as_deref()
        && !v.is_empty()
    {
        return Ok(v.to_owned());
    }
    Ok(match server {
        Server::CN | Server::Bilibili => "5",
        Server::EN | Server::JP | Server::KR => "1",
        Server::TW => {
            return Err(UpstreamError::Unsupported {
                server,
                what: "account/login",
            });
        }
    }
    .to_owned())
}

async fn get_secret(
    upstream: &Upstream,
    uid: &str,
    u8_token: &str,
    server: Server,
) -> Result<String, UpstreamError> {
    if upstream.config().version(server).res_version.is_empty() {
        upstream.reload_versions().await;
    }

    let first = match get_secret_once(upstream, uid, u8_token, server).await {
        Ok(secret) => return Ok(secret),
        Err(e) => e,
    };

    let before = upstream.config().version(server).clone();
    upstream.reload_versions().await;
    let after = upstream.config().version(server).clone();

    if after == before {
        tracing::warn!(
            uid,
            server = server.as_str(),
            res_version = %after.res_version,
            client_version = %after.client_version,
            error = ?first,
            "game-server login rejected with a current version config"
        );
        return Err(first);
    }

    tracing::info!(
        uid,
        server = server.as_str(),
        from_client_version = %before.client_version,
        to_client_version = %after.client_version,
        "game-server login rejected with a stale version config; retrying once"
    );
    get_secret_once(upstream, uid, u8_token, server).await
}

async fn get_secret_once(
    upstream: &Upstream,
    uid: &str,
    u8_token: &str,
    server: Server,
) -> Result<String, UpstreamError> {
    let config = upstream.config();
    let network_version = network_version(&config, server)?;
    let version = config.version(server);
    let ids = &config.device_ids;

    let body = serde_json::to_value(GetSecretBody {
        platform: 1,
        network_version: &network_version,
        assets_version: &version.res_version,
        client_version: &version.client_version,
        token: u8_token,
        uid,
        device_id: &ids.device_id,
        device_id2: &ids.device_id2,
        device_id3: &ids.device_id3,
    })
    .map_err(|e| UpstreamError::parse(ACCOUNT_LOGIN, e.to_string()))?;

    let session = AuthSession::new(Some(uid), Some(""), Some(1), None);
    let response = fetch_domain(
        upstream,
        Domain::GS,
        FetchRequest {
            endpoint: Some("account/login"),
            body: Some(&body),
            session: Some(&session),
            server,
            sign: true,
            timeout: upstream.request_timeout(),
        },
    )
    .await?;

    let data: GetSecretResponse = parse_json(response, ACCOUNT_LOGIN).await?;
    if data.result != 0 {
        return Err(UpstreamError::LoginRejected {
            context: ACCOUNT_LOGIN,
            code: data.result.to_string(),
            message: format!(
                "uid={uid} server={} networkVersion={network_version} clientVersion={} resVersion={}",
                server.as_str(),
                version.client_version,
                version.res_version
            ),
        });
    }
    if data.secret.is_empty() {
        return Err(UpstreamError::parse(
            ACCOUNT_LOGIN,
            format!("result=0 with no secret (uid={uid})"),
        ));
    }
    Ok(data.secret)
}

async fn get_u8_token(
    upstream: &Upstream,
    yostar_uid: &str,
    access_token: &str,
    server: Server,
) -> Result<U8TokenResponse, UpstreamError> {
    let channel_id = match server {
        Server::CN => "1",
        Server::Bilibili => "2",
        Server::EN | Server::JP | Server::KR => "3",
        Server::TW => {
            return Err(UpstreamError::Unsupported {
                server,
                what: "user/v1/getToken",
            });
        }
    };

    let extension = if channel_id == "3" {
        serde_json::json!({ "type": 1, "uid": yostar_uid, "token": access_token })
    } else {
        serde_json::json!({ "uid": yostar_uid, "access_token": access_token })
    }
    .to_string();

    let config = upstream.config();
    let ids = &config.device_ids;
    let sign = generate_u8_sign(&[
        ("appId", "1"),
        ("platform", "1"),
        ("channelId", channel_id),
        ("subChannel", channel_id),
        ("extension", &extension),
        ("worldId", channel_id),
        ("deviceId", &ids.device_id),
        ("deviceId2", &ids.device_id2),
        ("deviceId3", &ids.device_id3),
    ]);
    let body = serde_json::json!({
        "appId": "1",
        "platform": "1",
        "channelId": channel_id,
        "subChannel": channel_id,
        "extension": extension,
        "worldId": channel_id,
        "deviceId": ids.device_id,
        "deviceId2": ids.device_id2,
        "deviceId3": ids.device_id3,
        "sign": sign,
    });

    let response = fetch_domain(
        upstream,
        Domain::U8,
        FetchRequest {
            endpoint: Some("user/v1/getToken"),
            body: Some(&body),
            session: None,
            server,
            sign: true,
            timeout: upstream.request_timeout(),
        },
    )
    .await?;

    let data: U8TokenResponse = parse_json(response, GET_TOKEN).await?;
    if data.result != 0 || data.uid.is_empty() || data.token.is_empty() {
        return Err(UpstreamError::LoginRejected {
            context: GET_TOKEN,
            code: data.result.to_string(),
            message: format!("server={} channelId={channel_id}", server.as_str()),
        });
    }
    Ok(data)
}

async fn mint(
    upstream: &Upstream,
    channel_uid: String,
    channel_token: String,
    server: Server,
) -> Result<AuthSession, UpstreamError> {
    let u8_data = get_u8_token(upstream, &channel_uid, &channel_token, server).await?;
    let secret = get_secret(upstream, &u8_data.uid, &u8_data.token, server).await?;
    Ok(AuthSession {
        uid: u8_data.uid.into(),
        secret: secret.into(),
        seqnum: 1,
        token: u8_data.token.into(),
        yostar_uid: channel_uid.into(),
        yostar_token: channel_token.into(),
    })
}

pub async fn login(
    upstream: &Upstream,
    email: &str,
    code: &str,
    server: Server,
) -> Result<LoginResult, UpstreamError> {
    let yostar_data = submit_auth(upstream, email, code, server).await?;

    let portal_session = if upstream.config().yostar_sdk(server).is_some() {
        account_portal_login(upstream, email, &yostar_data.token)
            .await
            .ok()
    } else {
        None
    };

    let token_data = request_token(upstream, email, &yostar_data.token, server).await?;
    let session = mint(upstream, token_data.uid, token_data.token, server).await?;

    Ok(LoginResult {
        session,
        yostar_email: email.to_owned(),
        portal_session,
    })
}

pub async fn login_bilibili(
    upstream: &Upstream,
    username: &str,
    password: &str,
) -> Result<AuthSession, UpstreamError> {
    let bili = bilibili::login(upstream, username, password).await?;
    mint(upstream, bili.uid, bili.access_key, Server::Bilibili).await
}

pub async fn send_bilibili_sms(
    upstream: &Upstream,
    phone: &str,
    country_code: &str,
) -> Result<OtpChallenge, UpstreamError> {
    bilibili::send_otp(upstream, phone, country_code).await
}

pub async fn login_bilibili_sms(
    upstream: &Upstream,
    challenge: &OtpChallenge,
    sms_code: &str,
) -> Result<AuthSession, UpstreamError> {
    let bili = bilibili::login_otp(upstream, challenge, sms_code).await?;
    mint(upstream, bili.uid, bili.access_key, Server::Bilibili).await
}

pub async fn login_cn(
    upstream: &Upstream,
    credential: PassportCredential<'_>,
) -> Result<AuthSession, UpstreamError> {
    let passport_token = passport::login(upstream, credential).await?;
    let grant = passport::oauth2_grant(upstream, &passport_token).await?;
    mint(upstream, grant.uid, grant.code, Server::CN).await
}

pub async fn refresh_secret(
    upstream: &Upstream,
    session: &mut AuthSession,
    server: Server,
) -> Result<(), UpstreamError> {
    refresh_secret_by_step(upstream, session, server)
        .await
        .map_err(|(_, e)| e)
}

pub async fn refresh_secret_by_step(
    upstream: &Upstream,
    session: &mut AuthSession,
    server: Server,
) -> Result<(), (RefreshStep, UpstreamError)> {
    if session.yostar_uid.is_empty() || session.yostar_token.is_empty() {
        return Err((RefreshStep::Token, UpstreamError::NotLoggedIn));
    }

    let u8_data = get_u8_token(upstream, &session.yostar_uid, &session.yostar_token, server)
        .await
        .map_err(|e| (RefreshStep::Token, e))?;
    let secret = get_secret(upstream, &u8_data.uid, &u8_data.token, server)
        .await
        .map_err(|e| (RefreshStep::Login, e))?;

    session.uid = u8_data.uid.into();
    session.token = u8_data.token.into();
    session.secret = secret.into();
    session.seqnum = 1;
    Ok(())
}

pub async fn ensure_fresh(
    upstream: &Upstream,
    store: &dyn SessionStore,
    key: &SessionKey,
) -> Result<AuthSession, UpstreamError> {
    let cached = match store.load(key).await {
        Ok(cached) => cached,
        Err(e) => {
            tracing::warn!(
                error = e.kind(),
                "session cache unreadable; rebuilding from the credential"
            );
            None
        }
    };

    let mut session = match cached {
        Some(session) if !session.yostar_token.is_empty() => session,
        _ => {
            let credential = store
                .load_durable(key)
                .await?
                .ok_or(UpstreamError::NotLoggedIn)?;
            let uid = match key {
                SessionKey::Player { uid, .. } => uid.as_str(),
                SessionKey::ServiceAccount { .. } => "",
            };
            AuthSession::from_durable(uid, &credential)
        }
    };

    refresh_secret(upstream, &mut session, key.server()).await?;
    if let Err(e) = store.save(key, &session).await {
        tracing::warn!(error = e.kind(), "failed to save the refreshed session");
    }
    Ok(session)
}
