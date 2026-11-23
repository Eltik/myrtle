use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use myrtle_types::PageRequest;
use serde::{Deserialize, Deserializer};

use crate::error::ApiError;

#[derive(Debug, Clone, Copy)]
pub struct Paging(pub PageRequest);

#[derive(Deserialize)]
struct Raw {
    #[serde(default, deserialize_with = "optional_u32")]
    limit: Option<u32>,
    #[serde(default, deserialize_with = "optional_u32")]
    offset: Option<u32>,
}

impl<S: Send + Sync> FromRequestParts<S> for Paging {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, ApiError> {
        let Query(Raw { limit, offset }) = Query::from_request_parts(parts, state)
            .await
            .map_err(|e| ApiError::bad_request(e.body_text()))?;
        Ok(Self(PageRequest { limit, offset }))
    }
}

pub fn optional_u32<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<u32>, D::Error> {
    let raw = Option::<String>::deserialize(deserializer)?;
    match raw.as_deref().map(str::trim) {
        None | Some("") => Ok(None),
        Some(text) => text.parse().map(Some).map_err(serde::de::Error::custom),
    }
}
