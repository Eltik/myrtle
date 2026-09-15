//! Cross-cutting request middleware: observation, timeout, rate limiting.
//!
//! Written as `axum::middleware::from_fn` layers rather than assembled from
//! third-party tower layers, because all three need the same three values - the
//! matched route, the request id and the resolved client address - and deriving
//! those once in one place is simpler than keeping three layers' notions of
//! them in agreement.

use std::net::{IpAddr, SocketAddr};
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, MatchedPath, Request, State};
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use dashmap::DashMap;
use subtle::ConstantTimeEq;
use tracing::Instrument;

use crate::app::metrics::METRICS;
use crate::app::state::AppState;

/// Header carrying the id that ties a log line to a response.
const REQUEST_ID: &str = "x-request-id";

/// Process start, so window arithmetic is on cheap `u64` seconds rather than
/// `Instant`s held inside the map.
static START: LazyLock<Instant> = LazyLock::new(Instant::now);

fn now_secs() -> u64 {
    START.elapsed().as_secs()
}

// ── Observation ─────────────────────────────────────────────────────────

/// Ceiling on how long a handler may take to produce a response.
///
/// This bounds the handler future, not the response body: a streamed response
/// is produced as soon as the stream exists, so a slow download is unaffected.
const HANDLER_TIMEOUT: Duration = Duration::from_secs(30);

/// The matched route (`/api/operators/{id}`) when the router has resolved one.
///
/// Its two callers need different fallbacks, hence the two wrappers below:
///
/// - [`metric_label`] must stay bounded, because it becomes a metric label, so
///   everything unmatched collapses into one series.
/// - [`classify_path`] must stay accurate even with no match, because the rate
///   limiter derives an allowance from it, so it falls back to the real path.
fn matched_route(req: &Request) -> Option<&str> {
    req.extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
}

fn metric_label(req: &Request) -> String {
    matched_route(req).unwrap_or("<unmatched>").to_owned()
}

fn classify_path(req: &Request) -> String {
    matched_route(req)
        .unwrap_or_else(|| req.uri().path())
        .to_owned()
}

/// One log line and one metric sample per request, plus the request id that
/// ties them to anything the handler logged.
pub async fn observe(req: Request, next: Next) -> Response {
    let started = Instant::now();
    let method = req.method().clone();
    let route = metric_label(&req);

    // Reuse an id supplied by the edge when there is one, so a trace spans the
    // proxy and this process; mint one otherwise.
    let request_id = req
        .headers()
        .get(REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.is_empty() && v.len() <= 128)
        .map_or_else(|| uuid::Uuid::new_v4().to_string(), ToOwned::to_owned);

    let span = tracing::info_span!(
        "http",
        method = %method,
        route = %route,
        request_id = %request_id,
    );

    let mut response =
        tokio::time::timeout(HANDLER_TIMEOUT, next.run(req).instrument(span.clone()))
            .await
            .unwrap_or_else(|_| {
                tracing::error!(
                    parent: &span,
                    timeout_secs = HANDLER_TIMEOUT.as_secs(),
                    "handler timed out"
                );
                timeout_response()
            });

    let elapsed = started.elapsed();
    let status = response.status();

    METRICS.record_http(&route, status.as_u16(), elapsed.as_micros() as u64);

    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(REQUEST_ID, value);
    }

    // One line per request: route, status, latency and the id to correlate on.
    // It is DEBUG, not INFO: production runs at `backend=info` and this line was
    // the whole of the log volume there, while route, status and latency are
    // already on the metrics above and the id is on the response header. Server
    // errors log at WARN so they surface without widening the filter.
    let latency_ms = elapsed.as_secs_f64() * 1000.0;
    if status.is_server_error() {
        tracing::warn!(
            parent: &span,
            status = status.as_u16(),
            latency_ms,
            "request failed"
        );
    } else {
        tracing::debug!(
            parent: &span,
            status = status.as_u16(),
            latency_ms,
            "request"
        );
    }

    response
}

fn timeout_response() -> Response {
    (
        StatusCode::GATEWAY_TIMEOUT,
        axum::Json(serde_json::json!({
            "error": { "code": "TIMEOUT", "message": "request timed out" }
        })),
    )
        .into_response()
}

// ── Rate limiting ───────────────────────────────────────────────────────

