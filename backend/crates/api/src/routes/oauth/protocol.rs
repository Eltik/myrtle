use axum::Json;
use axum::extract::rejection::FormRejection;
use axum::extract::{Form, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use myrtle_app::AppContext;
use myrtle_app::services::oauth::error::ProtocolError;
use myrtle_app::services::oauth::{protocol, tokens};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::oauth_error::OAuthResult;
use crate::router::ApiRouter;

const NO_STORE: HeaderValue = HeaderValue::from_static("no-store");

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/oauth/token", post(token))
        .route("/oauth/revoke", post(revoke))
        .route("/oauth/userinfo", get(userinfo))
}

#[derive(Deserialize)]
struct TokenRequest {
    grant_type: Option<String>,
    code: Option<String>,
    redirect_uri: Option<String>,
    code_verifier: Option<String>,
    refresh_token: Option<String>,
    scope: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
}

#[derive(Deserialize)]
struct RevokeRequest {
    token: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
}

fn no_store<T: Serialize>(body: T) -> Response {
    ([(header::CACHE_CONTROL, NO_STORE)], Json(body)).into_response()
}

fn form_body<T>(form: Result<Form<T>, FormRejection>) -> Result<T, ProtocolError> {
    form.map(|Form(body)| body).map_err(|_| {
        ProtocolError::invalid_request(
            "the body must be application/x-www-form-urlencoded, each parameter at most once",
        )
    })
}

fn authorization(headers: &HeaderMap) -> Option<&str> {
    headers.get(header::AUTHORIZATION)?.to_str().ok()
}

async fn authenticate(
    ctx: &AppContext,
    headers: &HeaderMap,
    client_id: Option<&str>,
    client_secret: Option<&str>,
) -> Result<Uuid, ProtocolError> {
    tokens::authenticate(ctx, authorization(headers), client_id, client_secret).await
}

fn required<'a>(value: Option<&'a str>, name: &str) -> Result<&'a str, ProtocolError> {
    value
        .filter(|v| !v.is_empty())
        .ok_or_else(|| ProtocolError::invalid_request(format!("{name} is required")))
}

async fn token(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    form: Result<Form<TokenRequest>, FormRejection>,
) -> OAuthResult<Response> {
    let body = form_body(form)?;
    let app_id = authenticate(
        &ctx,
        &headers,
        body.client_id.as_deref(),
        body.client_secret.as_deref(),
    )
    .await?;
    let issued = match body.grant_type.as_deref() {
        Some("authorization_code") => {
            let code = required(body.code.as_deref(), "code")?;
            tokens::exchange_code(
                &ctx,
                app_id,
                code,
                body.redirect_uri.as_deref(),
                body.code_verifier.as_deref(),
            )
            .await?
        }
        Some("refresh_token") => {
            let refresh_token = required(body.refresh_token.as_deref(), "refresh_token")?;
            tokens::refresh(&ctx, app_id, refresh_token, body.scope.as_deref()).await?
        }
        Some(_) => return Err(ProtocolError::unsupported_grant_type().into()),
        None => return Err(ProtocolError::invalid_request("grant_type is required").into()),
    };
    Ok(no_store(issued))
}

async fn revoke(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    form: Result<Form<RevokeRequest>, FormRejection>,
) -> OAuthResult<Response> {
    let body = form_body(form)?;
    let app_id = authenticate(
        &ctx,
        &headers,
        body.client_id.as_deref(),
        body.client_secret.as_deref(),
    )
    .await?;
    let token = required(body.token.as_deref(), "token")?;
    tokens::revoke(&ctx, app_id, token).await?;
    Ok((StatusCode::OK, [(header::CACHE_CONTROL, NO_STORE)]).into_response())
}

async fn userinfo(State(ctx): State<AppContext>, headers: HeaderMap) -> OAuthResult<Response> {
    let access_token =
        protocol::bearer_token(authorization(&headers)).ok_or_else(ProtocolError::invalid_token)?;
    let text = |name: &str| headers.get(name).and_then(|v| v.to_str().ok());
    let api_base = protocol::api_base(
        &ctx.config.server.public_api_url,
        text("host"),
        text("x-forwarded-proto"),
    );
    Ok(no_store(
        tokens::userinfo(&ctx, access_token, &api_base).await?,
    ))
}
