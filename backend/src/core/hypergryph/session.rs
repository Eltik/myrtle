use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::core::hypergryph::{
    config::config,
    constants::{AuthSession, Domain, Server},
    crypto::generate_u8_sign,
    fetch::{FetchError, FetchRequest, fetch_domain},
    loaders,
    yostar::{AccountPortalSession, account_portal_login, request_token, submit_auth},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetSecretBody<'a> {
    platform: u32,
    network_version: &'static str,
    assets_version: &'a str,
    client_version: &'a str,
    token: &'a str,
    uid: &'a str,
    device_id: &'a str,
    device_id2: &'a str,
    device_id3: &'a str,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct GetSecretResponse {
    result: i32,
    uid: String,
    secret: String,
}

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

    let network_version = match server {
        Server::CN | Server::Bilibili => "5",
        Server::EN | Server::JP | Server::KR => "1",
        Server::TW => return Err(FetchError::ParseError("TW server not supported".into())),
    };

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
        network_version,
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
        },
    )
    .await?;

    let data: GetSecretResponse = response
        .json()
        .await
        .map_err(|e| FetchError::ParseError(e.to_string()))?;

    if data.result != 0 {
        return Err(FetchError::ParseError(format!(
            "getSecret failed: result={} uid={}",
            data.result, uid
        )));
    }

    Ok(data.secret)
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct U8TokenResponse {
    result: i32,
    uid: String,
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
        },
    )
    .await?;

    response
        .json()
        .await
        .map_err(|e| FetchError::ParseError(e.to_string()))
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
    };

    Ok(LoginResult {
        session,
        yostar_email: email.to_owned(),
        portal_session,
    })
}
