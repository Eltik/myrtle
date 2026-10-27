//! `STORAGE_*`, `CLOUDFLARE_*` and `ASSET_DELIVERY` parsing.
//!
use std::env::var;
use std::fmt::{self, Display, Formatter};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::str::FromStr;
use std::thread::available_parallelism;

use thiserror::Error;

use crate::evict::EvictPolicy;

/// Served roots synced by default, generic asset route's allowlist
pub const DEFAULT_ROOTS: &[&str] = &["textures", "spine", "audio", "video", "portraits"];
/// `Cache-Control` on every uploaded object
pub const DEFAULT_CACHE_CONTROL: &str = "public, max-age=604800";
/// Concurrent uploads and deletes
pub const DEFAULT_SYNC_CONCURRENCY: usize = 16;
/// When to do multipart uploads (256 MiB)
pub const DEFAULT_MULTIPART_THRESHOLD_BYTES: u64 = 268_435_456;
/// Part size for multipart uploads (64 MiB)
pub const MULTIPART_PART_SIZE_BYTES: u64 = 67_108_864;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageKind {
    Cloudflare,
    Local,
}

impl StorageKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cloudflare => "cloudflare",
            Self::Local => "local",
        }
    }
}

impl Display for StorageKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for StorageKind {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "cloudflare" | "r2" => Ok(Self::Cloudflare),
            "local" | "fs" => Ok(Self::Local),
            _ => Err(ConfigError::UnknownType(s.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigError {
    #[error(
        "STORAGE_TYPE={0:?} is not a storage type; accepted: cloudflare (r2), local (fs), or off"
    )]
    UnknownType(String),
    #[error("{var} must be set when STORAGE_TYPE={kind}")]
    Missing { var: &'static str, kind: String },
    #[error("{var}={value:?} is invalid: {reason}")]
    Invalid {
        var: &'static str,
        value: String,
        reason: &'static str,
    },
    #[error(
        "ASSET_DELIVERY=redirect needs CLOUDFLARE_PUBLIC_BASE_URL, the public base the browser is sent to"
    )]
    RedirectWithoutPublicUrl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageConfig {
    pub kind: StorageKind,
    /// Empty for `local`
    pub bucket: String,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub account_id: Option<String>,
    /// Resolved endpoint: `CLOUDFLARE_S3_API`, else derived from account for
    /// `cloudflare`; `None` for `local`
    pub endpoint: Option<String>,
    /// Key prefix without leading or trailing slashes; empty for none
    pub prefix: String,
    /// Public GET base without a trailing slash
    pub public_url: Option<String>,
    pub local_dir: Option<PathBuf>,
    /// Top-level dirs under each server
    pub roots: Vec<String>,
    pub cache_control: String,
    pub sync_concurrency: usize,
    pub hash_threads: usize,
    pub multipart_threshold: u64,
    pub part_size: u64,
    /// `STORAGE_EVICT_LOCAL` and `STORAGE_EVICT_KEEP`
    pub evict: EvictPolicy,
}

/// Reads `name`, treating empty or whitespace-only values as unset.
pub(crate) fn get(lookup: &dyn Fn(&str) -> Option<String>, name: &str) -> Option<String> {
    lookup(name)
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

fn require(
    lookup: &dyn Fn(&str) -> Option<String>,
    name: &'static str,
    kind: StorageKind,
) -> Result<String, ConfigError> {
    get(lookup, name).ok_or_else(|| ConfigError::Missing {
        var: name,
        kind: kind.to_string(),
    })
}

fn parse_positive<T: FromStr + PartialEq + Default>(
    lookup: &dyn Fn(&str) -> Option<String>,
    name: &'static str,
    default: T,
) -> Result<T, ConfigError> {
    get(lookup, name).map_or_else(
        || Ok(default),
        |v| match v.parse::<T>() {
            Ok(n) if n != T::default() => Ok(n),
            _ => Err(ConfigError::Invalid {
                var: name,
                value: v,
                reason: "expected a positive integer",
            }),
        },
    )
}

/// `scheme://host[:port]` of an endpoint URL, dropping any path and
/// trailing slash
#[must_use]
pub fn normalize_endpoint(raw: &str) -> String {
    let raw = raw.trim();
    raw.find("://").map_or_else(
        || raw.split('/').next().unwrap_or(raw).to_owned(),
        |i| {
            let rest = &raw[i + 3..];
            let host = rest.split(['/', '?', '#']).next().unwrap_or(rest);
            format!("{}{host}", &raw[..i + 3])
        },
    )
}

#[must_use]
pub fn parse_roots(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|r| r.trim().trim_matches('/').to_owned())
        .filter(|r| !r.is_empty())
        .collect()
}

fn available_cpus() -> usize {
    available_parallelism().map_or(4, NonZeroUsize::get)
}

impl StorageConfig {
    /// Reads process environment. `Ok(None)` means storage is disabled.
    ///
    /// # Errors
    /// See [`StorageConfig::from_lookup`].
    pub fn from_env() -> Result<Option<Self>, ConfigError> {
        Self::from_lookup(&|k| var(k).ok())
    }

    /// Reads variables through `lookup`. `Ok(None)` when `STORAGE_TYPE` is
    /// unset, empty or `off`.
    ///
    /// # Errors
    /// An unknown `STORAGE_TYPE`, a variable the kind requires that is missing,
    /// or a value that does not parse. Each error names the variable.
    #[allow(clippy::too_many_lines)]
    pub fn from_lookup(
        lookup: &dyn Fn(&str) -> Option<String>,
    ) -> Result<Option<Self>, ConfigError> {
        let Some(raw_kind) = get(lookup, "STORAGE_TYPE") else {
            return Ok(None);
        };
        if raw_kind.eq_ignore_ascii_case("off") {
            return Ok(None);
        }
        let kind: StorageKind = raw_kind.parse()?;

        let access_key_id = get(lookup, "CLOUDFLARE_API_ACCESS_KEY");
        let secret_access_key = get(lookup, "CLOUDFLARE_API_SECRET_KEY");
        let account_id = get(lookup, "CLOUDFLARE_ACCOUNT_ID");
        let explicit_endpoint = get(lookup, "CLOUDFLARE_S3_API").map(|e| normalize_endpoint(&e));
        let missing = |var: &'static str| ConfigError::Missing {
            var,
            kind: kind.to_string(),
        };

