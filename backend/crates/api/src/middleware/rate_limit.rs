use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::middleware::Next;
use axum::response::Response;
use dashmap::DashMap;
use myrtle_app::AppContext;
use myrtle_app::services::accounts::jwt;

use crate::error::ErrorCode;
use crate::extract::auth::{bearer, is_service_call};
use crate::extract::client_ip::peer_ip;
use crate::metrics::METRICS;
use crate::middleware::observe::matched_route;

const SWEEP_EVERY: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bucket {
    Auth,
    Expensive,
    Upstream,
    Default,
}

const ROUTE_BUCKETS: &[(&str, Bucket)] = &[
    ("/api/dps/calculate", Bucket::Expensive),
    ("/api/hps/calculate", Bucket::Expensive),
    ("/api/dps/batch", Bucket::Expensive),
    ("/api/hps/batch", Bucket::Expensive),
    ("/api/dps/rank", Bucket::Expensive),
    ("/api/dps/supporters", Bucket::Expensive),
    ("/api/base/optimize", Bucket::Expensive),
    ("/api/base/rotation", Bucket::Expensive),
    ("/api/base/evaluate", Bucket::Expensive),
    ("/api/user/max-level-cost", Bucket::Expensive),
    ("/api/user/improvements", Bucket::Expensive),
    ("/api/squads/analyze", Bucket::Expensive),
    ("/api/gacha/fetch", Bucket::Upstream),
    ("/api/refresh", Bucket::Upstream),
    ("/api/friends", Bucket::Upstream),
    ("/api/players/search", Bucket::Upstream),
];

impl Bucket {
    #[must_use]
    pub fn of(route: &str) -> Self {
        if route.starts_with("/api/login") || route.starts_with("/api/auth/") {
            return Self::Auth;
        }
        ROUTE_BUCKETS
            .iter()
            .find_map(|&(path, bucket)| (path == route).then_some(bucket))
            .unwrap_or(Self::Default)
    }

    #[must_use]
    pub const fn allowance(self, rpm: u32) -> u32 {
        match self {
            Self::Auth => 10,
            Self::Expensive => 20,
            Self::Upstream => 6,
            Self::Default => rpm,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Auth => "auth",
            Self::Expensive => "expensive",
            Self::Upstream => "upstream",
            Self::Default => "default",
        }
    }
}

#[must_use]
pub fn bucket_for(route: &str, headers: &HeaderMap, jwt_secret: &str) -> Bucket {
    let bucket = Bucket::of(route);
    let signed_in = || bearer(headers).is_some_and(|t| jwt::verify_token(jwt_secret, t).is_ok());
    if bucket == Bucket::Upstream && !signed_in() {
        return Bucket::Default;
    }
    bucket
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cidr {
    V4 { net: u32, mask: u32 },
    V6 { net: u128, mask: u128 },
}

impl Cidr {
    #[must_use]
    pub fn parse(spec: &str) -> Option<Self> {
        let (addr, bits) = match spec.split_once('/') {
            Some((addr, bits)) => (addr, Some(bits.trim().parse::<u32>().ok()?)),
            None => (spec, None),
        };
        match addr.trim().parse::<IpAddr>().ok()? {
            IpAddr::V4(v4) => {
                let bits = bits.unwrap_or(32);
                let mask = u32::MAX.checked_shl(32u32.checked_sub(bits)?).unwrap_or(0);
                Some(Self::V4 {
                    net: u32::from(v4) & mask,
                    mask,
                })
            }
            IpAddr::V6(v6) => {
                let bits = bits.unwrap_or(128);
                let mask = u128::MAX
                    .checked_shl(128u32.checked_sub(bits)?)
                    .unwrap_or(0);
                Some(Self::V6 {
                    net: u128::from(v6) & mask,
                    mask,
                })
            }
        }
    }

    #[must_use]
    pub fn contains(self, ip: IpAddr) -> bool {
        match (self, ip) {
            (Self::V4 { net, mask }, IpAddr::V4(v4)) => u32::from(v4) & mask == net,
            (Self::V6 { net, mask }, IpAddr::V6(v6)) => u128::from(v6) & mask == net,
            (Self::V4 { .. }, IpAddr::V6(v6)) => v6
                .to_ipv4_mapped()
                .is_some_and(|v4| self.contains(IpAddr::V4(v4))),
            (Self::V6 { .. }, IpAddr::V4(_)) => false,
        }
    }
}

struct Window {
    minute: AtomicU64,
    hits: AtomicU64,
}

pub struct RateLimiter {
    rpm: u32,
    hops: usize,
    exempt: Vec<Cidr>,
    windows: DashMap<(Bucket, IpAddr), Window>,
    started: Instant,
    seen: AtomicU64,
}

impl RateLimiter {
    #[must_use]
    pub fn new(rpm: u32, trusted_proxy_hops: u8, exempt_ips: &[String]) -> Arc<Self> {
        let exempt = exempt_ips.iter().filter_map(|entry| {
            let cidr = Cidr::parse(entry);
            if cidr.is_none() {
                tracing::warn!(entry, "ignoring unparseable rate_limit.exempt_ips entry");
            }
            cidr
        });
        Arc::new(Self {
            rpm,
            hops: usize::from(trusted_proxy_hops),
            exempt: exempt.collect(),
            windows: DashMap::new(),
            started: Instant::now(),
            seen: AtomicU64::new(0),
        })
    }

    pub fn over(&self, bucket: Bucket, ip: IpAddr) -> bool {
        let minute = self.started.elapsed().as_secs() / 60;
        if self
            .seen
            .fetch_add(1, Ordering::Relaxed)
            .is_multiple_of(SWEEP_EVERY)
        {
            self.windows
                .retain(|_, w| w.minute.load(Ordering::Relaxed) >= minute);
        }
        let window = self.windows.entry((bucket, ip)).or_insert_with(|| Window {
            minute: AtomicU64::new(minute),
            hits: AtomicU64::new(0),
        });
        let hits = if window.minute.swap(minute, Ordering::Relaxed) == minute {
            window.hits.fetch_add(1, Ordering::Relaxed) + 1
        } else {
            window.hits.store(1, Ordering::Relaxed);
            1
        };
        drop(window);
        hits > u64::from(bucket.allowance(self.rpm))
    }
}

pub async fn rate_limit(
    State((ctx, limiter)): State<(AppContext, Arc<RateLimiter>)>,
    req: Request,
    next: Next,
) -> Response {
    if is_service_call(req.headers(), &ctx) {
        return next.run(req).await;
    }
    let Some(ip) = peer_ip(req.headers(), req.extensions(), limiter.hops) else {
        return next.run(req).await;
    };
    if limiter.exempt.iter().any(|cidr| cidr.contains(ip)) {
        return next.run(req).await;
    }
    let route = matched_route(&req).unwrap_or_else(|| req.uri().path());
    let bucket = bucket_for(route, req.headers(), &ctx.secrets.jwt);
    if limiter.over(bucket, ip) {
        METRICS.rate_limited(bucket.label());
        tracing::warn!(route, bucket = bucket.label(), %ip, "rate limited");
        let mut response = ErrorCode::RateLimited.respond("rate limited");
        response
            .headers_mut()
            .insert(header::RETRY_AFTER, HeaderValue::from_static("60"));
        return response;
    }
    next.run(req).await
}
