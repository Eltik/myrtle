use axum::{Json, extract::State};
use serde::Deserialize;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::routes::{StatusOk, ok_status};
use crate::app::services;
use crate::app::services::auth::parse_server;
use crate::app::state::AppState;
use crate::core::auth::permissions::GlobalRole;
use crate::database::queries;

/// The `/auth/verify` response: the session a token currently represents.
///
/// Serialized directly by the handler, so this declaration is the wire format,
/// the `OpenAPI` schema and the generated TypeScript at once.
#[derive(serde::Serialize, TS, ToSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct VerifySession {
    /// Always true - a failing token is a 401, not `valid: false`.
    pub valid: bool,
    pub user_id: String,
    /// The player's in-game uid.
    pub uid: String,
    /// The game server the session is bound to.
    pub server: String,
    /// Re-read from the database on every call, so a promotion takes effect
    /// without the user logging out and back in.
    pub role: String,
    /// True for a role that may open the admin panel, or for any holder of a
    /// translation grant.
    pub can_access_admin_panel: bool,
}

/// The `/auth/disconnect` response.
#[derive(serde::Serialize, TS, ToSchema)]
#[ts(export)]
pub struct DisconnectResult {
    #[schema(example = "ok")]
    pub status: String,
    /// Whether there were stored credentials to remove. Disconnecting twice is
    /// not an error; the second call reports `false`.
    pub removed: bool,
}

/// `save_credentials` is the player's answer to "remember this login".
/// Defaults to true so an older client, or any caller that omits it, keeps the
/// behaviour it had. False means store nothing and clear anything a previous
/// login left; see `services::auth::persist_credentials`.
const fn default_true() -> bool {
    true
}

#[derive(Deserialize, ToSchema)]
pub struct SendCodeRequest {
    pub email: String,
    pub server: String,
}

/// Ask Yostar to email a one-time login code.
///
/// The code is then spent by `POST /login`.
#[utoipa::path(
    post,
    path = "/login/send-code",
    tag = "auth",
    request_body = SendCodeRequest,
    responses(
        (status = 200, description = "The code was requested. Nothing is returned about whether the address exists.", body = StatusOk),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn send_code(
    State(state): State<AppState>,
    Json(body): Json<SendCodeRequest>,
) -> Result<Json<StatusOk>, ApiError> {
    let server = parse_server(&body.server)?;
    services::auth::send_code(&state, &body.email, server).await?;
    Ok(ok_status())
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub code: String,
    pub server: String,
    /// See the module note on `save_credentials`.
    #[serde(default = "default_true")]
    pub save_credentials: bool,
}

/// Exchange an emailed code for a site token.
///
/// The returned token authenticates every other endpoint and lasts seven days.
#[utoipa::path(
    post,
    path = "/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "A site token plus the identity it is bound to.", body = crate::app::services::auth::LoginResponse),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<services::auth::LoginResponse>, ApiError> {
    let server = parse_server(&body.server)?;
    let result = services::auth::login(
        &state,
        &body.email,
        &body.code,
        server,
        body.save_credentials,
    )
    .await?;
    Ok(Json(result))
}

#[derive(Deserialize, ToSchema)]
pub struct BilibiliLoginRequest {
    pub username: String,
    pub password: String,
    /// See the module note on `save_credentials`.
    #[serde(default = "default_true")]
    pub save_credentials: bool,
}

