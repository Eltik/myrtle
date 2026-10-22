//! `S3_*` and `ASSET_DELIVERY` parsing.
//!
//! Every reader takes a lookup function so tests run without touching the
//! process environment; [`StorageConfig::from_env`] and
//! [`DeliveryMode::from_env`] pass `std::env::var`. A value that is empty or
//! only whitespace counts as unset, so `S3_ROOTS=""` means the default roots
//! and `S3_TYPE=""` means storage is disabled.
//!
//! For `cloudflare`, each generic credential name falls back to the
//! Cloudflare-native one when unset: `S3_ACCOUNT_ID` to
//! `CLOUDFLARE_ACCOUNT_ID`, `S3_ENDPOINT` to `CLOUDFLARE_S3_API`,
//! `S3_ACCESS_KEY_ID` to `CLOUDFLARE_API_ACCESS_KEY`, `S3_SECRET_ACCESS_KEY`
//! to `CLOUDFLARE_API_SECRET_KEY`. `CLOUDFLARE_API_TOKEN` is the dashboard
//! API token, not an S3 credential, and is never read.
//!
//! The public base is `S3_PUBLIC_BASE_URL`, with `S3_PUBLIC_URL` accepted as
//! an alias; the canonical name wins when both are set.

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use crate::evict::EvictPolicy;

/// The served roots synced by default: the generic asset route's allowlist.
pub const DEFAULT_ROOTS: &[&str] = &["textures", "spine", "audio", "video", "portraits"];
/// `Cache-Control` on every uploaded object: what the backend sends today.
pub const DEFAULT_CACHE_CONTROL: &str = "public, max-age=604800";
/// Concurrent uploads and deletes.
pub const DEFAULT_SYNC_CONCURRENCY: usize = 16;
/// Files at or above this size (256 MiB) go up as multipart uploads.
pub const DEFAULT_MULTIPART_THRESHOLD_BYTES: u64 = 268_435_456;
/// Part size for multipart uploads: 64 MiB.
pub const MULTIPART_PART_SIZE_BYTES: u64 = 67_108_864;

/// Which backend holds the objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageKind {
    /// Cloudflare R2 (`cloudflare`, alias `r2`).
    Cloudflare,
    /// Amazon S3 (`aws`).
    Aws,
    /// Any other S3-compatible server: MinIO, Backblaze, Garage (`s3`).
    S3,
    /// A plain directory standing in for the bucket (`local`, alias `fs`).
    Local,
}

impl StorageKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cloudflare => "cloudflare",
            Self::Aws => "aws",
            Self::S3 => "s3",
            Self::Local => "local",
        }
    }
}

impl fmt::Display for StorageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for StorageKind {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "cloudflare" | "r2" => Ok(Self::Cloudflare),
            "aws" => Ok(Self::Aws),
            "s3" => Ok(Self::S3),
            "local" | "fs" => Ok(Self::Local),
            _ => Err(ConfigError::UnknownType(s.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ConfigError {
    #[error(
        "S3_TYPE={0:?} is not a storage type; accepted: cloudflare (r2), aws, s3, local (fs), or off"
    )]
    UnknownType(String),
    #[error("{var} must be set when S3_TYPE={kind}")]
    Missing { var: &'static str, kind: String },
    #[error("{var}={value:?} is invalid: {reason}")]
    Invalid {
        var: &'static str,
        value: String,
        reason: &'static str,
    },
    #[error(
        "ASSET_DELIVERY=redirect needs S3_PUBLIC_BASE_URL (alias S3_PUBLIC_URL), the public base the browser is sent to"
    )]
    RedirectWithoutPublicUrl,
}

/// Everything the `S3_*` variables describe, with per-kind defaults applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageConfig {
    pub kind: StorageKind,
    /// Empty for `local`, which needs no bucket.
    pub bucket: String,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub account_id: Option<String>,
    /// Resolved endpoint: `S3_ENDPOINT`, else derived from the account for
    /// `cloudflare`, else `None` (the SDK's own endpoint resolution).
    pub endpoint: Option<String>,
    pub region: String,
    pub force_path_style: bool,
    /// Key prefix without leading or trailing slashes; empty for none.
    pub prefix: String,
    /// Public GET base without a trailing slash.
    pub public_url: Option<String>,
    pub local_dir: Option<PathBuf>,
    /// Top-level dirs under each server that live in the store.
    pub roots: Vec<String>,
    pub cache_control: String,
    pub sync_concurrency: usize,
    pub hash_threads: usize,
    pub multipart_threshold: u64,
    pub part_size: u64,
    /// `S3_EVICT_LOCAL` and `S3_EVICT_KEEP`; off by default.
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
    get(lookup, name).ok_or(ConfigError::Missing {
        var: name,
        kind: kind.to_string(),
    })
}

