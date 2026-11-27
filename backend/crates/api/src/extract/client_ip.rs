use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};

use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::request::Parts;
use axum::http::{Extensions, HeaderMap};
use myrtle_app::AppContext;

#[derive(Debug, Clone, Copy)]
pub struct ClientIp(pub Option<IpAddr>);

impl FromRequestParts<AppContext> for ClientIp {
    type Rejection = Infallible;

    fn from_request_parts(
        parts: &mut Parts,
        ctx: &AppContext,
    ) -> impl Future<Output = Result<Self, Infallible>> + Send {
        let hops = usize::from(ctx.config.server.trusted_proxy_hops);
        std::future::ready(Ok(Self(peer_ip(&parts.headers, &parts.extensions, hops))))
    }
}

#[must_use]
pub fn peer_ip(headers: &HeaderMap, extensions: &Extensions, hops: usize) -> Option<IpAddr> {
    let forwarded = headers
        .get("x-forwarded-for")
        .filter(|_| hops > 0)
        .and_then(|v| v.to_str().ok())
        .and_then(|list| list.rsplit(',').nth(hops - 1)?.trim().parse().ok());
    forwarded.or_else(|| {
        extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| addr.ip())
    })
}