/// How many proxies sit in front of this process.
///
/// 0 (the default) means the peer address is the client and `X-Forwarded-For`
/// is ignored entirely. Behind one reverse proxy, set 1.
///
/// Hops are counted from the RIGHT of the header, because only the entries a
/// trusted proxy appended are trustworthy - the leftmost entries are whatever
/// the client chose to send.
static TRUSTED_PROXY_HOPS: LazyLock<usize> = LazyLock::new(|| {
    std::env::var("TRUSTED_PROXY_HOPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
});

fn client_ip(req: &Request) -> Option<IpAddr> {
    let hops = *TRUSTED_PROXY_HOPS;
    if hops > 0
        && let Some(forwarded) = req.headers().get("x-forwarded-for")
        && let Ok(list) = forwarded.to_str()
    {
        let entries: Vec<&str> = list.split(',').map(str::trim).collect();
        // The rightmost entry was appended by the nearest trusted proxy;
        // walking left by `hops - 1` reaches the address it saw as the client.
        if let Some(candidate) = entries.len().checked_sub(hops).and_then(|i| entries.get(i))
            && let Ok(ip) = candidate.parse::<IpAddr>()
        {
            return Some(ip);
        }
    }

    req.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.ip())
}

/// A single address or a CIDR block, for the exemption list.
///
/// Hand-parsed rather than pulling in an address crate: two integer comparisons
/// is the whole of it.
#[derive(Clone, Copy)]
enum Cidr {
    V4 { net: u32, mask: u32 },
    V6 { net: u128, mask: u128 },
}

impl Cidr {
    /// Accepts `10.0.0.0/8`, `2001:db8::/32`, or a bare address, which is taken
    /// as a single host. Returns `None` for anything it cannot read.
    fn parse(spec: &str) -> Option<Self> {
        let (addr, bits) = match spec.split_once('/') {
            Some((addr, bits)) => (addr, Some(bits.trim().parse::<u32>().ok()?)),
            None => (spec, None),
        };

        match addr.trim().parse::<IpAddr>().ok()? {
            IpAddr::V4(v4) => {
                let bits = bits.unwrap_or(32);
                if bits > 32 {
                    return None;
                }
                let mask = if bits == 0 {
                    0
                } else {
                    u32::MAX << (32 - bits)
                };
                Some(Self::V4 {
                    net: u32::from(v4) & mask,
                    mask,
                })
            }
            IpAddr::V6(v6) => {
                let bits = bits.unwrap_or(128);
                if bits > 128 {
                    return None;
                }
                let mask = if bits == 0 {
                    0
                } else {
                    u128::MAX << (128 - bits)
                };
                Some(Self::V6 {
                    net: u128::from(v6) & mask,
                    mask,
                })
            }
        }
    }

    fn contains(self, ip: IpAddr) -> bool {
        match (self, ip) {
            (Self::V4 { net, mask }, IpAddr::V4(v4)) => u32::from(v4) & mask == net,
            (Self::V6 { net, mask }, IpAddr::V6(v6)) => u128::from(v6) & mask == net,
            // A dual-stack listener reports an IPv4 peer as `::ffff:a.b.c.d`,
            // so an IPv4 rule has to match that form too or the exemption
            // silently stops applying depending on how the socket was bound.
            (Self::V4 { .. }, IpAddr::V6(v6)) => v6
                .to_ipv4_mapped()
                .is_some_and(|v4| self.contains(IpAddr::V4(v4))),
            (Self::V6 { .. }, IpAddr::V4(_)) => false,
        }
    }
}

/// Addresses that skip rate limiting entirely, from `RATE_LIMIT_EXEMPT_IPS`
/// (comma-separated addresses or CIDR blocks).
///
/// Loopback is always exempt and does not need listing. This is what keeps a
/// first-party server-side renderer on the same host from being throttled as
/// though it were one very busy visitor: its requests all arrive from one
/// address, so a per-address allowance is the wrong tool for them. Set this
/// when that renderer runs somewhere else - another host, a container network -
/// and so reaches this process from a routable address.
///
/// Only add addresses that cannot be reached by an untrusted client: an
/// exempted address has no allowance at all.
static EXEMPT_ADDRESSES: LazyLock<Vec<Cidr>> = LazyLock::new(|| {
    let raw = std::env::var("RATE_LIMIT_EXEMPT_IPS").unwrap_or_default();
    let mut parsed = Vec::new();
    for entry in raw.split(',').map(str::trim).filter(|e| !e.is_empty()) {
        if let Some(cidr) = Cidr::parse(entry) {
            parsed.push(cidr);
        } else {
            tracing::warn!(entry, "ignoring unparseable RATE_LIMIT_EXEMPT_IPS entry");
        }
    }
    if !parsed.is_empty() {
        tracing::info!(count = parsed.len(), "rate-limit exemptions loaded");
    }
    parsed
});