/// A credential's generic name and, for `cloudflare`, its native fallback.
struct Credential {
    generic: &'static str,
    cloudflare: &'static str,
    /// Both names, for an error message.
    either: &'static str,
}

const ACCOUNT_ID: Credential = Credential {
    generic: "S3_ACCOUNT_ID",
    cloudflare: "CLOUDFLARE_ACCOUNT_ID",
    either: "S3_ACCOUNT_ID (or CLOUDFLARE_ACCOUNT_ID)",
};
const ENDPOINT: Credential = Credential {
    generic: "S3_ENDPOINT",
    cloudflare: "CLOUDFLARE_S3_API",
    either: "S3_ENDPOINT (or CLOUDFLARE_S3_API)",
};
const ACCESS_KEY_ID: Credential = Credential {
    generic: "S3_ACCESS_KEY_ID",
    cloudflare: "CLOUDFLARE_API_ACCESS_KEY",
    either: "S3_ACCESS_KEY_ID (or CLOUDFLARE_API_ACCESS_KEY)",
};
const SECRET_ACCESS_KEY: Credential = Credential {
    generic: "S3_SECRET_ACCESS_KEY",
    cloudflare: "CLOUDFLARE_API_SECRET_KEY",
    either: "S3_SECRET_ACCESS_KEY (or CLOUDFLARE_API_SECRET_KEY)",
};

/// The generic name, else (for `cloudflare` only) the native one.
fn credential(
    lookup: &dyn Fn(&str) -> Option<String>,
    cred: &Credential,
    kind: StorageKind,
) -> Option<String> {
    get(lookup, cred.generic).or_else(|| {
        (kind == StorageKind::Cloudflare)
            .then(|| get(lookup, cred.cloudflare))
            .flatten()
    })
}

/// The name an error cites for `cred` under `kind`.
const fn credential_name(cred: &Credential, kind: StorageKind) -> &'static str {
    match kind {
        StorageKind::Cloudflare => cred.either,
        _ => cred.generic,
    }
}

fn parse_positive<T: FromStr + PartialEq + Default>(
    lookup: &dyn Fn(&str) -> Option<String>,
    name: &'static str,
    default: T,
) -> Result<T, ConfigError> {
    match get(lookup, name) {
        None => Ok(default),
        Some(v) => match v.parse::<T>() {
            Ok(n) if n != T::default() => Ok(n),
            _ => Err(ConfigError::Invalid {
                var: name,
                value: v,
                reason: "expected a positive integer",
            }),
        },
    }
}

fn parse_bool(
    lookup: &dyn Fn(&str) -> Option<String>,
    name: &'static str,
    default: bool,
) -> Result<bool, ConfigError> {
    match get(lookup, name) {
        None => Ok(default),
        Some(v) => match v.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" => Ok(true),
            "0" | "false" | "no" => Ok(false),
            _ => Err(ConfigError::Invalid {
                var: name,
                value: v,
                reason: "expected 1 or 0",
            }),
        },
    }
}

/// `scheme://host[:port]` of an endpoint URL, dropping any path and
/// trailing slash. R2's per-bucket page shows the endpoint with `/<bucket>`
/// appended; the SDK adds the bucket itself.
#[must_use]
pub fn normalize_endpoint(raw: &str) -> String {
    let raw = raw.trim();
    match raw.find("://") {
        Some(i) => {
            let rest = &raw[i + 3..];
            let host = rest.split(['/', '?', '#']).next().unwrap_or(rest);
            format!("{}{host}", &raw[..i + 3])
        }
        None => raw.split('/').next().unwrap_or(raw).to_owned(),
    }
}

/// Splits a comma list, trimming entries and dropping empty ones.
#[must_use]
pub fn parse_roots(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|r| r.trim().trim_matches('/').to_owned())
        .filter(|r| !r.is_empty())
        .collect()
}

fn available_cpus() -> usize {
    std::thread::available_parallelism().map_or(4, std::num::NonZeroUsize::get)
}

impl StorageConfig {
    /// Reads the process environment. `Ok(None)` means storage is disabled.
    ///
    /// # Errors
    /// See [`StorageConfig::from_lookup`].
    pub fn from_env() -> Result<Option<Self>, ConfigError> {
        Self::from_lookup(&|k| std::env::var(k).ok())
    }

