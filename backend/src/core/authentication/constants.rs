use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::LazyLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Distributor {
    Yostar,
    Hypergryph,
    Bilibili,
    Longcheng,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Server {
    EN,
    JP,
    KR,
    CN,
    #[serde(rename = "bili")]
    Bilibili,
    TW,
}

impl Server {
    /// Returns all server variants for iteration
    pub fn all() -> &'static [Server] {
        &[
            Server::EN,
            Server::JP,
            Server::KR,
            Server::CN,
            Server::Bilibili,
            Server::TW,
        ]
    }
}

impl Server {
    pub fn yostar_domain(&self) -> Option<&'static str> {
        match self {
            Server::EN => Some("https://en-sdk-api.yostarplat.com"),
            Server::JP | Server::KR => Some("https://jp-sdk-api.yostarplat.com"),
            Server::CN | Server::Bilibili | Server::TW => None, // Not supported
        }
    }

    pub fn network_route(&self) -> Option<&'static str> {
        match self {
            Server::EN => {
                Some("https://ak-conf.arknights.global/config/prod/official/network_config")
            }
            Server::JP => Some("https://ak-conf.arknights.jp/config/prod/official/network_config"),
            Server::KR => Some("https://ak-conf.arknights.kr/config/prod/official/network_config"),
            Server::CN => {
                Some("https://ak-conf.hypergryph.com/config/prod/official/network_config")
            }
            Server::Bilibili => Some("https://ak-conf.hypergryph.com/config/prod/b/network_config"),
            Server::TW => {
                Some("https://ak-conf-tw.gryphline.com/config/prod/official/network_config")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Domain {
    GS,
    AS,
    U8,
    HU,
    HV,
    RC,
    AN,
    PREAN,
    SL,
    OF,
    #[serde(rename = "pkgAd")]
    PkgAd,
    #[serde(rename = "pkgIOS")]
    PkgIos,
}

pub static DEFAULT_HEADERS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("Content-Type", "application/json"),
        ("X-Unity-Version", "2017.4.39f1"),
        (
            "User-Agent",
            "Dalvik/2.1.0 (Linux; U; Android 11; KB2000 Build/RP1A.201005.001)",
        ),
        ("Connection", "Keep-Alive"),
    ])
});

#[derive(Debug, Clone, Default)]
pub struct AuthSession {
    pub uid: String,
    pub secret: String,
    pub seqnum: u32,
    pub token: String,
}

impl AuthSession {
    pub fn new(
        uid: Option<&str>,
        secret: Option<&str>,
        seqnum: Option<u32>,
        token: Option<&str>,
    ) -> Self {
        Self {
            uid: uid.unwrap_or_default().to_string(),
            secret: secret.unwrap_or_default().to_string(),
            seqnum: seqnum.unwrap_or(1),
            token: token.unwrap_or_default().to_string(),
        }
    }
}

#[derive(Debug)]
pub enum FetchError {
    InvalidServer(Server),
    RequestFailed(reqwest::Error),
    Timeout,
    ParseError(String),
}
