use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::routing::{get, post};
use myrtle_app::privacy::{self, UpdateSettingsRequest};
use myrtle_app::services::accounts::account::{self, VerifySession};
use myrtle_app::services::accounts::game_session::{self, DisconnectResult};
use myrtle_app::services::accounts::jwt::{self, Claims};
use myrtle_app::services::accounts::login::{self, LoginResponse, parse_server};
use myrtle_app::services::sync;
use myrtle_app::{AppContext, AppError};
use serde::Deserialize;
use serde_json::Value;

use crate::error::ApiResult;
use crate::extract::auth::{bearer, is_service_call};
use crate::extract::{self, Caller};
use crate::respond::{StatusOk, ok};
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/login/send-code", post(send_code))
        .route("/login", post(login))
        .route("/login/bilibili", post(login_bilibili))
        .route("/login/bilibili/send-code", post(send_bilibili_sms))
        .route("/login/bilibili/sms", post(login_bilibili_sms))
        .route("/login/cn/send-code", post(send_code_cn))
        .route("/login/cn", post(login_cn))
        .route("/auth/verify", get(verify))
        .route("/auth/update-settings", post(update_settings))
        .route("/auth/disconnect", post(disconnect))
        .route("/auth/delete-account", post(delete_account))
        .route("/refresh", post(refresh))
}

const fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
struct SendCodeRequest {
    email: String,
    server: String,
}

async fn send_code(
    State(ctx): State<AppContext>,
    extract::Json(body): extract::Json<SendCodeRequest>,
) -> ApiResult<Json<StatusOk>> {
    let server = parse_server(&body.server)?;
    login::send_code(&ctx, &body.email, server).await?;
    Ok(ok())
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    code: String,
    server: String,
    #[serde(default = "default_true")]
    save_credentials: bool,
}

async fn login(
    State(ctx): State<AppContext>,
    extract::Json(body): extract::Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let server = parse_server(&body.server)?;
    let response =
        login::login(&ctx, &body.email, &body.code, server, body.save_credentials).await?;
    Ok(Json(response))
}

#[derive(Deserialize)]
struct BilibiliLoginRequest {
    username: String,
    password: String,
    #[serde(default = "default_true")]
    save_credentials: bool,
}

async fn login_bilibili(
    State(ctx): State<AppContext>,
    extract::Json(body): extract::Json<BilibiliLoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let response =
        login::login_bilibili(&ctx, &body.username, &body.password, body.save_credentials).await?;
    Ok(Json(response))
}

#[derive(Deserialize)]
struct PhoneRequest {
    phone: String,
}

async fn send_bilibili_sms(
    State(ctx): State<AppContext>,
    extract::Json(body): extract::Json<PhoneRequest>,
) -> ApiResult<Json<StatusOk>> {
    login::send_bilibili_sms(&ctx, &body.phone).await?;
    Ok(ok())
}

#[derive(Deserialize)]
struct BilibiliSmsLoginRequest {
    phone: String,
    code: String,
    #[serde(default = "default_true")]
    save_credentials: bool,
}

async fn login_bilibili_sms(
    State(ctx): State<AppContext>,
    extract::Json(body): extract::Json<BilibiliSmsLoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let response =
        login::login_bilibili_sms(&ctx, &body.phone, &body.code, body.save_credentials).await?;
    Ok(Json(response))
}

async fn send_code_cn(
    State(ctx): State<AppContext>,
    extract::Json(body): extract::Json<PhoneRequest>,
) -> ApiResult<Json<StatusOk>> {
    login::send_code_cn(&ctx, &body.phone).await?;
    Ok(ok())
}

#[derive(Deserialize)]
struct CnLoginRequest {
    phone: String,
    #[serde(default)]
    password: Option<String>,
    #[serde(default)]
    code: Option<String>,
    #[serde(default = "default_true")]
    save_credentials: bool,
}

async fn login_cn(
    State(ctx): State<AppContext>,
    extract::Json(body): extract::Json<CnLoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let response = login::login_cn(
        &ctx,
        &body.phone,
        body.password.as_deref(),
        body.code.as_deref(),
        body.save_credentials,
    )
    .await?;
    Ok(Json(response))
}

async fn verify(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
) -> ApiResult<Json<VerifySession>> {
    let claims = if is_service_call(&headers, &ctx) {
        Claims {
            sub: "service".to_owned(),
            uid: "service".to_owned(),
            server: "internal".to_owned(),
            role: jwt::role_name(myrtle_types::Role::SuperAdmin).to_owned(),
            exp: 0,
            iat: 0,
        }
    } else {
        let token = bearer(&headers).ok_or(AppError::Unauthorized)?;
        jwt::verify_token(&ctx.secrets.jwt, token)?
    };
    Ok(Json(account::verify(&ctx, &claims).await?))
}

async fn update_settings(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<UpdateSettingsRequest>,
) -> ApiResult<Json<StatusOk>> {
    privacy::update_settings(&ctx, &caller, &body).await?;
    Ok(ok())
}

async fn disconnect(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<DisconnectResult>> {
    Ok(Json(game_session::disconnect(&ctx, &caller).await?))
}

async fn delete_account(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<StatusOk>> {
    account::delete_account(&ctx, &caller).await?;
    Ok(ok())
}

async fn refresh(State(ctx): State<AppContext>, caller: Caller) -> ApiResult<Json<Value>> {
    Ok(Json(sync::refresh(&ctx, &caller).await?))
}