    /// Reads the variables through `lookup`. `Ok(None)` when `S3_TYPE` is
    /// unset, empty or `off`.
    ///
    /// # Errors
    /// An unknown `S3_TYPE`, a variable the kind requires that is missing,
    /// or a value that does not parse. Each error names the variable.
    pub fn from_lookup(
        lookup: &dyn Fn(&str) -> Option<String>,
    ) -> Result<Option<Self>, ConfigError> {
        let Some(raw_kind) = get(lookup, "S3_TYPE") else {
            return Ok(None);
        };
        if raw_kind.eq_ignore_ascii_case("off") {
            return Ok(None);
        }
        let kind: StorageKind = raw_kind.parse()?;

        let access_key_id = credential(lookup, &ACCESS_KEY_ID, kind);
        let secret_access_key = credential(lookup, &SECRET_ACCESS_KEY, kind);
        let account_id = credential(lookup, &ACCOUNT_ID, kind);
        let explicit_endpoint = credential(lookup, &ENDPOINT, kind).map(|e| normalize_endpoint(&e));
        let missing = |cred: &Credential| ConfigError::Missing {
            var: credential_name(cred, kind),
            kind: kind.to_string(),
        };

        let bucket = match kind {
            StorageKind::Local => get(lookup, "S3_BUCKET").unwrap_or_default(),
            _ => require(lookup, "S3_BUCKET", kind)?,
        };

        let endpoint = match kind {
            StorageKind::Cloudflare => match explicit_endpoint {
                Some(e) => Some(e),
                None => {
                    let account = account_id.clone().ok_or_else(|| missing(&ACCOUNT_ID))?;
                    Some(format!("https://{account}.r2.cloudflarestorage.com"))
                }
            },
            StorageKind::S3 => Some(explicit_endpoint.ok_or_else(|| missing(&ENDPOINT))?),
            StorageKind::Aws | StorageKind::Local => explicit_endpoint,
        };

        match kind {
            StorageKind::Cloudflare | StorageKind::S3 => {
                if access_key_id.is_none() {
                    return Err(missing(&ACCESS_KEY_ID));
                }
                if secret_access_key.is_none() {
                    return Err(missing(&SECRET_ACCESS_KEY));
                }
            }
            // Optional for aws, but one without the other is a typo, not a
            // request for the default chain.
            StorageKind::Aws => match (&access_key_id, &secret_access_key) {
                (Some(_), None) => {
                    return Err(ConfigError::Missing {
                        var: "S3_SECRET_ACCESS_KEY",
                        kind: kind.to_string(),
                    });
                }
                (None, Some(_)) => {
                    return Err(ConfigError::Missing {
                        var: "S3_ACCESS_KEY_ID",
                        kind: kind.to_string(),
                    });
                }
                _ => {}
            },
            StorageKind::Local => {}
        }

        let local_dir = match kind {
            StorageKind::Local => Some(PathBuf::from(require(lookup, "S3_LOCAL_DIR", kind)?)),
            _ => get(lookup, "S3_LOCAL_DIR").map(PathBuf::from),
        };

        let region = get(lookup, "S3_REGION").unwrap_or_else(|| {
            match kind {
                StorageKind::Cloudflare => "auto",
                StorageKind::Aws | StorageKind::S3 | StorageKind::Local => "us-east-1",
            }
            .to_owned()
        });
        // Path-style by default only for generic servers, which rarely have
        // wildcard DNS for virtual-hosted buckets.
        let force_path_style = parse_bool(lookup, "S3_FORCE_PATH_STYLE", kind == StorageKind::S3)?;

        let prefix = get(lookup, "S3_PREFIX")
            .map(|p| p.trim_matches('/').to_owned())
            .unwrap_or_default();
        let public_url = get(lookup, "S3_PUBLIC_BASE_URL")
            .or_else(|| get(lookup, "S3_PUBLIC_URL"))
            .map(|u| u.trim_end_matches('/').to_owned());

        let roots = match get(lookup, "S3_ROOTS") {
            Some(raw) => {
                let roots = parse_roots(&raw);
                if roots.is_empty() {
                    return Err(ConfigError::Invalid {
                        var: "S3_ROOTS",
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
            region,
            force_path_style,
            prefix,
            public_url,
            local_dir,
            roots,
            cache_control: get(lookup, "S3_CACHE_CONTROL")
                .unwrap_or_else(|| DEFAULT_CACHE_CONTROL.to_owned()),
            sync_concurrency: parse_positive(
                lookup,
                "S3_SYNC_CONCURRENCY",
                DEFAULT_SYNC_CONCURRENCY,
            )?,
            hash_threads: parse_positive(lookup, "S3_SYNC_HASH_THREADS", available_cpus())?,
            multipart_threshold: parse_positive(
                lookup,
                "S3_MULTIPART_THRESHOLD_BYTES",
                DEFAULT_MULTIPART_THRESHOLD_BYTES,
            )?,
            part_size: MULTIPART_PART_SIZE_BYTES,
            evict: EvictPolicy::from_lookup(lookup)?,
        }))
    }
}

/// How the backend answers a request for a synced asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryMode {
    /// Serve from disk, exactly as without storage.
    Local,
    /// `302 Found` to the object's public URL.
    Redirect,
    /// Stream the object from the store through the backend.
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
        Self::from_lookup(&|k| std::env::var(k).ok(), storage)
    }

