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
    pub const fn all() -> &'static [Self] {
        &[
            Self::EN,
            Self::JP,
            Self::KR,
            Self::CN,
            Self::Bilibili,
            Self::TW,
        ]
    }
}

impl Server {
    pub const fn index(self) -> usize {
        match self {
            Self::EN => 0,
            Self::JP => 1,
            Self::KR => 2,
            Self::CN => 3,
            Self::Bilibili => 4,
            Self::TW => 5,
        }
    }

    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::EN => "English",
            Self::JP => "Japanese",
            Self::KR => "Korean",
            Self::CN => "Chinese",
            Self::Bilibili => "Bilibili",
            Self::TW => "Taiwanese",
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::EN => "en",
            Self::JP => "jp",
            Self::KR => "kr",
            Self::CN => "cn",
            Self::Bilibili => "bili",
            Self::TW => "tw",
        }
    }

    pub const fn yostar_domain(&self) -> Option<&'static str> {
        match self {
            Self::EN => Some("https://en-sdk-api.yostarplat.com"),
            Self::JP | Self::KR => Some("https://jp-sdk-api.yostarplat.com"),
            Self::CN | Self::Bilibili | Self::TW => None, // Not supported yet
        }
    }

    pub const fn network_route(&self) -> Option<&'static str> {
        match self {
            Self::EN => {
                Some("https://ak-conf.arknights.global/config/prod/official/network_config")
            }
            Self::JP => Some("https://ak-conf.arknights.jp/config/prod/official/network_config"),
            Self::KR => Some("https://ak-conf.arknights.kr/config/prod/official/network_config"),
            Self::CN => {
                Some("https://ak-conf.hypergryph.com/config/prod/official/network_config")
            }
            Self::Bilibili => Some("https://ak-conf.hypergryph.com/config/prod/b/network_config"),
            Self::TW => {
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
            Self::GS => 0,
            Self::AS => 1,
            Self::U8 => 2,
            Self::HU => 3,
            Self::HV => 4,
            Self::RC => 5,
            Self::AN => 6,
            Self::PREAN => 7,
            Self::SL => 8,
            Self::OF => 9,
            Self::PkgAd => 10,
            Self::PkgIos => 11,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Pid {
    #[serde(rename = "US-ARKNIGHTS")]
    UsArknights,
    #[serde(rename = "JP-AK")]
    JpAk,
    #[serde(rename = "KR-ARKNIGHTS")]
    KrArknights,
}

impl From<Server> for Pid {
    fn from(server: Server) -> Self {
        match server {
            Server::EN => Self::UsArknights,
            Server::JP => Self::JpAk,
            Server::KR => Self::KrArknights,
            _ => Self::UsArknights,
        }
    }
}
