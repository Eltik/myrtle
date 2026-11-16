use std::fmt::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use myrtle_types::Server;
use serde::Serialize;
use sha1::Sha1;

use crate::constants::Pid;

const SECRET: &str = "886c085e4a8d30a703367b120dd8353948405ec2";
const U8_SECRET: &[u8] = b"91240f70c09a08a6bc72af1a5c8d4670";
const BILIBILI_SALT: &str = "8783abfb533544c59e598cddc933d1bf";

type HmacSha1 = Hmac<Sha1>;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Head<'a> {
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
    device_id: &'a str,
    device_model: &'static str,
    #[serde(rename = "UID")]
    uid: &'a str,
    token: &'a str,
    time: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct HeaderAuth<'a> {
    head: Head<'a>,
    sign: String,
}

#[must_use]
pub fn generate_auth_header(
    body: &str,
    server: Server,
    yostar_id: Option<&str>,
    yostar_token: Option<&str>,
    device_id: &str,
) -> String {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    auth_header_at(body, server, yostar_id, yostar_token, device_id, time)
}

#[must_use]
pub fn auth_header_at(
    body: &str,
    server: Server,
    yostar_id: Option<&str>,
    yostar_token: Option<&str>,
    device_id: &str,
    time: i64,
) -> String {
    let lang = match server {
        Server::EN => "en",
        Server::JP => "jp",
        _ => "ko",
    };
    let head = Head {
        pid: Pid::from(server),
        channel: "googleplay",
        platform: "android",
        version: "4.10.0",
        g_version_no: "2000112",
        g_build_no: "",
        lang,
        device_id,
        device_model: "F9",
        uid: yostar_id.unwrap_or_default(),
        token: yostar_token.unwrap_or_default(),
        time,
    };
    let head_json = serde_json::to_string(&head).unwrap_or_default();

    let mut hasher = Md5::new();
    hasher.update(format!("{head_json}{body}{SECRET}"));
    let sign = format!("{:X}", hasher.finalize());

    serde_json::to_string(&HeaderAuth { head, sign }).unwrap_or_default()
}

#[must_use]
pub fn generate_u8_sign(data: &[(&str, &str)]) -> String {
    let mut entries: Vec<_> = data.iter().collect();
    entries.sort_unstable_by_key(|(k, _)| *k);

    let mut query = String::new();
    for (i, (k, v)) in entries.iter().enumerate() {
        if i > 0 {
            query.push('&');
        }
        let _ = write!(
            query,
            "{}={}",
            urlencoding::encode(k),
            urlencoding::encode(v)
        );
    }

    let mut mac = HmacSha1::new_from_slice(U8_SECRET).expect("hmac takes a key of any length");
    mac.update(query.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

#[must_use]
pub fn generate_bilibili_sign(data: &[(&str, &str)]) -> String {
    let mut entries = data.to_vec();
    entries.sort_unstable_by_key(|(k, _)| *k);

    let mut concatenated: String = entries.iter().map(|(_, v)| *v).collect();
    concatenated.push_str(BILIBILI_SALT);

    let mut hasher = Md5::new();
    hasher.update(concatenated.as_bytes());
    format!("{:x}", hasher.finalize())
}
