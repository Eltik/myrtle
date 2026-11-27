use myrtle_types::Server;
use serde::{Deserialize, Serialize};

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
    pub const ALL: [Self; 12] = [
        Self::GS,
        Self::AS,
        Self::U8,
        Self::HU,
        Self::HV,
        Self::RC,
        Self::AN,
        Self::PREAN,
        Self::SL,
        Self::OF,
        Self::PkgAd,
        Self::PkgIos,
    ];

    #[must_use]
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

    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::GS => "gs",
            Self::AS => "as",
            Self::U8 => "u8",
            Self::HU => "hu",
            Self::HV => "hv",
            Self::RC => "rc",
            Self::AN => "an",
            Self::PREAN => "prean",
            Self::SL => "sl",
            Self::OF => "of",
            Self::PkgAd => "pkgAd",
            Self::PkgIos => "pkgIOS",
        }
    }

    #[must_use]
    pub fn parse(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|d| d.key().eq_ignore_ascii_case(key))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
            Server::JP => Self::JpAk,
            Server::KR => Self::KrArknights,
            _ => Self::UsArknights,
        }
    }
}

pub(crate) const ACCOUNT_PORTAL_BASE: &str = "https://account.yo-star.com";
pub(crate) const PASSPORT_BASE: &str = "https://as.hypergryph.com";
pub(crate) const BILIBILI_BASE: &str =
    "https://line1-sdk-center-login-sh.biligame.net/api/external";

/// The Yostar SDK host, for the servers Yostar publishes
#[must_use]
pub const fn yostar_domain(server: Server) -> Option<&'static str> {
    match server {
        Server::EN => Some("https://en-sdk-api.yostarplat.com"),
        Server::JP | Server::KR => Some("https://jp-sdk-api.yostarplat.com"),
        Server::CN | Server::Bilibili | Server::TW => None,
    }
}

#[must_use]
pub const fn network_route(server: Server) -> &'static str {
    match server {
        Server::EN => "https://ak-conf.arknights.global/config/prod/official/network_config",
        Server::JP => "https://ak-conf.arknights.jp/config/prod/official/network_config",
        Server::KR => "https://ak-conf.arknights.kr/config/prod/official/network_config",
        Server::CN => "https://ak-conf.hypergryph.com/config/prod/official/network_config",
        Server::Bilibili => "https://ak-conf.hypergryph.com/config/prod/b/network_config",
        Server::TW => "https://ak-conf-tw.gryphline.com/config/prod/official/network_config",
    }
}
