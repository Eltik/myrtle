use std::convert::Infallible;

use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use myrtle_app::AppContext;
use myrtle_types::Server;
use serde::Deserialize;

use crate::server_prefix::PrefixServer;

/// The `/api/{server}` prefix, then a `?server=` naming one, then the default server
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerSel(pub Server);

#[derive(Deserialize)]
struct Named {
    server: Option<String>,
}

impl FromRequestParts<AppContext> for ServerSel {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, ctx: &AppContext) -> Result<Self, Infallible> {
        if let Some(PrefixServer(server)) = parts.extensions.get() {
            return Ok(Self(*server));
        }
        let named = Query::<Named>::from_request_parts(parts, ctx)
            .await
            .ok()
            .and_then(|Query(named)| Server::parse(named.server.as_deref()?));
        Ok(Self(
            named.unwrap_or_else(|| ctx.snapshots.default_server()),
        ))
    }
}