        let (bucket, endpoint, local_dir) = match kind {
            StorageKind::Cloudflare => {
                let bucket = require(lookup, "CLOUDFLARE_BUCKET", kind)?;
                let endpoint = if let Some(e) = explicit_endpoint {
                    e
                } else {
                    let account = account_id
                        .clone()
                        .ok_or_else(|| missing("CLOUDFLARE_ACCOUNT_ID"))?;
                    format!("https://{account}.r2.cloudflarestorage.com")
                };
                if access_key_id.is_none() {
                    return Err(missing("CLOUDFLARE_API_ACCESS_KEY"));
                }
                if secret_access_key.is_none() {
                    return Err(missing("CLOUDFLARE_API_SECRET_KEY"));
                }
                let local_dir = get(lookup, "STORAGE_LOCAL_DIR").map(PathBuf::from);
                (bucket, Some(endpoint), local_dir)
            }
            StorageKind::Local => (
                get(lookup, "CLOUDFLARE_BUCKET").unwrap_or_default(),
                None,
                Some(PathBuf::from(require(lookup, "STORAGE_LOCAL_DIR", kind)?)),
            ),
        };

        let prefix = get(lookup, "CLOUDFLARE_PREFIX")
            .map(|p| p.trim_matches('/').to_owned())
            .unwrap_or_default();
        let public_url =
            get(lookup, "CLOUDFLARE_PUBLIC_BASE_URL").map(|u| u.trim_end_matches('/').to_owned());

        let roots = match get(lookup, "STORAGE_ROOTS") {
            Some(raw) => {
                let roots = parse_roots(&raw);
                if roots.is_empty() {
                    return Err(ConfigError::Invalid {
                        var: "STORAGE_ROOTS",
                        value: raw,
                        reason: "expected a comma list of directory names",
                    });
                }
                roots
            }
            None => DEFAULT_ROOTS.iter().map(|r| (*r).to_owned()).collect(),
        };

        Ok(Some(Self {
            kind,
            bucket,
            access_key_id,
            secret_access_key,
            account_id,
            endpoint,
            prefix,
            public_url,
            local_dir,
            roots,
            cache_control: get(lookup, "STORAGE_CACHE_CONTROL")
                .unwrap_or_else(|| DEFAULT_CACHE_CONTROL.to_owned()),
            sync_concurrency: parse_positive(
                lookup,
                "STORAGE_SYNC_CONCURRENCY",
                DEFAULT_SYNC_CONCURRENCY,
            )?,
            hash_threads: parse_positive(lookup, "STORAGE_SYNC_HASH_THREADS", available_cpus())?,
            multipart_threshold: parse_positive(
                lookup,
                "STORAGE_MULTIPART_THRESHOLD_BYTES",
                DEFAULT_MULTIPART_THRESHOLD_BYTES,
            )?,
            part_size: MULTIPART_PART_SIZE_BYTES,
            evict: EvictPolicy::from_lookup(lookup)?,
        }))
    }
}

/// How backend answers a request for a synced asset
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryMode {
    /// Serve from disk, exactly as without storage
    Local,
    /// `302 Found` to the object's public URL
    Redirect,
    /// Stream object from the store through the backend
    Proxy,
}

impl DeliveryMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Redirect => "redirect",
            Self::Proxy => "proxy",
        }
    }

    /// Reads `ASSET_DELIVERY` from the process environment.
    ///
    /// # Errors
    /// See [`DeliveryMode::from_lookup`].
    pub fn from_env(storage: Option<&StorageConfig>) -> Result<Self, ConfigError> {
        Self::from_lookup(&|k| var(k).ok(), storage)
    }

    /// `ASSET_DELIVERY` if set, else `redirect` when storage has a public
    /// URL, `proxy` when it has none, and `local` when storage is disabled.
    /// Storage disabled always yields `local` without reading
    /// `ASSET_DELIVERY` at all: `STORAGE_TYPE` unset is the kill switch.
    ///
    /// # Errors
    /// An unknown value, or `redirect` without `CLOUDFLARE_PUBLIC_BASE_URL`.
    pub fn from_lookup(
        lookup: &dyn Fn(&str) -> Option<String>,
        storage: Option<&StorageConfig>,
    ) -> Result<Self, ConfigError> {
        let Some(storage) = storage else {
            return Ok(Self::Local);
        };
        let explicit = match get(lookup, "ASSET_DELIVERY") {
            None => None,
            Some(v) => Some(match v.to_ascii_lowercase().as_str() {
                "local" => Self::Local,
                "redirect" => Self::Redirect,
                "proxy" => Self::Proxy,
                _ => {
                    return Err(ConfigError::Invalid {
                        var: "ASSET_DELIVERY",
                        value: v,
                        reason: "expected local, redirect or proxy",
                    });
                }
            }),
        };
        let mode = explicit.unwrap_or_else(|| {
            if storage.public_url.is_some() {
                Self::Redirect
            } else {
                Self::Proxy
            }
        });
        if mode == Self::Redirect && storage.public_url.is_none() {
            return Err(ConfigError::RedirectWithoutPublicUrl);
        }
        Ok(mode)
    }
}

impl Display for DeliveryMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
