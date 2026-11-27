use std::convert::Infallible;
use std::ops::Deref;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{HeaderMap, header};
use myrtle_app::services::accounts::{self, jwt};
use myrtle_app::{AppContext, AppError, Subject};
use subtle::ConstantTimeEq;

use crate::error::ApiError;

const SERVICE_KEY: &str = "x-service-key";

#[derive(Debug, Clone)]
pub struct Caller(pub Subject);

impl Deref for Caller {
    type Target = Subject;

    fn deref(&self) -> &Subject {
        &self.0
    }
}

impl FromRequestParts<AppContext> for Caller {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, ctx: &AppContext) -> Result<Self, ApiError> {
        if is_service_call(&parts.headers, ctx) {
            return Ok(Self(Subject::System));
        }
        let token = bearer(&parts.headers).ok_or(AppError::Unauthorized)?;
        let claims = jwt::verify_token(&ctx.secrets.jwt, token)?;
        Ok(Self(accounts::subject(ctx, &claims).await?))
    }
}

#[derive(Debug, Clone)]
pub struct MaybeCaller(pub Subject);

impl Deref for MaybeCaller {
    type Target = Subject;

    fn deref(&self) -> &Subject {
        &self.0
    }
}

impl FromRequestParts<AppContext> for MaybeCaller {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, ctx: &AppContext) -> Result<Self, Infallible> {
        Ok(Self(
            accounts::subject_optional(ctx, bearer(&parts.headers)).await,
        ))
    }
}

pub fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

#[must_use]
pub fn is_service_call(headers: &HeaderMap, ctx: &AppContext) -> bool {
    let Some(configured) = ctx.secrets.service_key.as_deref().filter(|k| !k.is_empty()) else {
        return false;
    };
    headers
        .get(SERVICE_KEY)
        .is_some_and(|key| key.as_bytes().ct_eq(configured.as_bytes()).into())
}
