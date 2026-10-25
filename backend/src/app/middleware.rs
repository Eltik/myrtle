//! Cross-cutting request middleware: observation, timeout, rate limiting.
//!
//! Plain `from_fn` layers rather than third-party tower layers: all three need the
//! matched route, request id and client address, and deriving them in one place
//! beats keeping three layers in agreement.

use std::net::{IpAddr, SocketAddr};
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, MatchedPath, Request, State};
use axum::http::{Extensions, HeaderMap, HeaderValue, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use dashmap::DashMap;
use subtle::ConstantTimeEq;
use tracing::Instrument;

use crate::app::metrics::METRICS;
use crate::app::state::AppState;
use crate::core::auth::jwt::verify_token;

/// Header carrying the id that ties a log line to a response.
const REQUEST_ID: &str = "x-request-id";

/// Process start, so window arithmetic is on cheap `u64` seconds rather than
/// `Instant`s held inside the map.
static START: LazyLock<Instant> = LazyLock::new(Instant::now);

fn now_secs() -> u64 {
    START.elapsed().as_secs()
}

/// Ceiling on how long a handler may take to produce a response.
///
/// This bounds the handler future, not the response body: a streamed response
/// is produced as soon as the stream exists, so a slow download is unaffected.
const HANDLER_TIMEOUT: Duration = Duration::from_secs(30);

/// The matched route (`/api/operators/{id}`), when the router resolved one.
///
/// [`metric_label`] collapses unmatched requests into one series to keep labels
/// bounded; [`classify_path`] falls back to the real path because the rate
/// limiter derives an allowance from it.
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

    // DEBUG, not INFO: production runs `backend=info` and this line was its whole
    // log volume, while route/status/latency are already metrics and the id is on
    // the response header. Server errors log at WARN.
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

/// Proxies in front of this process. 0 (default): the peer is the client and
/// `X-Forwarded-For` is ignored. Behind one reverse proxy, set 1.
///
/// Hops count from the RIGHT: only entries a trusted proxy appended are
/// trustworthy, the leftmost are whatever the client sent.
static TRUSTED_PROXY_HOPS: LazyLock<usize> = LazyLock::new(|| {
    std::env::var("TRUSTED_PROXY_HOPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
});

fn client_ip(req: &Request) -> Option<IpAddr> {
    peer_ip(req.headers(), req.extensions())
}

/// The client address the rate limiter keys on: the `X-Forwarded-For` entry
/// `TRUSTED_PROXY_HOPS` points at, else the socket peer.
pub(crate) fn peer_ip(headers: &HeaderMap, extensions: &Extensions) -> Option<IpAddr> {
    let hops = *TRUSTED_PROXY_HOPS;
    if hops > 0
        && let Some(forwarded) = headers.get("x-forwarded-for")
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

    extensions
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
/// Loopback is always exempt, so a same-host SSR renderer isn't throttled as one
/// very busy visitor. Set this when the renderer reaches us from a routable
/// address (another host, a container network).
///
/// Only list addresses untrusted clients can't reach: an exempt address has no
/// allowance at all.
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Bucket {
    /// Triggers a message from an upstream provider, or mints a token.
    Auth,
    /// Runs a combinatorial search or a simulation in this process.
    Expensive,
    /// Pulls from an upstream game API on the caller's behalf.
    Upstream,
    Default,
}

/// Every route outside the Auth prefixes whose bucket is not `Default`, keyed by
/// the path axum matched (the `OpenAPI` spelling, under `/api`). [`Bucket::of`]
/// reads nothing else, so this is the one place a route's cost is declared.
///
/// A route missing from here is `Default`, so a new compute route used to land
/// there silently: the batch, rank and supporter routes did (100 rpm against 20
/// for one `/dps/calculate`). `tests::route_costs` walks the served router and
/// every CPU-pool call site and fails when a route that runs an engine is not
/// `Expensive` (or stricter), or when a row here names no route.
const ROUTE_BUCKETS: &[(&str, Bucket)] = &[
    // The DPS and HPS simulators. One batch is up to `MAX_BATCH` 64 requests x
    // `MAX_SWEEP_STEPS` 101 points = 6,464 calculations, one rank miss every
    // (operator, skill, module) the simulator supports; a single calculate is 1.
    ("/api/dps/calculate", Bucket::Expensive),
    ("/api/hps/calculate", Bucket::Expensive),
    ("/api/dps/batch", Bucket::Expensive),
    ("/api/hps/batch", Bucket::Expensive),
    ("/api/dps/rank", Bucket::Expensive),
    ("/api/dps/supporters", Bucket::Expensive),
    // The base optimizer, rotation search and layout scorer.
    ("/api/base/optimize", Bucket::Expensive),
    ("/api/base/rotation", Bucket::Expensive),
    ("/api/base/evaluate", Bucket::Expensive),
    // Scores the stationed base on every call (0.2 s, the same scorer as
    // `/base/evaluate`), for any public profile's `uid`.
    ("/api/user/max-level-cost", Bucket::Expensive),
    ("/api/user/improvements", Bucket::Expensive),
    // The squad analysis runs on the posted squad, uncached.
    ("/api/squads/analyze", Bucket::Expensive),
    // Each call pulls from Hypergryph with the caller's game session.
    ("/api/gacha/fetch", Bucket::Upstream),
    ("/api/refresh", Bucket::Upstream),
    ("/api/friends", Bucket::Upstream),
    ("/api/players/search", Bucket::Upstream),
];

impl Bucket {
    fn of(route: &str) -> Self {
        if route.starts_with("/api/login") || route.starts_with("/api/auth/") {
            return Self::Auth;
        }
        ROUTE_BUCKETS
            .iter()
            .find_map(|&(path, bucket)| (path == route).then_some(bucket))
            .unwrap_or(Self::Default)
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

/// The bucket a request counts against.
///
/// Every `Upstream` route takes `AuthUser`, which refuses a request without a
/// valid bearer token (401) before the handler can call upstream. Such a
/// request is a cheap rejection, so it counts in `Default`. Counted in
/// `Upstream`, anonymous calls from an address spent that address's 6 upstream
/// calls a minute, so they could lock a signed-in user on the same address out
/// of their roster refresh. A caller with a valid token counts in `Upstream` as
/// before, whatever its handler then does. (The service key skips the limiter
/// before this is asked.)
fn bucket_for(route: &str, headers: &HeaderMap, jwt_secret: &str) -> Bucket {
    let bucket = Bucket::of(route);
    if bucket == Bucket::Upstream && !carries_valid_token(headers, jwt_secret) {
        return Bucket::Default;
    }
    bucket
}

/// The same check `AuthUser` makes: a `Bearer` token this process signed and
/// that has not expired. One HMAC, and only on the `Upstream` routes.
fn carries_valid_token(headers: &HeaderMap, jwt_secret: &str) -> bool {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|token| verify_token(jwt_secret, token).is_ok())
}

struct Window {
    /// Index of the 60-second window this count belongs to.
    window: AtomicU64,
    hits: AtomicU64,
}

static BUCKETS: LazyLock<DashMap<(Bucket, IpAddr), Window>> = LazyLock::new(DashMap::new);
static SWEEP_TICKER: AtomicU64 = AtomicU64::new(0);

/// Drops expired entries every 10k admissions. Inline rather than a spawned task:
/// one moving part, ~free per request. Without it the map keeps every address seen.
fn maybe_sweep(current_window: u64) {
    if SWEEP_TICKER
        .fetch_add(1, Ordering::Relaxed)
        .is_multiple_of(10_000)
    {
        BUCKETS.retain(|_, w| w.window.load(Ordering::Relaxed) >= current_window);
    }
}

/// The caller presented the internal service key (frontend SSR). Those calls all
/// arrive from one address, so a per-IP allowance would throttle the site itself.
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

    if is_service_call(&req, &state) {
        return next.run(req).await;
    }

    let Some(ip) = client_ip(&req) else {
        // No usable client address (misconfig, or a transport without one).
        // Refusing would take the site down over a deployment detail, so it passes.
        return next.run(req).await;
    };

    // Loopback and anything in `RATE_LIMIT_EXEMPT_IPS`: first-party callers
    // whose traffic is the sum of many real visitors, so a per-address
    // allowance would throttle all of them together.
    if is_exempt(ip) {
        return next.run(req).await;
    }

    let bucket = bucket_for(&route, req.headers(), &state.config.jwt_secret);
    let window = now_secs() / 60;
    let allowance = bucket.allowance(state.config.rate_limit_rpm);
    maybe_sweep(window);

    let entry = BUCKETS.entry((bucket, ip)).or_insert_with(|| Window {
        window: AtomicU64::new(window),
        hits: AtomicU64::new(0),
    });

    // Rolling the window and counting are two atomics, so two requests right at the
    // minute turn can both count as first of the new window: a few extra admissions
    // under a race. Deliberate: two `Relaxed` atomics cost nothing, and a mutex or a
    // true sliding window buys precision a rate limiter doesn't need.
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
    use super::{Bucket, Cidr, bucket_for, is_exempt};
    use crate::core::auth::jwt::create_token;
    use axum::http::{HeaderMap, HeaderValue, header};

    const SECRET: &str = "bucket-test-secret";
    const UPSTREAM: [&str; 4] = [
        "/api/gacha/fetch",
        "/api/refresh",
        "/api/friends",
        "/api/players/search",
    ];

    fn bearer(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).expect("header value"),
        );
        headers
    }

    fn token(secret: &str) -> String {
        create_token(
            secret,
            "00000000-0000-4000-8000-000000000001",
            "1",
            "en",
            "user",
            1,
        )
        .expect("token")
    }

    #[test]
    fn only_a_caller_the_handler_admits_spends_the_upstream_allowance() {
        let signed_in = bearer(&token(SECRET));
        let foreign = bearer(&token("some-other-secret"));
        let garbage = bearer("not-a-jwt");
        for route in UPSTREAM {
            assert!(
                matches!(bucket_for(route, &signed_in, SECRET), Bucket::Upstream),
                "{route}"
            );
            // Each of these is a 401 from `AuthUser` before any upstream call.
            for headers in [&HeaderMap::new(), &foreign, &garbage] {
                assert!(
                    matches!(bucket_for(route, headers, SECRET), Bucket::Default),
                    "{route}"
                );
            }
        }
        // The other buckets do not depend on the caller.
        assert!(matches!(
            bucket_for("/api/login/send-code", &HeaderMap::new(), SECRET),
            Bucket::Auth
        ));
        assert!(matches!(
            bucket_for("/api/dps/calculate", &HeaderMap::new(), SECRET),
            Bucket::Expensive
        ));
        assert!(matches!(
            bucket_for("/api/operators/index", &signed_in, SECRET),
            Bucket::Default
        ));
    }

    #[test]
    fn routes_land_in_the_bucket_that_matches_their_cost() {
        // A read of a resource is not the same cost as fetching it upstream.
        assert!(matches!(Bucket::of("/api/login/send-code"), Bucket::Auth));
        assert!(matches!(Bucket::of("/api/login/bilibili"), Bucket::Auth));
        assert!(matches!(Bucket::of("/api/auth/verify"), Bucket::Auth));
        // Every route that runs a simulator, an optimizer or a scorer on the
        // caller's input; `route_costs` derives the full set.
        for route in [
            "/api/dps/calculate",
            "/api/hps/calculate",
            "/api/dps/batch",
            "/api/hps/batch",
            "/api/dps/rank",
            "/api/dps/supporters",
            "/api/base/optimize",
            "/api/base/rotation",
            "/api/base/evaluate",
            "/api/user/improvements",
        ] {
            assert_eq!(Bucket::of(route), Bucket::Expensive, "{route}");
        }
        assert!(matches!(Bucket::of("/api/gacha/fetch"), Bucket::Upstream));
        assert!(matches!(Bucket::of("/api/refresh"), Bucket::Upstream));
        // Both call Hypergryph with the caller's game session on every request.
        assert!(matches!(Bucket::of("/api/friends"), Bucket::Upstream));
        assert!(matches!(
            Bucket::of("/api/players/search"),
            Bucket::Upstream
        ));
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

    /// Ties every route's bucket to what its handler costs, from the served
    /// router and the CPU-pool call sites, so a route that runs an engine cannot
    /// land in `Default` without a test failing.
    ///
    /// The route list is derived, not picked by hand: every path in the
    /// `OpenAPI` document `api_parts` builds alongside the router `run` serves
    /// (every `/api` handler is a `routes!(..)` with a `#[utoipa::path]`, see
    /// `routes::router`), and every handler's source, read the way
    /// `tests/admin_route_guards_test.rs` reads it. The cost side is derived too:
    /// every `cpu::run`, `cpu::offload`, `cpu::admit` and `cpu::admit_light` call
    /// under `src/` names a job kind, and [`JOBS`] must classify exactly that set.
    mod route_costs {
        use std::collections::{BTreeMap, BTreeSet};
        use std::fs;
        use std::path::{Path, PathBuf};

        use super::super::{Bucket, ROUTE_BUCKETS};
        use crate::app::server::api_parts;

        /// How a CPU-pool job is paid for, as the rate limiter sees it.
        enum Paid {
            /// Runs an engine (a simulator, an optimizer, a scorer, an analysis)
            /// on the caller's input on every uncached call. Each route that
            /// reaches it must be `Expensive` or stricter. `called_as` names the
            /// service entry a handler calls when the job is not started in the
            /// handler itself, so a second route calling it is caught too.
            Engine {
                routes: &'static [&'static str],
                called_as: &'static [&'static str],
            },
            /// Runs on every call but is a parse or a lookup, not an engine, so
            /// its routes stay where they are. The text says why.
            Light(&'static str),
            /// Built once per game-data load, or once per file on disk, and then
            /// served from memory or disk: a caller cannot make it run again.
            Cached,
            /// No request waits on it.
            Background,
        }

        const fn engine(routes: &'static [&'static str]) -> Paid {
            Paid::Engine {
                routes,
                called_as: &[],
            }
        }

        /// Every CPU-pool job kind under `src/`.
        const JOBS: &[(&str, Paid)] = &[
            ("dps_calculate", engine(&["/api/dps/calculate"])),
            ("hps_calculate", engine(&["/api/hps/calculate"])),
            ("dps_batch", engine(&["/api/dps/batch"])),
            ("hps_batch", engine(&["/api/hps/batch"])),
            ("dps_rank", engine(&["/api/dps/rank"])),
            ("dps_supporters", engine(&["/api/dps/supporters"])),
            ("base_evaluate", engine(&["/api/base/evaluate"])),
            ("base_optimize", engine(&["/api/base/optimize"])),
            ("base_rotation", engine(&["/api/base/rotation"])),
            (
                "max_level_income",
                Paid::Engine {
                    routes: &["/api/user/max-level-cost"],
                    called_as: &["max_level_costs("],
                },
            ),
            (
                "user_improvements",
                Paid::Engine {
                    routes: &["/api/user/improvements"],
                    called_as: &["get_improvements("],
                },
            ),
            (
                "squad_analyze",
                Paid::Engine {
                    routes: &["/api/squads/analyze"],
                    called_as: &["squad::analyze("],
                },
            ),
            // Grades the refreshed roster; `/api/refresh` is `Upstream` (6 rpm).
            (
                "user_grade",
                Paid::Engine {
                    routes: &["/api/refresh"],
                    called_as: &["roster::refresh("],
                },
            ),
            (
                "sync_story_census",
                Paid::Light("parses the caller's own game payload inside `/api/refresh`"),
            ),
            (
                "story_reverdict",
                Paid::Light("re-reads the caller's own stored story rows; signed in only"),
            ),
            (
                "story_parse",
                Paid::Light("one script; the longest EN script parses in under 10 ms"),
            ),
            (
                "story_illustrations",
                Paid::Light("a lookup in the cached story index; no script is read"),
            ),
            ("dps_data_identity", Paid::Cached),
            ("archives", Paid::Cached),
            ("archives_json", Paid::Cached),
            ("music_index", Paid::Cached),
            ("static_resource", Paid::Cached),
            ("story_index", Paid::Cached),
            ("story_sprites", Paid::Cached),
            ("story_gallery", Paid::Cached),
            ("story_art_gallery", Paid::Cached),
            ("story_community_plan", Paid::Cached),
            ("story_community_batch", Paid::Cached),
            ("story_gallery_variant", Paid::Cached),
            ("story_sprite_thumb", Paid::Cached),
            ("event_banner", Paid::Cached),
            ("story_sprite_variant_thumb", Paid::Cached),
            ("dps_retention", Paid::Background),
            ("regrade", Paid::Background),
        ];

        /// The `cpu` entry points, without their `cpu::` prefix so this file
        /// does not match its own search.
        const CPU_FNS: [&str; 4] = ["run(", "offload(", "admit(", "admit_light("];

        fn src_dir() -> PathBuf {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
        }

        fn rust_files(dir: &Path) -> Vec<PathBuf> {
            let mut out = Vec::new();
            let mut queue = vec![dir.to_path_buf()];
            while let Some(dir) = queue.pop() {
                for entry in fs::read_dir(&dir).expect("source dir is readable") {
                    let path = entry.expect("readable dir entry").path();
                    if path.is_dir() {
                        queue.push(path);
                    } else if path.extension().is_some_and(|e| e == "rs") {
                        out.push(path);
                    }
                }
            }
            out.sort();
            out
        }

        /// The job kind of every CPU-pool call in `src`, or the text after the
        /// call when the kind is not a string literal (which [`JOBS`] could not
        /// see, so it fails the test).
        fn job_kinds(src: &str) -> Vec<Result<String, String>> {
            let mut out = Vec::new();
            for f in CPU_FNS {
                let call = format!("cpu::{f}");
                for (at, _) in src.match_indices(&call) {
                    let rest = src[at + call.len()..].trim_start();
                    let kind = rest
                        .strip_prefix('"')
                        .and_then(|r| r.split('"').next())
                        .map(str::to_owned);
                    out.push(kind.ok_or_else(|| rest.chars().take(40).collect()));
                }
            }
            out
        }

        /// Every path the served router answers under `/api`.
        fn served_paths() -> BTreeSet<String> {
            let (_router, api) = api_parts();
            api.paths.paths.keys().cloned().collect()
        }

        fn is_fn_start(line: &str) -> bool {
            [
                "pub async fn ",
                "async fn ",
                "pub fn ",
                "pub(crate) async fn ",
            ]
            .iter()
            .any(|p| line.starts_with(p))
        }

        /// Every annotated handler's source, from its `fn` line to its closing
        /// brace, keyed by the path as the document spells it (several methods
        /// on one path are joined: a bucket is per path).
        fn handler_sources() -> BTreeMap<String, String> {
            let mut out: BTreeMap<String, String> = BTreeMap::new();
            for file in rust_files(&src_dir().join("app/routes")) {
                let src = fs::read_to_string(&file).expect("route module is readable");
                let lines: Vec<&str> = src.lines().collect();
                let mut i = 0;
                while i < lines.len() {
                    if lines[i].trim() != "#[utoipa::path(" {
                        i += 1;
                        continue;
                    }
                    let mut j = i + 1;
                    let mut path = None;
                    while j < lines.len() && !lines[j].trim_start().starts_with(")]") {
                        if path.is_none() {
                            path = lines[j]
                                .trim()
                                .strip_prefix("path = \"")
                                .and_then(|rest| rest.split('"').next());
                        }
                        j += 1;
                    }
                    while j < lines.len() && !is_fn_start(lines[j]) {
                        j += 1;
                    }
                    let start = j;
                    while j < lines.len() && lines[j] != "}" {
                        j += 1;
                    }
                    if let Some(path) = path {
                        let end = j.min(lines.len().saturating_sub(1));
                        let body = out.entry(format!("/api{path}")).or_default();
                        body.push_str(&lines[start..=end].join("\n"));
                        body.push('\n');
                    }
                    i = j + 1;
                }
            }
            out
        }

        /// At most `Expensive`'s allowance, whatever `RATE_LIMIT_RPM` says.
        fn at_least_expensive(route: &str) -> bool {
            Bucket::of(route).allowance(u32::MAX) <= Bucket::Expensive.allowance(u32::MAX)
        }

        #[test]
        fn jobs_names_every_cpu_pool_job_exactly_once() {
            let mut found = BTreeSet::new();
            let mut unreadable = Vec::new();
            for file in rust_files(&src_dir()) {
                if file.ends_with("app/cpu.rs") {
                    continue;
                }
                let src = fs::read_to_string(&file).expect("source is readable");
                for kind in job_kinds(&src) {
                    match kind {
                        Ok(kind) => {
                            found.insert(kind);
                        }
                        Err(rest) => unreadable.push(format!("{}: {rest}", file.display())),
                    }
                }
            }
            assert!(
                unreadable.is_empty(),
                "a CPU job kind must be a string literal: {unreadable:#?}"
            );
            for (kind, paid) in JOBS {
                if let Paid::Light(why) = paid {
                    assert!(!why.is_empty(), "{kind}: say why it stays where it is");
                }
            }
            let listed: BTreeSet<String> = JOBS.iter().map(|(k, _)| (*k).to_owned()).collect();
            assert_eq!(listed.len(), JOBS.len(), "a job kind is listed twice");
            let missing: Vec<_> = found.difference(&listed).collect();
            let stale: Vec<_> = listed.difference(&found).collect();
            assert!(
                missing.is_empty() && stale.is_empty(),
                "JOBS is out of step with the CPU-pool call sites.\n\
                 Started in src but no row (say how it is paid for): {missing:#?}\n\
                 A row but no call site: {stale:#?}"
            );
        }

        #[test]
        fn every_route_that_runs_an_engine_is_expensive_or_stricter() {
            let served = served_paths();
            let handlers = handler_sources();
            let mut wrong = Vec::new();
            for (kind, paid) in JOBS {
                let Paid::Engine { routes, called_as } = paid else {
                    continue;
                };
                for route in *routes {
                    if !served.contains(*route) {
                        wrong.push(format!("{kind}: {route} is not a served route"));
                    } else if !at_least_expensive(route) {
                        wrong.push(format!(
                            "{kind}: {route} is {:?}, below Expensive",
                            Bucket::of(route)
                        ));
                    }
                }
                // A handler that starts the job, or calls its service entry,
                // reaches the engine whatever its row says.
                for (route, src) in &handlers {
                    let starts_it = job_kinds(src).iter().any(|k| k.as_deref() == Ok(*kind));
                    let calls_it = called_as.iter().any(|c| src.contains(c));
                    if (starts_it || calls_it) && !routes.contains(&route.as_str()) {
                        wrong.push(format!(
                            "{route} runs `{kind}` but is not in its row (bucket {:?})",
                            Bucket::of(route)
                        ));
                    }
                }
            }
            assert!(wrong.is_empty(), "{wrong:#?}");
        }

        #[test]
        fn every_handler_that_starts_a_cpu_job_names_a_classified_kind() {
            let listed: BTreeSet<&str> = JOBS.iter().map(|(k, _)| *k).collect();
            let mut wrong = Vec::new();
            for (route, src) in handler_sources() {
                for kind in job_kinds(&src) {
                    match kind {
                        Ok(kind) if listed.contains(kind.as_str()) => {}
                        other => wrong.push(format!("{route}: {other:?}")),
                    }
                }
            }
            assert!(wrong.is_empty(), "{wrong:#?}");
        }

        #[test]
        fn every_route_bucket_row_names_a_served_route() {
            let served = served_paths();
            let mut seen = BTreeSet::new();
            for (route, bucket) in ROUTE_BUCKETS {
                assert!(served.contains(*route), "{route} is not a served route");
                assert!(seen.insert(*route), "{route} is listed twice");
                // Auth is decided by prefix before the table is read.
                assert_eq!(Bucket::of(route), *bucket, "{route} is shadowed");
                assert_ne!(*bucket, Bucket::Default, "{route}: Default needs no row");
            }
        }
    }
}
