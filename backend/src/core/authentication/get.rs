#![allow(dead_code)]

use std::sync::Arc;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::core::authentication::{
    config::GlobalConfig,
    constants::{AuthSession, Domain, Server},
    fetch,
    generate::generate_u8_sign,
    loaders,
};
use crate::events::EventEmitter;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetSecretBody {
    platform: u32,
    network_version: &'static str,
    assets_version: String,
    client_version: String,
    token: String,
    uid: String,
    device_id: String,
    device_id2: String,
    device_id3: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetSecretResponse {
    result: i32,
    uid: String,
    secret: String,
    service_license_version: Option<i32>,
    major_version: Option<String>,
}

pub async fn get_secret(
    client: &Client,
    config: &Arc<RwLock<GlobalConfig>>,
    events: &Arc<EventEmitter>,
    uid: &str,
    u8_token: &str,
    server: Server,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let needs_version_load = {
        let config = config.read().await;
        config
            .versions
            .get(&server)
            .map(|v| v.res_version.is_empty())
            .unwrap_or(true)
    };
    if needs_version_load {
        loaders::version::load_version_config(client, config, events, Some(server)).await;
    }

    let network_version = match server {
        Server::CN | Server::Bilibili => "5",
        Server::EN | Server::JP | Server::KR => "1",
        Server::TW => return Err("TW server not supported".into()),
    };

    let body = {
        let config = config.read().await;

        let version_info = config
            .versions
            .get(&server)
            .ok_or("Version info not found")?;

        GetSecretBody {
            platform: 1,
            network_version,
            assets_version: version_info.res_version.clone(),
            client_version: version_info.client_version.clone(),
            token: u8_token.to_string(),
            uid: uid.to_string(),
            device_id: config.device_ids.device_id.clone(),
            device_id2: config.device_ids.device_id2.clone(),
            device_id3: config.device_ids.device_id3.clone(),
        }
    };

    // Create a session with uid, empty secret, and seqnum=1 for headers
    let session = AuthSession::new(Some(uid), Some(""), Some(1), None);

    let body_json = serde_json::to_value(&body)?;
    let response = fetch(
        client,
        config,
        Domain::GS,
        server,
        Some("account/login"),
        Some(body_json),
        Some(&session),
        true, // assign_headers for Authorization
    )
    .await
    .map_err(|e| format!("Get secret fetch error: {e:?}"))?;

    let data: GetSecretResponse = response.json().await?;
    if data.result != 0 {
        eprintln!("getSecret error: result={} for uid={}", data.result, uid);
        return Err(format!("getSecret failed with result: {}", data.result).into());
    }

    Ok(data.secret)
}

#[derive(Deserialize)]
pub struct U8TokenResponse {
    pub result: i32,
    pub error: Option<String>,
    pub uid: String,
    pub token: String,
}

pub async fn get_u8_token(
    client: &Client,
    config: &Arc<RwLock<GlobalConfig>>,
    yostar_uid: &str,
    access_token: &str,
    server: Server,
) -> Result<U8TokenResponse, Box<dyn std::error::Error + Send + Sync>> {
    let channel_id = match server {
        Server::CN => "1",
        Server::Bilibili => "2",
        Server::EN | Server::JP | Server::KR => "3",
        Server::TW => "ERROR",
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
    };

    let (device_ids, _u8_url) = {
        let config = config.read().await;
        let device_ids = config.device_ids.clone();
        let u8_url = config
            .domains
            .get(&server)
            .and_then(|d| d.get(&Domain::U8))
            .ok_or("U8 domain not found")?
            .clone();
        (device_ids, u8_url)
    };

    let mut body_map: HashMap<String, String> = HashMap::new();
    body_map.insert("appId".to_string(), "1".to_string());
    body_map.insert("platform".to_string(), "1".to_string());
    body_map.insert("channelId".to_string(), channel_id.to_string());
    body_map.insert("subChannel".to_string(), channel_id.to_string());
    body_map.insert("extension".to_string(), extension?);
    body_map.insert("worldId".to_string(), channel_id.to_string());
    body_map.insert("deviceId".to_string(), device_ids.device_id);
    body_map.insert("deviceId2".to_string(), device_ids.device_id2);
    body_map.insert("deviceId3".to_string(), device_ids.device_id3);

    let sign = generate_u8_sign(&body_map);
    body_map.insert("sign".to_string(), sign);

    let body_json = serde_json::to_value(&body_map)?;

    let response = fetch(
        client,
        config,
        Domain::U8,
        server,
        Some("user/v1/getToken"),
        Some(body_json),
        None,
        true,
    )
    .await
    .map_err(|e| format!("Get token fetch error: {e:?}"))?;

    let data: U8TokenResponse = response.json().await?;

    Ok(data)
}
