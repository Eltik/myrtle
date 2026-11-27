use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use rand::Rng;
use rand::rand_core::UnwrapErr;
use rand::rngs::SysRng;
use reqwest::Url;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use super::error::ProtocolError;

pub const CODE_TTL_SECS: i64 = 5 * 60;
pub const ACCESS_TTL_SECS: i64 = 7 * 24 * 60 * 60;
pub const REFRESH_TTL_SECS: i64 = 90 * 24 * 60 * 60;
pub const MAX_APPS_PER_USER: i64 = 10;
pub const MAX_REDIRECT_URIS: usize = 10;
pub const MAX_SESSIONS_PER_GRANT: i64 = 20;
pub const MAX_NAME_LEN: usize = 64;
pub const MAX_DESCRIPTION_LEN: usize = 280;
pub const MAX_REDIRECT_URI_LEN: usize = 512;
pub const MAX_STATE_LEN: usize = 1024;

const DEFAULT_API_BASE: &str = "https://api.myrtle.moe";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretKind {
    ClientSecret,
    Code,
    AccessToken,
    RefreshToken,
}

impl SecretKind {
    #[must_use]
    pub const fn prefix(self) -> &'static str {
        match self {
            Self::ClientSecret => "myrtle_cs_",
            Self::Code => "myrtle_ac_",
            Self::AccessToken => "myrtle_at_",
            Self::RefreshToken => "myrtle_rt_",
        }
    }
}

#[must_use]
pub fn mint(kind: SecretKind) -> String {
    let mut bytes = [0u8; 32];
    UnwrapErr(SysRng).fill_bytes(&mut bytes);
    format!("{}{}", kind.prefix(), URL_SAFE_NO_PAD.encode(bytes))
}

#[must_use]
pub fn digest(secret: &str) -> Vec<u8> {
    Sha256::digest(secret.as_bytes()).to_vec()
}

#[must_use]
pub fn matches_digest(presented: &str, stored: &[u8]) -> bool {
    digest(presented).ct_eq(stored).into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scope {
    Identify,
}

impl Scope {
    pub const ALL: [Self; 1] = [Self::Identify];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Identify => "identify",
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.as_str() == raw)
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub fn parse_scopes(raw: Option<&str>) -> Result<Vec<Scope>, String> {
    let mut scopes = Vec::new();
    for name in raw.unwrap_or_default().split_ascii_whitespace() {
        let scope = Scope::parse(name).ok_or_else(|| name.to_owned())?;
        if !scopes.contains(&scope) {
            scopes.push(scope);
        }
    }
    if scopes.is_empty() {
        scopes.push(Scope::Identify);
    }
    scopes.sort_unstable();
    Ok(scopes)
}

#[must_use]
pub fn join_scopes(scopes: &[Scope]) -> String {
    scopes
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

#[must_use]
pub fn stored_scopes(stored: &str) -> Vec<Scope> {
    stored
        .split_ascii_whitespace()
        .filter_map(Scope::parse)
        .collect()
}

#[must_use]
pub fn scope_names(stored: &str) -> Vec<String> {
    stored_scopes(stored)
        .iter()
        .map(ToString::to_string)
        .collect()
}

pub fn validate_redirect_uri(uri: &str) -> Result<(), String> {
    if uri.len() > MAX_REDIRECT_URI_LEN {
        return Err(format!("is longer than {MAX_REDIRECT_URI_LEN} characters"));
    }
    if uri.trim() != uri || uri.chars().any(char::is_control) {
        return Err("must not contain whitespace or control characters".to_owned());
    }
    let url = Url::parse(uri).map_err(|_| "is not an absolute URL".to_owned())?;
    let Some(host) = url.host_str() else {
        return Err("has no host".to_owned());
    };
    match url.scheme() {
        "https" => {}
        "http" if is_loopback_host(host) => {}
        "http" => {
            return Err(
                "must use https (http is allowed only for localhost, 127.0.0.1 and [::1])"
                    .to_owned(),
            );
        }
        _ => return Err("must be an https URL".to_owned()),
    }
    if url.fragment().is_some() {
        return Err("must not contain a fragment (#...)".to_owned());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("must not contain credentials".to_owned());
    }
    Ok(())
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "[::1]")
}

#[must_use]
pub fn redirect_with(redirect_uri: &str, params: &[(&str, &str)]) -> Option<String> {
    let mut url = Url::parse(redirect_uri).ok()?;
    url.query_pairs_mut().extend_pairs(params);
    Some(url.into())
}

#[must_use]
pub fn redirect_host(redirect_uri: &str) -> Option<String> {
    let url = Url::parse(redirect_uri).ok()?;
    let host = url.host_str()?;
    Some(match url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.to_owned(),
    })
}