fn is_exempt(ip: IpAddr) -> bool {
    ip.is_loopback() || EXEMPT_ADDRESSES.iter().any(|cidr| cidr.contains(ip))
}

/// Which bucket a route falls in.
///
/// Deliberately coarse. The distinction that matters is between routes that
/// cost an upstream provider something, routes that cost this process a core,
/// and routes that are a cached read.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Bucket {
    /// Triggers a message from an upstream provider, or mints a token.
    Auth,
    /// Runs a combinatorial search or a simulation in this process.
    Expensive,
    /// Pulls from an upstream game API on the caller's behalf.
    Upstream,
    Default,
}

impl Bucket {
    fn of(route: &str) -> Self {
        if route.starts_with("/api/login") || route.starts_with("/api/auth/") {
            return Self::Auth;
        }
        if matches!(
            route,
            "/api/dps/calculate"
                | "/api/hps/calculate"
                | "/api/base/optimize"
                | "/api/base/rotation"
                | "/api/base/evaluate"
                | "/api/user/improvements"
        ) {
            return Self::Expensive;
        }
        if matches!(route, "/api/gacha/fetch" | "/api/refresh") {
            return Self::Upstream;
        }
        Self::Default
    }

    /// Requests per minute. The default bucket follows the configured
    /// `RATE_LIMIT_RPM`; the rest are fixed at what the route can afford.
    const fn allowance(self, configured_rpm: u32) -> u32 {
        match self {
            Self::Auth => 10,
            Self::Expensive => 20,
            Self::Upstream => 6,
            Self::Default => configured_rpm,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Auth => "auth",
            Self::Expensive => "expensive",
            Self::Upstream => "upstream",
            Self::Default => "default",
        }
    }
}

struct Window {
    /// Index of the 60-second window this count belongs to.
    window: AtomicU64,
    hits: AtomicU64,
}

static BUCKETS: LazyLock<DashMap<(Bucket, IpAddr), Window>> = LazyLock::new(DashMap::new);
static SWEEP_TICKER: AtomicU64 = AtomicU64::new(0);

/// Drop entries whose window has passed, every 10k admissions.
///
/// Sweeping inline rather than from a spawned task keeps this to one moving
/// part, and amortises to roughly nothing per request. Without it the map
/// would retain an entry per address seen.
fn maybe_sweep(current_window: u64) {
    if SWEEP_TICKER
        .fetch_add(1, Ordering::Relaxed)
        .is_multiple_of(10_000)
    {
        BUCKETS.retain(|_, w| w.window.load(Ordering::Relaxed) >= current_window);
    }
}

/// True when the caller presented the internal service key, which the
/// frontend's server-side rendering uses. Those calls are trusted and
/// first-party; holding them to a per-IP allowance would throttle the site
/// itself, since they all arrive from one address.
fn is_service_call(req: &Request, state: &AppState) -> bool {
    let configured = state.config.service_key.as_bytes();
    if configured.is_empty() {
        return false;
    }
    req.headers()
        .get("x-service-key")
        .is_some_and(|key| !key.is_empty() && key.as_bytes().ct_eq(configured).into())
}

pub async fn rate_limit(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let route = classify_path(&req);
    let bucket = Bucket::of(&route);

    if is_service_call(&req, &state) {
        return next.run(req).await;
    }

    let Some(ip) = client_ip(&req) else {
        // No usable client address - a misconfiguration, or a transport that
        // carries none. Refusing every such request would take the site down
        // over a deployment detail, so it passes.
        return next.run(req).await;
    };

    // Loopback and anything in `RATE_LIMIT_EXEMPT_IPS`: first-party callers
    // whose traffic is the sum of many real visitors, so a per-address
    // allowance would throttle all of them together.
    if is_exempt(ip) {
        return next.run(req).await;
    }

    let window = now_secs() / 60;
    let allowance = bucket.allowance(state.config.rate_limit_rpm);
    maybe_sweep(window);

    let entry = BUCKETS.entry((bucket, ip)).or_insert_with(|| Window {
        window: AtomicU64::new(window),
        hits: AtomicU64::new(0),
    });

    // Rolling the window and counting the hit are two atomics, not one, so two
    // requests arriving exactly as the minute turns can both count as the first
    // of the new window. That is a few extra admissions per address per minute
    // under a race, and it is the deliberate trade: a fixed window on two
    // `Relaxed` atomics costs nothing per request, where a mutex or a true
    // sliding window buys precision a rate limiter does not need.
    let hits = if entry.window.swap(window, Ordering::Relaxed) == window {
        entry.hits.fetch_add(1, Ordering::Relaxed) + 1
    } else {
        entry.hits.store(1, Ordering::Relaxed);
        1
    };
    drop(entry);

    if hits > u64::from(allowance) {
        METRICS.rate_limited(bucket.label());
        tracing::warn!(
            route = %route,
            bucket = bucket.label(),
            %ip,
            hits,
            allowance,
            "rate limited"
        );
        return rate_limited_response();
    }

    next.run(req).await
}