    /// `ASSET_DELIVERY` if set, else `redirect` when storage has a public
    /// URL, `proxy` when it has none, and `local` when storage is disabled.
    /// Storage disabled always yields `local` without reading
    /// `ASSET_DELIVERY` at all: `S3_TYPE` unset is the kill switch.
    ///
    /// # Errors
    /// An unknown value, or `redirect` without `S3_PUBLIC_BASE_URL`.
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
        let mode = explicit.unwrap_or(if storage.public_url.is_some() {
            Self::Redirect
        } else {
            Self::Proxy
        });
        if mode == Self::Redirect && storage.public_url.is_none() {
            return Err(ConfigError::RedirectWithoutPublicUrl);
        }
        Ok(mode)
    }
}

impl fmt::Display for DeliveryMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    fn parse(pairs: &[(&str, &str)]) -> Result<Option<StorageConfig>, ConfigError> {
        let map = env(pairs);
        StorageConfig::from_lookup(&|k| map.get(k).cloned())
    }

    fn delivery(pairs: &[(&str, &str)]) -> Result<DeliveryMode, ConfigError> {
        let map = env(pairs);
        let lookup = |k: &str| map.get(k).cloned();
        let storage = StorageConfig::from_lookup(&lookup)?;
        DeliveryMode::from_lookup(&lookup, storage.as_ref())
    }

    const R2: &[(&str, &str)] = &[
        ("S3_TYPE", "cloudflare"),
        ("S3_BUCKET", "myrtle"),
        ("S3_ACCOUNT_ID", "abc123"),
        ("S3_ACCESS_KEY_ID", "id"),
        ("S3_SECRET_ACCESS_KEY", "secret"),
    ];

    fn with(
        base: &[(&str, &str)],
        extra: &[(&'static str, &'static str)],
    ) -> Vec<(String, String)> {
        let mut map: Vec<(String, String)> = base
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        for (k, v) in extra {
            map.retain(|(key, _)| key != k);
            map.push(((*k).to_owned(), (*v).to_owned()));
        }
        map
    }

    fn parse_owned(pairs: &[(String, String)]) -> Result<Option<StorageConfig>, ConfigError> {
        let refs: Vec<(&str, &str)> = pairs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        parse(&refs)
    }

    #[test]
    fn unset_empty_and_off_disable_storage() {
        assert_eq!(parse(&[]), Ok(None));
        assert_eq!(parse(&[("S3_TYPE", "")]), Ok(None));
        assert_eq!(parse(&[("S3_TYPE", "  ")]), Ok(None));
        assert_eq!(parse(&[("S3_TYPE", "off")]), Ok(None));
        assert_eq!(parse(&[("S3_TYPE", "OFF")]), Ok(None));
    }

    #[test]
    fn unknown_type_names_the_accepted_values() {
        let err = parse(&[("S3_TYPE", "gcs")]).unwrap_err();
        assert_eq!(err, ConfigError::UnknownType("gcs".into()));
        let msg = err.to_string();
        for word in ["cloudflare", "r2", "aws", "s3", "local", "fs", "off"] {
            assert!(msg.contains(word), "{msg}");
        }
    }

    #[test]
    fn aliases_parse() {
        assert_eq!("r2".parse(), Ok(StorageKind::Cloudflare));
        assert_eq!("Cloudflare".parse(), Ok(StorageKind::Cloudflare));
        assert_eq!("aws".parse(), Ok(StorageKind::Aws));
        assert_eq!("s3".parse(), Ok(StorageKind::S3));
        assert_eq!("fs".parse(), Ok(StorageKind::Local));
        assert_eq!(" local ".parse(), Ok(StorageKind::Local));
    }

    #[test]
    fn cloudflare_defaults() {
        let c = parse(R2).unwrap().unwrap();
        assert_eq!(c.kind, StorageKind::Cloudflare);
        assert_eq!(c.bucket, "myrtle");
        assert_eq!(
            c.endpoint.as_deref(),
            Some("https://abc123.r2.cloudflarestorage.com")
        );
        assert_eq!(c.region, "auto");
        assert!(!c.force_path_style);
        assert_eq!(c.prefix, "");
        assert_eq!(c.public_url, None);
        assert_eq!(
            c.roots,
            ["textures", "spine", "audio", "video", "portraits"]
        );
        assert_eq!(c.cache_control, "public, max-age=604800");
        assert_eq!(c.sync_concurrency, 16);
        assert!(c.hash_threads >= 1);
        assert_eq!(c.multipart_threshold, 268_435_456);
        assert_eq!(c.part_size, 64 * 1024 * 1024);
    }

    #[test]
    fn r2_alias_and_endpoint_override_need_no_account() {
        let pairs = with(
            R2,
            &[
                ("S3_TYPE", "r2"),
                ("S3_ACCOUNT_ID", ""),
                ("S3_ENDPOINT", "https://r2.example/"),
            ],
        );
        let c = parse_owned(&pairs).unwrap().unwrap();
        assert_eq!(c.kind, StorageKind::Cloudflare);
        assert_eq!(c.endpoint.as_deref(), Some("https://r2.example"));
    }

    #[test]
    fn cloudflare_errors_name_the_variable() {
        let missing = |var: &'static str| {
            let pairs = with(R2, &[(var, "")]);
            parse_owned(&pairs).unwrap_err()
        };
        for (var, named) in [
            ("S3_BUCKET", "S3_BUCKET"),
            ("S3_ACCOUNT_ID", "S3_ACCOUNT_ID (or CLOUDFLARE_ACCOUNT_ID)"),
            (
                "S3_ACCESS_KEY_ID",
                "S3_ACCESS_KEY_ID (or CLOUDFLARE_API_ACCESS_KEY)",
            ),
            (
                "S3_SECRET_ACCESS_KEY",
                "S3_SECRET_ACCESS_KEY (or CLOUDFLARE_API_SECRET_KEY)",
            ),
        ] {
            let err = missing(var);
            assert_eq!(
                err,
                ConfigError::Missing {
                    var: named,
                    kind: "cloudflare".into()
                }
            );
            assert!(err.to_string().contains(var));
        }
    }

    #[test]
    fn aws_defaults_and_optional_credentials() {
        let c = parse(&[("S3_TYPE", "aws"), ("S3_BUCKET", "b")])
            .unwrap()
            .unwrap();
        assert_eq!(c.kind, StorageKind::Aws);
        assert_eq!(c.region, "us-east-1");
        assert_eq!(c.endpoint, None);
        assert!(!c.force_path_style);
        assert_eq!(c.access_key_id, None);

        let half = parse(&[
            ("S3_TYPE", "aws"),
            ("S3_BUCKET", "b"),
            ("S3_ACCESS_KEY_ID", "x"),
        ]);
        assert_eq!(
            half.unwrap_err(),
            ConfigError::Missing {
                var: "S3_SECRET_ACCESS_KEY",
                kind: "aws".into()
            }
        );
        let other_half = parse(&[
            ("S3_TYPE", "aws"),
            ("S3_BUCKET", "b"),
            ("S3_SECRET_ACCESS_KEY", "x"),
        ]);
        assert_eq!(
            other_half.unwrap_err(),
            ConfigError::Missing {
                var: "S3_ACCESS_KEY_ID",
                kind: "aws".into()
            }
        );
        assert!(matches!(
            parse(&[("S3_TYPE", "aws")]),
            Err(ConfigError::Missing {
                var: "S3_BUCKET",
                ..
            })
        ));
    }

    #[test]
    fn generic_s3_defaults_and_requirements() {
        let base = [
            ("S3_TYPE", "s3"),
            ("S3_BUCKET", "b"),
            ("S3_ENDPOINT", "http://127.0.0.1:9000"),
            ("S3_ACCESS_KEY_ID", "id"),
            ("S3_SECRET_ACCESS_KEY", "secret"),
        ];
        let c = parse(&base).unwrap().unwrap();
        assert_eq!(c.kind, StorageKind::S3);
        assert_eq!(c.region, "us-east-1");
        assert!(c.force_path_style);
        assert_eq!(c.endpoint.as_deref(), Some("http://127.0.0.1:9000"));

        let no_endpoint = with(&base, &[("S3_ENDPOINT", "")]);
        assert!(matches!(
            parse_owned(&no_endpoint),
            Err(ConfigError::Missing {
                var: "S3_ENDPOINT",
                ..
            })
        ));
        let no_secret = with(&base, &[("S3_SECRET_ACCESS_KEY", "")]);
        assert!(matches!(
            parse_owned(&no_secret),
            Err(ConfigError::Missing {
                var: "S3_SECRET_ACCESS_KEY",
                ..
            })
        ));
        let virtual_hosted = with(&base, &[("S3_FORCE_PATH_STYLE", "0")]);
        assert!(
            !parse_owned(&virtual_hosted)
                .unwrap()
                .unwrap()
                .force_path_style
        );
    }

    #[test]
    fn local_needs_only_a_dir() {
        let c = parse(&[("S3_TYPE", "fs"), ("S3_LOCAL_DIR", "/tmp/bucket")])
            .unwrap()
            .unwrap();
        assert_eq!(c.kind, StorageKind::Local);
        assert_eq!(c.bucket, "");
        assert_eq!(c.local_dir, Some(PathBuf::from("/tmp/bucket")));
        assert!(matches!(
            parse(&[("S3_TYPE", "local")]),
            Err(ConfigError::Missing {
                var: "S3_LOCAL_DIR",
                ..
            })
        ));
    }

    #[test]
    fn overrides_parse_and_normalize() {
        let pairs = with(
            R2,
            &[
                ("S3_PREFIX", "/assets/"),
                ("S3_PUBLIC_BASE_URL", "https://assets.myrtle.moe/"),
                ("S3_ROOTS", " spine , audio,,"),
                ("S3_REGION", "weur"),
                ("S3_FORCE_PATH_STYLE", "1"),
                ("S3_CACHE_CONTROL", "public, max-age=60"),
                ("S3_SYNC_CONCURRENCY", "4"),
                ("S3_SYNC_HASH_THREADS", "2"),
                ("S3_MULTIPART_THRESHOLD_BYTES", "1024"),
            ],
        );
        let c = parse_owned(&pairs).unwrap().unwrap();
        assert_eq!(c.prefix, "assets");
        assert_eq!(c.public_url.as_deref(), Some("https://assets.myrtle.moe"));
        assert_eq!(c.roots, ["spine", "audio"]);
        assert_eq!(c.region, "weur");
        assert!(c.force_path_style);
        assert_eq!(c.cache_control, "public, max-age=60");
        assert_eq!(c.sync_concurrency, 4);
        assert_eq!(c.hash_threads, 2);
        assert_eq!(c.multipart_threshold, 1024);
    }

    #[test]
    fn bad_numbers_and_switches_are_errors() {
        for (var, value) in [
            ("S3_SYNC_CONCURRENCY", "0"),
            ("S3_SYNC_CONCURRENCY", "many"),
            ("S3_SYNC_HASH_THREADS", "-1"),
            ("S3_MULTIPART_THRESHOLD_BYTES", "1e9"),
            ("S3_FORCE_PATH_STYLE", "maybe"),
            ("S3_ROOTS", ",,"),
        ] {
            let pairs = with(R2, &[(var, value)]);
            match parse_owned(&pairs) {
                Err(ConfigError::Invalid { var: v, .. }) => assert_eq!(v, var),
                other => panic!("{var}={value}: {other:?}"),
            }
        }
    }

    /// The user's file: native Cloudflare names, empty generic ones.
    const R2_NATIVE: &[(&str, &str)] = &[
        ("S3_TYPE", "cloudflare"),
        ("S3_BUCKET", "assets"),
        ("S3_PREFIX", "myrtle"),
        ("S3_ROOTS", ""),
        ("S3_ENDPOINT", ""),
        ("CLOUDFLARE_ACCOUNT_ID", "acct"),
        ("CLOUDFLARE_API_ACCESS_KEY", "native-id"),
        ("CLOUDFLARE_API_SECRET_KEY", "native-secret"),
        ("CLOUDFLARE_API_TOKEN", "dashboard-token"),
    ];

    #[test]
    fn cloudflare_native_names_fill_unset_generic_ones() {
        let c = parse(R2_NATIVE).unwrap().unwrap();
        assert_eq!(c.account_id.as_deref(), Some("acct"));
        assert_eq!(
            c.endpoint.as_deref(),
            Some("https://acct.r2.cloudflarestorage.com")
        );
        assert_eq!(c.access_key_id.as_deref(), Some("native-id"));
        assert_eq!(c.secret_access_key.as_deref(), Some("native-secret"));
        // `S3_ROOTS=""` is unset: the default five.
        assert_eq!(
            c.roots,
            ["textures", "spine", "audio", "video", "portraits"]
        );

        let with_api = with(
            R2_NATIVE,
            &[(
                "CLOUDFLARE_S3_API",
                "https://other.r2.cloudflarestorage.com/",
            )],
        );
        assert_eq!(
            parse_owned(&with_api).unwrap().unwrap().endpoint.as_deref(),
            Some("https://other.r2.cloudflarestorage.com")
        );
    }

    #[test]
    fn generic_names_win_over_cloudflare_ones() {
        let pairs = with(
            R2_NATIVE,
            &[
                ("S3_ACCOUNT_ID", "generic-acct"),
                ("S3_ACCESS_KEY_ID", "generic-id"),
                ("S3_SECRET_ACCESS_KEY", "generic-secret"),
                ("CLOUDFLARE_S3_API", "https://native.example"),
                ("S3_ENDPOINT", "https://generic.example"),
            ],
        );
        let c = parse_owned(&pairs).unwrap().unwrap();
        assert_eq!(c.account_id.as_deref(), Some("generic-acct"));
        assert_eq!(c.access_key_id.as_deref(), Some("generic-id"));
        assert_eq!(c.secret_access_key.as_deref(), Some("generic-secret"));
        assert_eq!(c.endpoint.as_deref(), Some("https://generic.example"));
    }

    #[test]
    fn cloudflare_names_are_ignored_for_other_kinds() {
        let pairs = [
            ("S3_TYPE", "s3"),
            ("S3_BUCKET", "b"),
            ("CLOUDFLARE_S3_API", "https://native.example"),
            ("CLOUDFLARE_API_ACCESS_KEY", "id"),
            ("CLOUDFLARE_API_SECRET_KEY", "secret"),
        ];
        assert!(matches!(
            parse(&pairs),
            Err(ConfigError::Missing {
                var: "S3_ENDPOINT",
                ..
            })
        ));
        let aws = parse(&[
            ("S3_TYPE", "aws"),
            ("S3_BUCKET", "b"),
            ("CLOUDFLARE_API_ACCESS_KEY", "id"),
        ])
        .unwrap()
        .unwrap();
        assert_eq!(aws.access_key_id, None);
    }

    #[test]
    fn the_api_token_is_not_a_credential() {
        let pairs = with(R2_NATIVE, &[("CLOUDFLARE_API_ACCESS_KEY", "")]);
        assert_eq!(
            parse_owned(&pairs).unwrap_err(),
            ConfigError::Missing {
                var: "S3_ACCESS_KEY_ID (or CLOUDFLARE_API_ACCESS_KEY)",
                kind: "cloudflare".into()
            }
        );
    }

    #[test]
    fn public_base_url_is_canonical_and_public_url_an_alias() {
        let alias = with(R2, &[("S3_PUBLIC_URL", "https://alias.example/")]);
        assert_eq!(
            parse_owned(&alias).unwrap().unwrap().public_url.as_deref(),
            Some("https://alias.example")
        );
        let both = with(
            R2,
            &[
                ("S3_PUBLIC_URL", "https://alias.example"),
                ("S3_PUBLIC_BASE_URL", "https://canonical.example"),
            ],
        );
        assert_eq!(
            parse_owned(&both).unwrap().unwrap().public_url.as_deref(),
            Some("https://canonical.example")
        );
        let empty_canonical = with(
            R2,
            &[
                ("S3_PUBLIC_URL", "https://alias.example"),
                ("S3_PUBLIC_BASE_URL", ""),
            ],
        );
        assert_eq!(
            parse_owned(&empty_canonical)
                .unwrap()
                .unwrap()
                .public_url
                .as_deref(),
            Some("https://alias.example")
        );
    }

    #[test]
    fn empty_values_are_unset() {
        assert_eq!(parse(&[("S3_TYPE", "")]), Ok(None));
        let pairs = with(
            R2,
            &[
                ("S3_ROOTS", ""),
                ("S3_CACHE_CONTROL", ""),
                ("S3_REGION", ""),
                ("S3_SYNC_CONCURRENCY", ""),
                ("S3_FORCE_PATH_STYLE", ""),
                ("ASSET_DELIVERY", ""),
            ],
        );
        let c = parse_owned(&pairs).unwrap().unwrap();
        assert_eq!(
            c.roots,
            ["textures", "spine", "audio", "video", "portraits"]
        );
        assert_eq!(c.cache_control, "public, max-age=604800");
        assert_eq!(c.region, "auto");
        assert_eq!(c.sync_concurrency, 16);
        assert!(!c.force_path_style);
    }

    /// The user's `.env` shape: double quotes around a value with a comma
    /// and a space must come through dotenv as the bare value.
    #[test]
    fn quoted_values_survive_dotenv() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        std::fs::write(
            &path,
            concat!(
                "S3_TYPE=\"cloudflare\"\n",
                "S3_BUCKET=\"assets\"\n",
                "S3_PREFIX=\"myrtle\"\n",
                "S3_ROOTS=\"\"\n",
                "S3_PUBLIC_BASE_URL=\"https://assets.eltik.net\"\n",
                "S3_CACHE_CONTROL=\"public, max-age=3600\"\n",
                "CLOUDFLARE_ACCOUNT_ID=\"acct\"\n",
                "CLOUDFLARE_API_ACCESS_KEY=\"id\"\n",
                "CLOUDFLARE_API_SECRET_KEY=\"secret\"\n",
            ),
        )
        .unwrap();
        // The iterator parses without touching the process environment,
        // which the parallel tests share; it is deprecated, not removed.
        #[allow(deprecated)]
        let vars: HashMap<String, String> = dotenv::from_path_iter(&path)
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(vars["S3_CACHE_CONTROL"], "public, max-age=3600");
        assert_eq!(vars["S3_ROOTS"], "");
        let lookup = |k: &str| vars.get(k).cloned();
        let c = StorageConfig::from_lookup(&lookup).unwrap().unwrap();
        assert_eq!(c.cache_control, "public, max-age=3600");
        assert_eq!(c.prefix, "myrtle");
        assert_eq!(c.public_url.as_deref(), Some("https://assets.eltik.net"));
        assert_eq!(c.roots.len(), 5);
        let key = crate::key::object_key(&c.prefix, "en", "textures/a.png");
        assert_eq!(
            crate::key::public_url(c.public_url.as_deref().unwrap(), &key),
            "https://assets.eltik.net/myrtle/en/textures/a.png"
        );
        assert_eq!(
            DeliveryMode::from_lookup(&lookup, Some(&c)),
            Ok(DeliveryMode::Redirect)
        );
    }

    #[test]
    fn endpoints_lose_their_path_and_trailing_slash() {
        for (raw, want) in [
            (
                "https://acct.r2.cloudflarestorage.com",
                "https://acct.r2.cloudflarestorage.com",
            ),
            (
                "https://acct.r2.cloudflarestorage.com/",
                "https://acct.r2.cloudflarestorage.com",
            ),
            (
                "https://acct.r2.cloudflarestorage.com/assets",
                "https://acct.r2.cloudflarestorage.com",
            ),
            (
                "https://acct.r2.cloudflarestorage.com/assets/",
                "https://acct.r2.cloudflarestorage.com",
            ),
            ("http://127.0.0.1:9000/bucket/x", "http://127.0.0.1:9000"),
        ] {
            assert_eq!(normalize_endpoint(raw), want, "{raw}");
        }
        let pairs = with(
            R2_NATIVE,
            &[(
                "CLOUDFLARE_S3_API",
                "https://acct.r2.cloudflarestorage.com/assets",
            )],
        );
        assert_eq!(
            parse_owned(&pairs).unwrap().unwrap().endpoint.as_deref(),
            Some("https://acct.r2.cloudflarestorage.com")
        );
        let generic = with(R2, &[("S3_ENDPOINT", "https://r2.example/myrtle/")]);
        assert_eq!(
            parse_owned(&generic).unwrap().unwrap().endpoint.as_deref(),
            Some("https://r2.example")
        );
    }

    #[test]
    fn delivery_defaults() {
        assert_eq!(delivery(&[]), Ok(DeliveryMode::Local));
        assert_eq!(delivery(R2), Ok(DeliveryMode::Proxy));
        let public = with(R2, &[("S3_PUBLIC_BASE_URL", "https://cdn.example")]);
        let refs: Vec<(&str, &str)> = public
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(delivery(&refs), Ok(DeliveryMode::Redirect));
    }

    #[test]
    fn delivery_explicit_values() {
        let pairs = |mode: &'static str| with(R2, &[("ASSET_DELIVERY", mode)]);
        let run = |p: Vec<(String, String)>| {
            let refs: Vec<(&str, &str)> = p.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
            delivery(&refs)
        };
        assert_eq!(run(pairs("local")), Ok(DeliveryMode::Local));
        assert_eq!(run(pairs("PROXY")), Ok(DeliveryMode::Proxy));
        assert_eq!(
            run(pairs("redirect")),
            Err(ConfigError::RedirectWithoutPublicUrl)
        );
        assert!(matches!(
            run(pairs("cdn")),
            Err(ConfigError::Invalid {
                var: "ASSET_DELIVERY",
                ..
            })
        ));
    }

    #[test]
    fn storage_disabled_forces_local_delivery() {
        assert_eq!(
            delivery(&[("ASSET_DELIVERY", "redirect")]),
            Ok(DeliveryMode::Local)
        );
        assert_eq!(
            delivery(&[("ASSET_DELIVERY", "proxy")]),
            Ok(DeliveryMode::Local)
        );
        assert_eq!(
            delivery(&[("S3_TYPE", "off"), ("ASSET_DELIVERY", "proxy")]),
            Ok(DeliveryMode::Local)
        );
        assert_eq!(
            delivery(&[("ASSET_DELIVERY", "nonsense")]),
            Ok(DeliveryMode::Local)
        );
    }
}
