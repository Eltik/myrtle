use serde::{Deserialize, Serialize};

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
    pub const fn index(self) -> usize {
        match self {
            Server::EN => 0,
            Server::JP => 1,
            Server::KR => 2,
            Server::CN => 3,
            Server::Bilibili => 4,
            Server::TW => 5,
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Server::EN => "en",
            Server::JP => "jp",
            Server::KR => "kr",
            Server::CN => "cn",
            Server::Bilibili => "bili",
            Server::TW => "tw",
        }
    }

    pub const fn yostar_domain(&self) -> Option<&'static str> {
        match self {
            Server::EN => Some("https://en-sdk-api.yostarplat.com"),
            Server::JP | Server::KR => Some("https://jp-sdk-api.yostarplat.com"),
            Server::CN | Server::Bilibili | Server::TW => None, // Not supported yet
        }
    }

    pub const fn network_route(&self) -> Option<&'static str> {
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

impl Domain {
    pub const fn index(self) -> usize {
        match self {
            Domain::GS => 0,
            Domain::AS => 1,
            Domain::U8 => 2,
            Domain::HU => 3,
            Domain::HV => 4,
            Domain::RC => 5,
            Domain::AN => 6,
            Domain::PREAN => 7,
            Domain::SL => 8,
            Domain::OF => 9,
            Domain::PkgAd => 10,
            Domain::PkgIos => 11,
        }
    }
}

pub const DEFAULT_HEADERS: &[(&str, &str)] = &[
    ("Content-Type", "application/json"),
    ("X-Unity-Version", "2017.4.39f1"),
    (
        "User-Agent",
        "Dalvik/2.1.0 (Linux; U; Android 11; KB2000 Build/RP1A.201005.001)",
    ),
    ("Connection", "Keep-Alive"),
];

#[derive(Debug, Clone, Default, Serialize)]
pub struct AuthSession {
    pub uid: Box<str>,
    pub secret: Box<str>,
    pub seqnum: u32,
    pub token: Box<str>,
}

impl AuthSession {
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
        }
    }
}