fn rate_limited_response() -> Response {
    let mut response = (
        StatusCode::TOO_MANY_REQUESTS,
        axum::Json(serde_json::json!({
            "error": { "code": "RATE_LIMITED", "message": "rate limited" }
        })),
    )
        .into_response();
    response
        .headers_mut()
        .insert("retry-after", HeaderValue::from_static("60"));
    response
}

#[cfg(test)]
mod tests {
    use super::{Bucket, Cidr, is_exempt};

    #[test]
    fn routes_land_in_the_bucket_that_matches_their_cost() {
        // A read of a resource is not the same cost as fetching it upstream.
        assert!(matches!(Bucket::of("/api/login/send-code"), Bucket::Auth));
        assert!(matches!(Bucket::of("/api/login/bilibili"), Bucket::Auth));
        assert!(matches!(Bucket::of("/api/auth/verify"), Bucket::Auth));
        assert!(matches!(
            Bucket::of("/api/dps/calculate"),
            Bucket::Expensive
        ));
        assert!(matches!(
            Bucket::of("/api/user/improvements"),
            Bucket::Expensive
        ));
        assert!(matches!(Bucket::of("/api/gacha/fetch"), Bucket::Upstream));
        assert!(matches!(
            Bucket::of("/api/operators/index"),
            Bucket::Default
        ));
        assert!(matches!(Bucket::of("/api/gacha/history"), Bucket::Default));
    }

    #[test]
    fn loopback_is_exempt_without_configuration() {
        assert!(is_exempt("127.0.0.1".parse().unwrap()));
        assert!(is_exempt("127.0.0.53".parse().unwrap()));
        assert!(is_exempt("::1".parse().unwrap()));
        assert!(!is_exempt("203.0.113.7".parse().unwrap()));
    }

    #[test]
    fn cidr_blocks_match_their_range_and_nothing_else() {
        let block = Cidr::parse("10.0.0.0/8").expect("parses");
        assert!(block.contains("10.1.2.3".parse().unwrap()));
        assert!(block.contains("10.255.255.255".parse().unwrap()));
        assert!(!block.contains("11.0.0.1".parse().unwrap()));

        let host = Cidr::parse("198.51.100.4").expect("a bare address is one host");
        assert!(host.contains("198.51.100.4".parse().unwrap()));
        assert!(!host.contains("198.51.100.5".parse().unwrap()));

        let six = Cidr::parse("2001:db8::/32").expect("parses");
        assert!(six.contains("2001:db8:1234::1".parse().unwrap()));
        assert!(!six.contains("2001:db9::1".parse().unwrap()));
    }

    #[test]
    fn an_ipv4_rule_matches_a_dual_stack_peer() {
        let block = Cidr::parse("172.16.0.0/12").expect("parses");
        assert!(block.contains("172.16.4.2".parse().unwrap()));
        assert!(
            block.contains("::ffff:172.16.4.2".parse().unwrap()),
            "a v4-mapped v6 peer must still match the v4 rule"
        );
    }

    #[test]
    fn a_zero_length_prefix_matches_its_whole_family() {
        let all_v4 = Cidr::parse("0.0.0.0/0").expect("parses");
        assert!(all_v4.contains("8.8.8.8".parse().unwrap()));
        assert!(!all_v4.contains("2001:db8::1".parse().unwrap()));
    }

    #[test]
    fn malformed_entries_are_rejected_rather_than_matching_everything() {
        for bad in ["", "nonsense", "10.0.0.0/33", "::/129", "10.0.0.0/x", "/8"] {
            assert!(Cidr::parse(bad).is_none(), "{bad:?} should not parse");
        }
    }

    #[test]
    fn only_the_default_bucket_follows_the_configured_rpm() {
        assert_eq!(Bucket::Default.allowance(250), 250);
        assert_eq!(Bucket::Auth.allowance(250), 10);
        assert_eq!(Bucket::Upstream.allowance(250), 6);
        assert_eq!(Bucket::Expensive.allowance(250), 20);
    }
}
