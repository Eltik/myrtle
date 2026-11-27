use std::sync::Arc;

use axum::Json;
use axum::body::Bytes;
use axum::http::{HeaderValue, header};
use axum::response::{IntoResponse, Response};
use myrtle_app::cache::CachedJson;
use serde::Serialize;

pub const GAMEDATA_CACHE: &str = "public, max-age=300";

#[derive(Debug, Clone, Copy, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct StatusOk {
    status: &'static str,
}

pub const fn ok() -> Json<StatusOk> {
    Json(StatusOk { status: "ok" })
}

struct Shared(Arc<str>);

impl AsRef<[u8]> for Shared {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct Cached {
    json: CachedJson,
    cache_control: Option<&'static str>,
}

impl Cached {
    #[must_use]
    pub const fn new(json: CachedJson) -> Self {
        Self {
            json,
            cache_control: None,
        }
    }

    #[must_use]
    pub const fn gamedata(json: CachedJson) -> Self {
        Self::new(json).cache_control(GAMEDATA_CACHE)
    }

    #[must_use]
    pub const fn cache_control(mut self, value: &'static str) -> Self {
        self.cache_control = Some(value);
        self
    }
}

impl From<CachedJson> for Cached {
    fn from(json: CachedJson) -> Self {
        Self::new(json)
    }
}

impl IntoResponse for Cached {
    fn into_response(self) -> Response {
        let mut response = Bytes::from_owner(Shared(self.json.body)).into_response();
        let headers = response.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        if let Ok(etag) = HeaderValue::from_str(&self.json.etag) {
            headers.insert(header::ETAG, etag);
        }
        if let Some(value) = self.cache_control {
            headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
        }
        response
    }
}