/// Log in to a Bilibili-server account with username and password.
#[utoipa::path(
    post,
    path = "/login/bilibili",
    tag = "auth",
    request_body = BilibiliLoginRequest,
    responses(
        (status = 200, description = "A site token plus the identity it is bound to.", body = crate::app::services::auth::LoginResponse),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn login_bilibili(
    State(state): State<AppState>,
    Json(body): Json<BilibiliLoginRequest>,
) -> Result<Json<services::auth::LoginResponse>, ApiError> {
    let result = services::auth::login_bilibili(
        &state,
        &body.username,
        &body.password,
        body.save_credentials,
    )
    .await?;
    Ok(Json(result))
}

#[derive(Deserialize, ToSchema)]
pub struct BilibiliSendSmsRequest {
    pub phone: String,
}

/// Ask Bilibili to send a login code by SMS.
///
/// Experimental: see the note on `login_bilibili_sms`.
#[utoipa::path(
    post,
    path = "/login/bilibili/send-code",
    tag = "auth",
    request_body = BilibiliSendSmsRequest,
    responses(
        (status = 200, description = "The SMS was requested.", body = StatusOk),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn send_bilibili_sms(
    State(state): State<AppState>,
    Json(body): Json<BilibiliSendSmsRequest>,
) -> Result<Json<StatusOk>, ApiError> {
    services::auth::send_bilibili_sms(&state, &body.phone).await?;
    Ok(ok_status())
}

#[derive(Deserialize, ToSchema)]
pub struct BilibiliSmsLoginRequest {
    pub phone: String,
    pub code: String,
    /// See the module note on `save_credentials`.
    #[serde(default = "default_true")]
    pub save_credentials: bool,
}

/// Experimental: see `core::hypergryph::bilibili` module docs. The SMS
/// endpoints themselves are unverified guesses, unlike username/password
/// login.
#[utoipa::path(
    post,
    path = "/login/bilibili/sms",
    tag = "auth",
    request_body = BilibiliSmsLoginRequest,
    responses(
        (status = 200, description = "A site token plus the identity it is bound to.", body = crate::app::services::auth::LoginResponse),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn login_bilibili_sms(
    State(state): State<AppState>,
    Json(body): Json<BilibiliSmsLoginRequest>,
) -> Result<Json<services::auth::LoginResponse>, ApiError> {
    let result =
        services::auth::login_bilibili_sms(&state, &body.phone, &body.code, body.save_credentials)
            .await?;
    Ok(Json(result))
}

#[derive(Deserialize, ToSchema)]
pub struct CnSendCodeRequest {
    pub phone: String,
}

/// Ask the CN passport service to send a login code by SMS.
///
/// Experimental: see `core::hypergryph::passport`.
#[utoipa::path(
    post,
    path = "/login/cn/send-code",
    tag = "auth",
    request_body = CnSendCodeRequest,
    responses(
        (status = 200, description = "The code was requested.", body = StatusOk),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn send_code_cn(
    State(state): State<AppState>,
    Json(body): Json<CnSendCodeRequest>,
) -> Result<Json<StatusOk>, ApiError> {
    services::auth::send_code_cn(&state, &body.phone).await?;
    Ok(ok_status())
}

#[derive(Deserialize, ToSchema)]
pub struct CnLoginRequest {
    pub phone: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    /// See the module note on `save_credentials`.
    #[serde(default = "default_true")]
    pub save_credentials: bool,
}

/// Experimental: see `core::hypergryph::passport` module docs. Expected to
/// fail until the real game-client appCode is known.
#[utoipa::path(
    post,
    path = "/login/cn",
    tag = "auth",
    request_body = CnLoginRequest,
    responses(
        (status = 200, description = "A site token plus the identity it is bound to.", body = crate::app::services::auth::LoginResponse),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn login_cn(
    State(state): State<AppState>,
    Json(body): Json<CnLoginRequest>,
) -> Result<Json<services::auth::LoginResponse>, ApiError> {
    let result = services::auth::login_cn(
        &state,
        &body.phone,
        body.password.as_deref(),
        body.code.as_deref(),
        body.save_credentials,
    )
    .await?;
    Ok(Json(result))
}

/// Check the caller's token and report the session it represents.
///
/// The role is re-read from the database rather than taken from the token's
/// frozen claim, so a promotion or demotion takes effect on the next call.
#[utoipa::path(
    get,
    path = "/auth/verify",
    tag = "auth",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The live session.", body = VerifySession),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn verify(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<VerifySession>, ApiError> {
    // AuthUser extractor already validated the token. Re-read the user's
    // role from the DB so admin promotions/demotions take effect on the next
    // `verify` call without requiring the user to log out and back in
    // (JWT claims freeze the role at login time).
    let db_role: Option<String> = if let Ok(uuid) = uuid::Uuid::parse_str(&auth.user_id) {
        sqlx::query_scalar::<_, String>("SELECT role FROM users WHERE id = $1")
            .bind(uuid)
            .fetch_optional(&state.db)
            .await?
    } else {
        None
    };

    let role = db_role.unwrap_or_else(|| auth.role.to_string());

    // The same gate the `/admin` route tree uses, answered here so the two
    // cannot disagree. It is computed from the DB role above - not the JWT
    // claim, which freezes at login - OR from holding any translation grant,
    // which is enough on its own: a locale grant would be unusable if its
    // holder could not open the screen that spends it.
    //
    // Before this, the session was built from `/get-user` (live DB role) while
    // every admin route read the JWT, so a freshly promoted user was let into
    // the panel and then 403'd by everything inside it.
    let can_access_admin_panel = role
        .parse::<GlobalRole>()
        .unwrap_or_default()
        .can_access_admin_panel()
        || match auth.user_uuid() {
            Ok(id) => queries::i18n::has_any_permission(&state.db, id).await?,
            Err(_) => false,
        };

    Ok(Json(VerifySession {
        valid: true,
        user_id: auth.user_id,
        uid: auth.uid,
        server: auth.server,
        role,
        can_access_admin_panel,
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateSettingsRequest {
    pub public_profile: bool,
    pub store_gacha: bool,
    pub share_stats: bool,
}

/// Change the caller's own privacy settings.
#[utoipa::path(
    post,
    path = "/auth/update-settings",
    operation_id = "auth_update_settings",
    tag = "auth",
    request_body = UpdateSettingsRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Settings saved and the profile cache invalidated.", body = StatusOk),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn update_settings(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpdateSettingsRequest>,
) -> Result<Json<StatusOk>, ApiError> {
    let user_id: uuid::Uuid = auth.user_uuid()?;
    services::auth::update_settings(
        &state,
        user_id,
        body.public_profile,
        body.store_gacha,
        body.share_stats,
    )
    .await?;
    // v_user_profile is read through a 10-minute cache; without this, fresh
    // reads after the write would still return the old settings.
    state
        .cache
        .invalidate(&CacheKey::User { uid: &auth.uid })
        .await;
    Ok(ok_status())
}

/// Forget the stored game credentials for the caller's own account.
///
/// The site session stays valid and the already-synced data stays put; what is
/// revoked is our ability to reach Yostar on their behalf. Re-syncing then
/// needs a fresh email code, which is the point. Before this, the only way to
/// withdraw that access was to email us and have the whole account deleted.
///
/// Idempotent: disconnecting twice is not an error. `removed` reports whether
/// there was anything stored to remove.
#[utoipa::path(
    post,
    path = "/auth/disconnect",
    tag = "auth",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Any stored credentials were removed.", body = DisconnectResult),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn disconnect(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<DisconnectResult>, ApiError> {
    let user_id = auth.user_uuid()?;
    let removed = services::game_session::disconnect(&state, &auth.uid, user_id).await?;
    Ok(Json(DisconnectResult {
        status: "ok".to_owned(),
        removed,
    }))
}

/// Re-sync the caller's roster and account data from the game servers.
///
/// Needs stored game credentials; a player who has disconnected must log in
/// again first.
#[utoipa::path(
    post,
    path = "/refresh",
    tag = "auth",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The account payload as returned by the game's own sync, passed through after parsing. Its fields are the game's rather than this API's, so they are not modelled here; the typed views of the same data are `/roster`, `/inventory` and `/stage-clears`.", content_type = "application/json"),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let server = parse_server(&auth.server)?;
    let data = services::roster::refresh(&state, &auth.uid, server).await?;
    Ok(Json(data))
}
