use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::core::hypergryph::{
    bilibili,
    config::config,
    constants::{AuthSession, Domain, Server},
    crypto::generate_u8_sign,
    fetch::{FetchError, FetchRequest, REQUEST_TIMEOUT, fetch_domain, parse_json},
    loaders, passport,
    yostar::{AccountPortalSession, account_portal_login, request_token, submit_auth},
};

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

/// `uid` and `secret` exist ONLY on success. A rejection is just `{"result":4}`;
/// requiring them made serde fail first, so every rejection surfaced as
/// `missing field uid` with neither the real failure nor its code.
#[derive(Deserialize)]
#[allow(dead_code)]
struct GetSecretResponse {
    result: i32,
    #[serde(default)]
    uid: String,
    #[serde(default)]
    secret: String,
}

/// Sent to `account/login`. Yostar has moved it before, and a wrong value gets a
/// bare `result` code. Set `AK_NETWORK_VERSION` to probe another value without a
/// rebuild.
fn network_version_for(server: Server) -> Result<String, FetchError> {
    if let Ok(v) = std::env::var("AK_NETWORK_VERSION")
        && !v.is_empty()
    {
        return Ok(v);
    }
    Ok(match server {
        Server::CN | Server::Bilibili => "5",
        Server::EN | Server::JP | Server::KR => "1",
        Server::TW => return Err(FetchError::ParseError("TW server not supported".into())),
    }
    .to_owned())
}

/// Mints the game-server `secret` for a u8 session.
///
/// On a non-zero result, retries ONCE after refreshing the version config. A
/// stale `clientVersion`/`resVersion` is the one cause this process can fix:
/// versions are cached at startup, so a client update shipped while the backend
/// runs rejects every login until restart. When versions are already current,
/// another failure costs one extra request and reports the same error.
async fn get_secret(
    client: &Client,
    uid: &str,
    u8_token: &str,
    server: Server,
) -> Result<String, FetchError> {
    {
        let cfg = config().read().await;
        if cfg.version(server).res_version.is_empty() {
            drop(cfg);
            loaders::version::load_version_config(client).await;
        }
    }

    match get_secret_once(client, uid, u8_token, server).await {
        Ok(secret) => Ok(secret),
        Err(first) => {
            let before = config().read().await.version(server).clone();
            loaders::version::load_version_config(client).await;
            let after = config().read().await.version(server).clone();

            if after.res_version == before.res_version
                && after.client_version == before.client_version
            {
                tracing::warn!(
                    uid = %uid,
                    server = server.as_str(),
                    res_version = %after.res_version,
                    client_version = %after.client_version,
                    error = ?first,
                    "game-server login rejected and the version config was already current, so this is not a stale-version failure"
                );
                return Err(first);
            }

            tracing::info!(
                uid = %uid,
                server = server.as_str(),
                from_client_version = %before.client_version,
                to_client_version = %after.client_version,
                "game-server login rejected with a stale version config; refreshed and retrying once"
            );
            get_secret_once(client, uid, u8_token, server).await
        }
    }
}

async fn get_secret_once(
    client: &Client,
    uid: &str,
    u8_token: &str,
    server: Server,
) -> Result<String, FetchError> {
    let network_version = network_version_for(server)?;

    let (res_version, client_version, device_ids) = {
        let cfg = config().read().await;
        let v = cfg.version(server);
        (
            v.res_version.clone(),
            v.client_version.clone(),
            cfg.device_ids.clone(),
        )
    };

    let body = GetSecretBody {
        platform: 1,
        network_version: &network_version,
        assets_version: &res_version,
        client_version: &client_version,
        token: u8_token,
        uid,
        device_id: &device_ids.device_id,
        device_id2: &device_ids.device_id2,
        device_id3: &device_ids.device_id3,
    };

    let body_json =
        serde_json::to_value(&body).map_err(|e| FetchError::ParseError(e.to_string()))?;

    let session = AuthSession::new(Some(uid), Some(""), Some(1), None);

    let response = fetch_domain(
        client,
        Domain::GS,
        FetchRequest {
            endpoint: Some("account/login"),
            body: Some(&body_json),
            session: Some(&session),
            server,
            sign: true,
            timeout: REQUEST_TIMEOUT,
        },
    )
    .await?;

    let data: GetSecretResponse = parse_json(response, "get_secret").await?;

    if data.result != 0 {
        // Enough to tell a stale client from a rejected account, on one log line.
        return Err(FetchError::ParseError(format!(
            "getSecret failed: result={} uid={} server={} networkVersion={} clientVersion={} resVersion={}",
            data.result,
            uid,
            server.as_str(),
            network_version,
            client_version,
            res_version
        )));
    }

    if data.secret.is_empty() {
        return Err(FetchError::ParseError(format!(
            "getSecret returned result=0 with no secret (uid={uid})"
        )));
    }

    Ok(data.secret)
}

