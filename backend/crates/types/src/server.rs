use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
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
    pub const ALL: [Self; 6] = [
        Self::EN,
        Self::JP,
        Self::KR,
        Self::CN,
        Self::Bilibili,
        Self::TW,
    ];

    /// `users.server_id`
    pub const fn index(self) -> i16 {
        match self {
            Self::EN => 0,
            Self::JP => 1,
            Self::KR => 2,
            Self::CN => 3,
            Self::Bilibili => 4,
            Self::TW => 5,
        }
    }

    pub fn from_index(index: i16) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.index() == index)
    }

    /// `en`, `jp`, `kr`, `cn`, `bili`, `tw`
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EN => "en",
            Self::JP => "jp",
            Self::KR => "kr",
            Self::CN => "cn",
            Self::Bilibili => "bili",
            Self::TW => "tw",
        }
    }

    /// Case-insensitive inverse of [`Self::as_str`]
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "en" => Some(Self::EN),
            "jp" => Some(Self::JP),
            "kr" => Some(Self::KR),
            "cn" => Some(Self::CN),
            "bili" | "bilibili" => Some(Self::Bilibili),
            "tw" => Some(Self::TW),
            _ => None,
        }
    }

    /// Bilibili shares CN's game data
    #[must_use]
    pub const fn data_server(self) -> Self {
        match self {
            Self::Bilibili => Self::CN,
            other => other
        }
    }
}
