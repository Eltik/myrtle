use std::{fmt, str::FromStr};

use crate::error::DownloaderError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Server {
    /// CN Official (Hypergryph)
    Official = 0,
    /// CN Bilibili
    Bilibili = 1,
    /// Global/EN (Yostar)
    En = 2,
    /// Japan (Yostar)
    Jp = 3,
    /// Korea (Yostar)
    Kr = 4,
    /// Taiwan (Gryphline)
    Tw = 5,
}

impl Server {
    /// URL to fetch version info
    pub fn version_url(&self) -> &'static str {
        match self {
            Self::Official => "https://ak-conf.hypergryph.com/config/prod/official/Android/version",
            Self::Bilibili => "https://ak-conf.hypergryph.com/config/prod/b/Android/version",
            Self::En => {
                "https://ark-us-static-online.yo-star.com/assetbundle/official/Android/version"
            }
            Self::Jp => {
                "https://ark-jp-static-online.yo-star.com/assetbundle/official/Android/version"
            }
            Self::Kr => {
                "https://ark-kr-static-online.yo-star.com/assetbundle/official/Android/version"
            }
            Self::Tw => {
                "https://ark-tw-static-online.yo-star.com/assetbundle/official/Android/version"
            }
        }
    }

    /// Base URL for asset downloads (append version + filename)
    pub fn cdn_base_url(&self) -> &'static str {
        match self {
            Self::Official => "https://ak.hycdn.cn/assetbundle/official/Android/assets",
            Self::Bilibili => "https://ak.hycdn.cn/assetbundle/bilibili/Android/assets",
            Self::En => {
                "https://ark-us-static-online.yo-star.com/assetbundle/official/Android/assets"
            }
            Self::Jp => {
                "https://ark-jp-static-online.yo-star.com/assetbundle/official/Android/assets"
            }
            Self::Kr => {
                "https://ark-kr-static-online.yo-star.com/assetbundle/official/Android/assets"
            }
            Self::Tw => {
                "https://ark-tw-static-online.yo-star.com/assetbundle/official/Android/assets"
            }
        }
    }
}

impl FromStr for Server {
    type Err = DownloaderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "official" | "cn" => Ok(Self::Official),
            "bilibili" | "bili" | "b" => Ok(Self::Bilibili),
            "en" | "global" => Ok(Self::En),
            "jp" | "japan" => Ok(Self::Jp),
            "kr" | "korea" => Ok(Self::Kr),
            "tw" | "taiwan" => Ok(Self::Tw),
            _ => Err(DownloaderError::InvalidServer(s.to_string())),
        }
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Official => write!(f, "official"),
            Self::Bilibili => write!(f, "bilibili"),
            Self::En => write!(f, "en"),
            Self::Jp => write!(f, "jp"),
            Self::Kr => write!(f, "kr"),
            Self::Tw => write!(f, "tw"),
        }
    }
}
