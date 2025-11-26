use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use serde::Serialize;
use sha1::Sha1;
use std::collections::HashMap;
use uuid::Uuid;

use crate::core::authentication::constants::Server;

const SECRET: &str = "886c085e4a8d30a703367b120dd8353948405ec2";
const U8_SECRET: &[u8] = b"91240f70c09a08a6bc72af1a5c8d4670";

type HmacSha1 = Hmac<Sha1>;

#[derive(Debug, Clone, Copy, Serialize)]
enum PID {
    #[serde(rename = "US-ARKNIGHTS")]
    UsArknights,
    #[serde(rename = "JP-AK")]
    JpAk,
    #[serde(rename = "KR-ARKNIGHTS")]
    KrArknights,
}

impl From<Server> for PID {
    fn from(server: Server) -> Self {
        match server {
            Server::EN => PID::UsArknights,
            Server::JP => PID::JpAk,
            Server::KR => PID::KrArknights,
            _ => PID::UsArknights,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct HeadersInner {
    #[serde(rename = "PID")]
    pid: PID,
    channel: &'static str,
    platform: &'static str,
    version: &'static str,
    #[serde(rename = "GVersionNo")]
    g_version_no: &'static str,
    #[serde(rename = "GBuildNo")]
    g_build_no: &'static str,
    lang: &'static str,
    #[serde(rename = "DeviceID")]
    device_id: String,
    device_model: &'static str,
    #[serde(rename = "UID")]
    uid: String,
    token: String,
    time: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct HeaderAuth {
    head: HeadersInner,
    sign: String,
}

pub fn generate_headers(
    body: &str,
    server: Server,
    yostar_id: Option<&str>,
    yostar_token: Option<&str>,
    device_id: Option<&str>,
) -> HashMap<&'static str, String> {
    let pid = if server == Server::EN {
        PID::UsArknights
    } else if server == Server::JP {
        PID::JpAk
    } else {
        PID::KrArknights
    };

    let lang = if server == Server::EN {
        "en"
    } else if server == Server::JP {
        "jp"
    } else {
        "ko"
    };

    let device_id: String = match device_id {
        Some(id) => id.to_string(),
        None => Uuid::new_v4().to_string(),
    };

    let uid = match yostar_id {
        Some(id) => id.to_string(),
        None => "".to_string(),
    };
    let token = yostar_token.unwrap_or("").to_string();

    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let headers_inner = HeadersInner {
        pid,
        channel: "googleplay",
        platform: "android",
        version: "4.10.0",
        g_version_no: "2000112",
        g_build_no: "",
        lang,
        device_id,
        device_model: "F9",
        uid,
        token,
        time,
    };

    // Serialize to JSON (compact, no spaces)
    let json_string = serde_json::to_string(&headers_inner).unwrap();

    // Create MD5 hash
    let mut hasher = Md5::new();
    hasher.update(format!("{}{}{}", json_string, body, SECRET));
    let hash = hasher.finalize();
    let sign = format!("{:X}", hash); // Uppercase hex

    // Create authorization header
    let header_auth = HeaderAuth {
        head: headers_inner,
        sign,
    };
    let authorization = serde_json::to_string(&header_auth).unwrap();

    // Return final headers
    HashMap::from([
        ("X-Unity-Version", "2017.4.39f1".to_string()),
        (
            "User-Agent",
            "Dalvik/2.1.0 (Linux; U; Android 11; KB2000 Build/RP1A.201005.001)".to_string(),
        ),
        ("Connection", "Keep-Alive".to_string()),
        ("Content-Type", "application/json".to_string()),
        ("Authorization", authorization),
    ])
}

pub fn generate_u8_sign(data: &HashMap<String, String>) -> String {
    // Sort entries by key
    let mut entries: Vec<_> = data.iter().collect();
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));

    // Create URL query string
    let query: String = entries
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    // Create HMAC-SHA1 hash
    let mut mac = HmacSha1::new_from_slice(U8_SECRET).unwrap();
    mac.update(query.as_bytes());
    let result = mac.finalize();

    // Return lowercase hex
    hex::encode(result.into_bytes())
}