/// Like `GetSecretResponse`: `uid` and `token` are success-only, so they default
/// instead of failing the parse before the `result` check.
#[derive(Deserialize)]
#[allow(dead_code)]
struct U8TokenResponse {
    result: i32,
    #[serde(default)]
    uid: String,
    #[serde(default)]
    token: String,
}

async fn get_u8_token(
    client: &Client,
    yostar_uid: &str,
    access_token: &str,
    server: Server,
) -> Result<U8TokenResponse, FetchError> {
    let channel_id = match server {
        Server::CN => "1",
        Server::Bilibili => "2",
        Server::EN | Server::JP | Server::KR => "3",
        Server::TW => return Err(FetchError::ParseError("TW not supported".into())),
    };

    let extension = if channel_id == "3" {
        serde_json::to_string(&serde_json::json!({
            "type": 1,
            "uid": yostar_uid,
            "token": access_token
        }))
    } else {
        serde_json::to_string(&serde_json::json!({
            "uid": yostar_uid,
            "access_token": access_token
        }))
    }
    .map_err(|e| FetchError::ParseError(e.to_string()))?;

    let device_ids = config().read().await.device_ids.clone();

    let pairs = [
        ("appId", "1"),
        ("platform", "1"),
        ("channelId", channel_id),
        ("subChannel", channel_id),
        ("extension", &extension),
        ("worldId", channel_id),
        ("deviceId", &device_ids.device_id),
        ("deviceId2", &device_ids.device_id2),
        ("deviceId3", &device_ids.device_id3),
    ];
    let sign = generate_u8_sign(&pairs);

    let body_json = serde_json::json!({
        "appId": "1",
        "platform": "1",
        "channelId": channel_id,
        "subChannel": channel_id,
        "extension": extension,
        "worldId": channel_id,
        "deviceId": device_ids.device_id,
        "deviceId2": device_ids.device_id2,
        "deviceId3": device_ids.device_id3,
        "sign": sign,
    });

    let response = fetch_domain(
        client,
        Domain::U8,
        FetchRequest {
            endpoint: Some("user/v1/getToken"),
            body: Some(&body_json),
            session: None,
            server,
            sign: true,
            timeout: REQUEST_TIMEOUT,
        },
    )
    .await?;

    let data: U8TokenResponse = parse_json(response, "get_u8_token").await?;
    if data.result != 0 || data.uid.is_empty() || data.token.is_empty() {
        return Err(FetchError::ParseError(format!(
            "getToken failed: result={} server={} channelId={}",
            data.result,
            server.as_str(),
            channel_id
        )));
    }
    Ok(data)
}

/// Bilibili channel (`BiliGame` SDK) through the same u8/gs pipeline as Yostar,
/// with `channel_id = "2"`.
pub async fn login_bilibili(
    client: &Client,
    username: &str,
    password: &str,
) -> Result<AuthSession, FetchError> {
    let server = Server::Bilibili;

    let bili_data = bilibili::login(client, username, password).await?;
    let u8_data = get_u8_token(client, &bili_data.uid, &bili_data.access_key, server).await?;
    let secret = get_secret(client, &u8_data.uid, &u8_data.token, server).await?;

    Ok(AuthSession {
        uid: u8_data.uid.into(),
        secret: secret.into(),
        seqnum: 1,
        token: u8_data.token.into(),
        yostar_uid: bili_data.uid.into(),
        yostar_token: bili_data.access_key.into(),
    })
}

