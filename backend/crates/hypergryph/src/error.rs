use std::fmt;

use myrtle_types::Server;
use serde::Deserialize;
use thiserror::Error;

use crate::constants::Domain;
use crate::session::store::SessionStoreError;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct UpstreamEnvelope {
    #[serde(default, rename = "statusCode")]
    pub status_code: i64,
    #[serde(default)]
    pub code: i64,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub msg: String,
    #[serde(default)]
    pub info: String,
}

impl UpstreamEnvelope {
    #[must_use]
    pub fn reads_as_maintenance(&self) -> bool {
        [&self.error, &self.message, &self.msg, &self.info]
            .iter()
            .any(|s| {
                let lower = s.to_lowercase();
                lower.contains("maintenance")
                    || lower.contains("维护")
                    || lower.contains("メンテナンス")
            })
    }
}

impl fmt::Display for UpstreamEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match (self.msg.is_empty(), self.message.is_empty()) {
            (false, false) => format!("{} {}", self.msg, self.message),
            (true, false) => self.message.clone(),
            _ => self.msg.clone(),
        };
        write!(
            f,
            "upstream {} (code={}) {}: {} {}",
            self.status_code, self.code, self.error, text, self.info
        )
    }
}

pub mod envelope_code {
    /// `getBattleReplay` for a stage with no saved replay
    pub const STAGE_NO_REPLAY: i64 = 5516;
    /// `templateShop/getGoodList` for a shop that does not exist or before session's first `syncData`
    pub const INVALID_SHOP_ID: i64 = 5681;
}

/// Which half of a secret refresh failed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshStep {
    /// u8 `getToken`
    Token,
    /// `account/login`
    Login,
}

#[derive(Debug, Error)]
pub enum UpstreamError {
    #[error("no {domain:?} domain for {server:?}")]
    DomainNotFound { server: Server, domain: Domain },
    #[error("transport: {0}")]
    Transport(#[source] reqwest::Error),
    #[error("timed out: {0}")]
    Timeout(#[source] reqwest::Error),
    #[error("{url}: HTTP {status}")]
    HttpStatus { url: String, status: u16 },
    #[error("{url}: code {code} {message}")]
    ApiCode {
        url: String,
        code: i64,
        message: String,
    },
    #[error("no game session")]
    NotLoggedIn,
    #[error("{context}: {message}")]
    Parse { context: String, message: String },
    #[error("{0}")]
    Envelope(Box<UpstreamEnvelope>),
    #[error("game maintenance: {0}")]
    Maintenance(Box<UpstreamEnvelope>),
    #[error("{context} rejected (code {code}): {message}")]
    LoginRejected {
        context: &'static str,
        code: String,
        message: String,
    },
    #[error(
        "account under risk control (code {code}): {message}; a solved captcha does not clear it, only time or another account does"
    )]
    RiskControl { code: String, message: String },
    #[error("{what} is not supported on {}", .server.as_str())]
    Unsupported { server: Server, what: &'static str },
    #[error("service account for {} refused; log it in again", .server.as_str())]
    ServiceAccountRefused {
        server: Server,
        first_of_outage: bool,
    },
    #[error("session store: {}", .0.kind())]
    Store(#[from] SessionStoreError),
}

impl From<reqwest::Error> for UpstreamError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            Self::Timeout(e)
        } else {
            Self::Transport(e)
        }
    }
}

impl UpstreamError {
    #[must_use]
    pub fn from_envelope(envelope: UpstreamEnvelope) -> Self {
        if envelope.reads_as_maintenance() {
            Self::Maintenance(Box::new(envelope))
        } else {
            Self::Envelope(Box::new(envelope))
        }
    }

    #[must_use]
    pub const fn is_envelope_code(&self, code: i64) -> bool {
        matches!(self, Self::Envelope(e) | Self::Maintenance(e) if e.code == code)
    }

    pub(crate) fn parse(context: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Parse {
            context: context.into(),
            message: message.into(),
        }
    }
}
