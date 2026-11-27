#![allow(clippy::unwrap_used)]

use std::net::IpAddr;

use axum::http::{HeaderMap, HeaderValue, header};
use myrtle_api::middleware::rate_limit::{Bucket, Cidr, RateLimiter, bucket_for};
use myrtle_app::services::accounts::jwt::create_token;
use myrtle_types::{Role, Server, UserId};
use uuid::Uuid;

const SECRET: &str = "bucket-test-secret";
const UPSTREAM: [&str; 4] = [
    "/api/gacha/fetch",
    "/api/refresh",
    "/api/friends",
    "/api/players/search",
];

fn ip(s: &str) -> IpAddr {
    s.parse().unwrap()
}

fn bearer(secret: &str) -> HeaderMap {
    let token = create_token(
        secret,
        UserId(Uuid::from_u128(1)),
        "1",
        Server::EN,
        Role::User,
        chrono::Utc::now().timestamp() as u64,
    )
    .unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
    );
    headers
}

#[test]
fn routes_fall_in_their_buckets() {
    assert_eq!(Bucket::of("/api/login"), Bucket::Auth);
    assert_eq!(Bucket::of("/api/login/bilibili"), Bucket::Auth);
    assert_eq!(Bucket::of("/api/auth/verify"), Bucket::Auth);
    assert_eq!(Bucket::of("/api/dps/calculate"), Bucket::Expensive);
    assert_eq!(Bucket::of("/api/base/optimize"), Bucket::Expensive);
    assert_eq!(Bucket::of("/api/operators/{id}"), Bucket::Default);
    assert_eq!(Bucket::of("/api/dps/calculate/x"), Bucket::Default);
    for route in UPSTREAM {
        assert_eq!(Bucket::of(route), Bucket::Upstream, "{route}");
    }
}

#[test]
fn allowances_are_per_bucket_and_only_default_is_configured() {
    assert_eq!(Bucket::Auth.allowance(600), 10);
    assert_eq!(Bucket::Expensive.allowance(600), 20);
    assert_eq!(Bucket::Upstream.allowance(600), 6);
    assert_eq!(Bucket::Default.allowance(600), 600);
}

#[test]
fn the_upstream_bucket_needs_a_token_this_server_signed() {
    for route in UPSTREAM {
        assert_eq!(bucket_for(route, &bearer(SECRET), SECRET), Bucket::Upstream);
        assert_eq!(
            bucket_for(route, &HeaderMap::new(), SECRET),
            Bucket::Default
        );
        assert_eq!(
            bucket_for(route, &bearer("another-secret"), SECRET),
            Bucket::Default
        );
    }
    assert_eq!(
        bucket_for("/api/dps/calculate", &HeaderMap::new(), SECRET),
        Bucket::Expensive
    );
}

#[test]
fn cidrs_parse_and_match() {
    let net = Cidr::parse("10.1.0.0/16").unwrap();
    assert!(net.contains(ip("10.1.255.3")));
    assert!(!net.contains(ip("10.2.0.1")));
    assert!(net.contains(ip("::ffff:10.1.0.9")), "v4-mapped v6");

    let host = Cidr::parse(" 192.168.1.7 ").unwrap();
    assert!(host.contains(ip("192.168.1.7")));
    assert!(!host.contains(ip("192.168.1.8")));

    let everything = Cidr::parse("0.0.0.0/0").unwrap();
    assert!(everything.contains(ip("8.8.8.8")));
    assert!(!everything.contains(ip("::1")));

    let v6 = Cidr::parse("2001:db8::/32").unwrap();
    assert!(v6.contains(ip("2001:db8:1::5")));
    assert!(!v6.contains(ip("2001:db9::1")));
    assert!(!v6.contains(ip("10.0.0.1")));

    for bad in ["", "nope", "10.0.0.0/33", "::/129", "10.0.0.0/x"] {
        assert_eq!(Cidr::parse(bad), None, "{bad}");
    }
}

#[test]
fn the_limiter_refuses_the_hit_past_the_allowance() {
    let limiter = RateLimiter::new(3, 0, &[]);
    let (a, b) = (ip("203.0.113.1"), ip("203.0.113.2"));
    for hit in 1..=3 {
        assert!(!limiter.over(Bucket::Default, a), "hit {hit}");
    }
    assert!(limiter.over(Bucket::Default, a));
    assert!(!limiter.over(Bucket::Default, b), "another address");
    assert!(!limiter.over(Bucket::Auth, a), "another bucket");
}
