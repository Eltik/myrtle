use std::collections::BTreeMap;
use std::fmt;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderValue, USER_AGENT};
use tokio::sync::Mutex;
use tokio::time::{Interval, MissedTickBehavior};

use crate::corpus::CorpusEntry;

#[derive(Clone)]
pub struct Secret(String);

impl Secret {
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    A,
    B,
}

pub fn role_env_suffix(role: &str) -> String {
    role.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

pub fn token_env_names(role: &str, side: Option<Side>) -> Vec<String> {
    let suffix = role_env_suffix(role);
    let mut names = Vec::new();
    match side {
        Some(Side::A) => names.push(format!("PARITY_A_TOKEN_{suffix}")),
        Some(Side::B) => names.push(format!("PARITY_B_TOKEN_{suffix}")),
        None => {}
    }
    names.push(format!("PARITY_TOKEN_{suffix}"));
    names
}

#[derive(Debug, Clone, Default)]
pub struct Tokens {
    by_role: BTreeMap<String, Secret>,
}

impl Tokens {
    pub fn from_env<'a, I: IntoIterator<Item = &'a str>>(roles: I, side: Option<Side>) -> Self {
        Self::from_lookup(roles, side, |name| std::env::var(name).ok())
    }

    pub fn from_lookup<'a, I, F>(roles: I, side: Option<Side>, lookup: F) -> Self
    where
        I: IntoIterator<Item = &'a str>,
        F: Fn(&str) -> Option<String>,
    {
        let mut by_role = BTreeMap::new();
        for role in roles {
            let found = token_env_names(role, side)
                .iter()
                .find_map(|name| lookup(name).filter(|v| !v.trim().is_empty()));
            if let Some(v) = found {
                by_role.insert(role.to_owned(), Secret::new(v.trim().to_owned()));
            }
        }
        Self { by_role }
    }

    pub fn get(&self, role: &str) -> Option<&Secret> {
        self.by_role.get(role)
    }

    pub fn roles(&self) -> impl Iterator<Item = &str> {
        self.by_role.keys().map(String::as_str)
    }
}

#[derive(Debug, Clone)]
pub struct HttpOptions {
    pub concurrency: usize,
    pub timeout: Duration,
    pub rps: f64,
    pub no_proxy: bool,
}

impl Default for HttpOptions {
    fn default() -> Self {
        Self {
            concurrency: 4,
            timeout: Duration::from_secs(30),
            rps: 0.0,
            no_proxy: false,
        }
    }
}

pub struct Pacer {
    interval: Option<Mutex<Interval>>,
}

impl Pacer {
    pub fn new(rps: f64) -> Self {
        if rps <= 0.0 || !rps.is_finite() {
            return Self { interval: None };
        }
        let mut interval = tokio::time::interval(Duration::from_secs_f64(1.0 / rps));
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        Self {
            interval: Some(Mutex::new(interval)),
        }
    }

    pub async fn wait(&self) {
        if let Some(m) = &self.interval {
            m.lock().await.tick().await;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fetched {
    pub status: u16,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchError {
    MissingToken { role: String, env: String },
    Transport(String),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingToken { role, env } => write!(f, "no token for role `{role}` (set {env})"),
            Self::Transport(m) => f.write_str(m),
        }
    }
}

pub struct Target {
    pub label: String,
    base: String,
    side: Option<Side>,
    tokens: Tokens,
    client: reqwest::Client,
    pacer: Pacer,
}

impl fmt::Debug for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Target")
            .field("base", &self.base)
            .field("roles", &self.tokens.roles().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}

impl Target {
    pub fn new(base: &str, side: Option<Side>, tokens: Tokens, opts: &HttpOptions) -> Result<Self> {
        let base = base.trim_end_matches('/').to_owned();
        if !(base.starts_with("http://") || base.starts_with("https://")) {
            anyhow::bail!("base URL must start with http:// or https://, got {base:?}");
        }
        let _ = rustls::crypto::ring::default_provider().install_default();
        let mut builder = reqwest::Client::builder()
            .timeout(opts.timeout)
            .redirect(reqwest::redirect::Policy::none());
        if opts.no_proxy {
            builder = builder.no_proxy();
        }
        let client = builder.build().context("building HTTP client")?;
        Ok(Self {
            label: base.clone(),
            base,
            side,
            tokens,
            client,
            pacer: Pacer::new(opts.rps),
        })
    }

    pub async fn fetch(&self, entry: &CorpusEntry) -> Result<Fetched, FetchError> {
        let url = format!("{}{}", self.base, entry.path);
        let mut req = self.client.get(&url).header(
            USER_AGENT,
            HeaderValue::from_static(concat!("myrtle-parity/", env!("CARGO_PKG_VERSION"))),
        );
        if let Some(role) = &entry.role {
            let Some(secret) = self.tokens.get(role) else {
                let env = token_env_names(role, self.side).join(" or ");
                return Err(FetchError::MissingToken {
                    role: role.clone(),
                    env,
                });
            };
            let mut value =
                HeaderValue::from_str(&format!("Bearer {}", secret.expose())).map_err(|_| {
                    FetchError::Transport("token is not a valid header value".to_owned())
                })?;
            value.set_sensitive(true);
            req = req.header(AUTHORIZATION, value);
        }
        self.pacer.wait().await;
        let started = Instant::now();
        let resp = req.send().await.map_err(|e| transport(e, &self.base))?;
        let status = resp.status().as_u16();
        let content_type = resp
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let bytes = resp.bytes().await.map_err(|e| transport(e, &self.base))?;
        Ok(Fetched {
            status,
            content_type,
            bytes: bytes.to_vec(),
            elapsed_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        })
    }
}

fn transport(e: reqwest::Error, base: &str) -> FetchError {
    let what = if e.is_timeout() {
        "timeout"
    } else if e.is_connect() {
        "connect failed"
    } else if e.is_body() || e.is_decode() {
        "body read failed"
    } else {
        "request failed"
    };
    FetchError::Transport(format!("{what} ({base}): {}", e.without_url()))
}
