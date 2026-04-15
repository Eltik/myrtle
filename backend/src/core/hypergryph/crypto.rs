use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use serde::Serialize;
use sha1::Sha1;
use std::fmt::Write;

use crate::core::hypergryph::constants::{Pid, Server};

const SECRET: &str = "886c085e4a8d30a703367b120dd8353948405ec2";
const U8_SECRET: &[u8] = b"91240f70c09a08a6bc72af1a5c8d4670";

type HmacSha1 = Hmac<Sha1>;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct HeadersInner {
    #[serde(rename = "PID")]
    pid: Pid,
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

/// Generates the Authorization header value for game server requests.
pub fn generate_auth_header(
    body: &str,
    server: Server,
    yostar_id: Option<&str>,
    yostar_token: Option<&str>,
    device_id: &str,
) -> String {
    let lang = match server {
        Server::EN => "en",
        Server::JP => "jp",
        _ => "ko",
    };

    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let head = HeadersInner {
        pid: Pid::from(server),
        channel: "googleplay",
        platform: "android",
        version: "4.10.0",
        g_version_no: "2000112",
        g_build_no: "",
        lang,
        device_id: device_id.to_owned(),
        device_model: "F9",
        uid: yostar_id.unwrap_or_default().to_owned(),
        token: yostar_token.unwrap_or_default().to_owned(),
        time,
    };

    let json_string = serde_json::to_string(&head).unwrap();

    let mut hasher = Md5::new();
    hasher.update(format!("{json_string}{body}{SECRET}"));
    let sign = format!("{:X}", hasher.finalize());

    serde_json::to_string(&HeaderAuth { head, sign }).unwrap()
}

pub fn generate_u8_sign(data: &[(&str, &str)]) -> String {
    let mut entries: Vec<_> = data.iter().collect();
    entries.sort_unstable_by_key(|(k, _)| *k);

    let mut query = String::new();
    for (i, (k, v)) in entries.iter().enumerate() {
        if i > 0 {
            query.push('&');
        }
        write!(
            query,
            "{}={}",
            urlencoding::encode(k),
            urlencoding::encode(v)
        )
        .unwrap();
    }

    let mut mac = HmacSha1::new_from_slice(U8_SECRET).unwrap();
    mac.update(query.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
