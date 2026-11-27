use std::borrow::Cow;

use axum::extract::Request;
use axum::http::Uri;
use myrtle_types::Server;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrefixServer(pub Server);

pub fn strip_server(mut req: Request) -> Request {
    let (Some(server), path) = split_server(req.uri().path()) else {
        return req;
    };
    let rewritten = match req.uri().query() {
        Some(query) => format!("{path}?{query}"),
        None => path.into_owned(),
    };
    let mut parts = req.uri().clone().into_parts();
    parts.path_and_query = rewritten.parse().ok();
    if let Ok(uri) = Uri::from_parts(parts) {
        *req.uri_mut() = uri;
        req.extensions_mut().insert(PrefixServer(server));
    }
    req
}

#[must_use]
pub fn split_server(path: &str) -> (Option<Server>, Cow<'_, str>) {
    let Some(rest) = path.strip_prefix("/api/") else {
        return (None, Cow::Borrowed(path));
    };
    let (first, tail) = rest.split_once('/').unwrap_or((rest, ""));
    match Server::parse(first) {
        Some(server) => (Some(server), Cow::Owned(format!("/api/{tail}"))),
        None => (None, Cow::Borrowed(path)),
    }
}