/// UNVERIFIED: the endpoint is a structural guess (see
/// `core::hypergryph::bilibili`).
pub async fn send_bilibili_sms(client: &Client, phone: &str) -> Result<(), FetchError> {
    bilibili::send_sms_code(client, phone).await
}

/// [`login_bilibili`] via phone + SMS code. UNVERIFIED: see
/// `core::hypergryph::bilibili`.
pub async fn login_bilibili_sms(
    client: &Client,
    phone: &str,
    sms_code: &str,
) -> Result<AuthSession, FetchError> {
    let server = Server::Bilibili;

    let bili_data = bilibili::login_sms(client, phone, sms_code).await?;
    let u8_data = get_u8_token(client, &bili_data.uid, &bili_data.access_key, server).await?;
    let secret = get_secret(client, &u8_data.uid, &u8_data.token, server).await?;

    Ok(AuthSession {
        uid: u8_data.uid.into(),
        secret: secret.into(),
        seqnum: 1,
        token: u8_data.token.into(),
        yostar_uid: bili_data.uid.into(),
        yostar_token: bili_data.access_key.into(),
    })
}

/// Experimental official CN login: passport login, oauth2 grant, then the shared
/// u8/gs pipeline with `channel_id = "1"`. The grant's appCode is an unverified
/// placeholder (`core::hypergryph::passport`), so expect failure until the real
/// game-client appCode is known.
pub async fn login_cn(
    client: &Client,
    credential: passport::PassportCredential<'_>,
) -> Result<AuthSession, FetchError> {
    let server = Server::CN;

    let passport_token = passport::login(client, credential).await?;
    let grant = passport::oauth2_grant(client, &passport_token).await?;

    let u8_data = get_u8_token(client, &grant.uid, &grant.code, server).await?;
    let secret = get_secret(client, &u8_data.uid, &u8_data.token, server).await?;

    Ok(AuthSession {
        uid: u8_data.uid.into(),
        secret: secret.into(),
        seqnum: 1,
        token: u8_data.token.into(),
        yostar_uid: grant.uid.into(),
        yostar_token: grant.code.into(),
    })
}

pub struct LoginResult {
    pub session: AuthSession,
    pub yostar_email: String,
    pub portal_session: Option<AccountPortalSession>,
}

pub async fn login(
    client: &Client,
    email: &str,
    code: &str,
    server: Server,
) -> Result<LoginResult, FetchError> {
    let yostar_data = submit_auth(client, email, code, server).await?;

    let portal_session = if server.yostar_domain().is_some() {
        account_portal_login(client, email, &yostar_data.token)
            .await
            .ok()
    } else {
        None
    };

    let token_data = request_token(client, email, &yostar_data.token, server).await?;

    let u8_data = get_u8_token(client, &token_data.uid, &token_data.token, server).await?;

    let secret = get_secret(client, &u8_data.uid, &u8_data.token, server).await?;

    let session = AuthSession {
        uid: u8_data.uid.into(),
        secret: secret.into(),
        seqnum: 1,
        token: u8_data.token.into(),
        yostar_uid: token_data.uid.into(),
        yostar_token: token_data.token.into(),
    };

    Ok(LoginResult {
        session,
        yostar_email: email.to_owned(),
        portal_session,
    })
}

/// Re-mint `secret` from the session's durable Yostar token, updating
/// `uid`/`token`/`secret` in place and resetting `seqnum`. One `secret` is live
/// per account, so rerunning u8 + `account/login` re-establishes the session.
/// [`FetchError::NotLoggedIn`] when the durable token is absent; callers should
/// ask for a re-login.
pub async fn refresh_secret(
    client: &Client,
    session: &mut AuthSession,
    server: Server,
) -> Result<(), FetchError> {
    if session.yostar_uid.is_empty() || session.yostar_token.is_empty() {
        return Err(FetchError::NotLoggedIn);
    }

    let u8_data = get_u8_token(client, &session.yostar_uid, &session.yostar_token, server).await?;
    let secret = get_secret(client, &u8_data.uid, &u8_data.token, server).await?;

    session.uid = u8_data.uid.into();
    session.token = u8_data.token.into();
    session.secret = secret.into();
    session.seqnum = 1;

    Ok(())
}