#[must_use]
pub fn is_valid_challenge(challenge: &str) -> bool {
    challenge.len() == 43 && challenge.bytes().all(is_base64url)
}

#[must_use]
pub fn is_valid_verifier(verifier: &str) -> bool {
    (43..=128).contains(&verifier.len())
        && verifier
            .bytes()
            .all(|b| is_base64url(b) || b == b'.' || b == b'~')
}

const fn is_base64url(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

#[must_use]
pub fn verify_pkce(verifier: &str, challenge: &str) -> bool {
    if !is_valid_verifier(verifier) {
        return false;
    }
    let computed = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    computed.as_bytes().ct_eq(challenge.as_bytes()).into()
}

#[must_use]
pub fn avatar_stem(avatar_id: &str) -> String {
    let stem = if avatar_id.contains('@') {
        avatar_id.replace('@', "_")
    } else if let Some(base) = avatar_id.strip_suffix("#1") {
        base.to_owned()
    } else {
        match avatar_id.rsplit_once('#') {
            Some((base, suffix)) if is_art_suffix(suffix) => format!("{base}_{suffix}"),
            _ => avatar_id.to_owned(),
        }
    };
    urlencoding::encode(&stem).into_owned()
}

fn is_art_suffix(suffix: &str) -> bool {
    let digits = suffix.strip_suffix('+').unwrap_or(suffix);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

#[must_use]
pub fn api_base(configured: &str, host: Option<&str>, forwarded_proto: Option<&str>) -> String {
    let configured = configured.trim().trim_end_matches('/');
    if !configured.is_empty() {
        return configured.to_owned();
    }
    let Some(host) = host.filter(|h| !h.is_empty()) else {
        return DEFAULT_API_BASE.to_owned();
    };
    let scheme = match forwarded_proto.map(str::trim) {
        Some("https") => "https",
        _ => "http",
    };
    format!("{scheme}://{host}")
}

fn credentials_of<'a>(authorization: Option<&'a str>, scheme: &str) -> Option<&'a str> {
    let (given, rest) = authorization?.split_once(' ')?;
    given.eq_ignore_ascii_case(scheme).then(|| rest.trim())
}

pub fn client_credentials(
    authorization: Option<&str>,
    body_id: Option<&str>,
    body_secret: Option<&str>,
) -> Result<(String, String), ProtocolError> {
    let Some(encoded) = credentials_of(authorization, "basic") else {
        return match (body_id, body_secret) {
            (Some(id), Some(secret)) => Ok((id.to_owned(), secret.to_owned())),
            _ => Err(ProtocolError::invalid_client()),
        };
    };
    let decoded = STANDARD
        .decode(encoded)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .ok_or_else(ProtocolError::invalid_client)?;
    let (id, secret) = decoded
        .split_once(':')
        .ok_or_else(ProtocolError::invalid_client)?;
    let unescape = |part: &str| {
        urlencoding::decode(part)
            .map(std::borrow::Cow::into_owned)
            .map_err(|_| ProtocolError::invalid_client())
    };
    let (id, secret) = (unescape(id)?, unescape(secret)?);
    if body_id.is_some_and(|other| other != id) {
        return Err(ProtocolError::invalid_request(
            "client_id in the body does not match the Authorization header",
        ));
    }
    Ok((id, secret))
}

#[must_use]
pub fn bearer_token(authorization: Option<&str>) -> Option<&str> {
    credentials_of(authorization, "bearer").filter(|token| !token.is_empty())
}
