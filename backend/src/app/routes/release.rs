use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    app::{
        error::ApiError,
        extractors::auth::AuthUser,
        routes::{StatusOk, ok_status},
        services::release as svc,
        state::AppState,
    },
    core::{
        auth::permissions::GlobalRole,
        release::{
            BannersResponse, EventsResponse, LagResponse, PutOverride, PutReleasePlan,
            ReleaseOverride, ReleasePlan, SkinsResponse,
        },
    },
};

/// Events that have run on CN, with their projected arrival on each server.
#[utoipa::path(
    get,
    path = "/release/events",
    tag = "release",
    responses(
        (status = 200, description = "Events and projected dates.", body = EventsResponse),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn events(State(state): State<AppState>) -> Result<Json<EventsResponse>, ApiError> {
    Ok(Json(svc::get_events(&state).await?))
}

/// Gacha banners that have run on CN, with their projected arrival.
#[utoipa::path(
    get,
    path = "/release/banners",
    tag = "release",
    responses(
        (status = 200, description = "Banners and projected dates.", body = BannersResponse),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn banners(State(state): State<AppState>) -> Result<Json<BannersResponse>, ApiError> {
    Ok(Json(svc::get_banners(&state).await?))
}

/// Skins released on CN, with their projected arrival.
#[utoipa::path(
    get,
    path = "/release/skins",
    tag = "release",
    responses(
        (status = 200, description = "Skins and projected dates.", body = SkinsResponse),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn skins(State(state): State<AppState>) -> Result<Json<SkinsResponse>, ApiError> {
    Ok(Json(svc::get_skins(&state).await?))
}

/// How far each server currently trails CN, which is what the projections
/// above are built from.
#[utoipa::path(
    get,
    path = "/release/lag",
    tag = "release",
    responses(
        (status = 200, description = "Measured lag per server.", body = LagResponse),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn lag(State(state): State<AppState>) -> Result<Json<LagResponse>, ApiError> {
    Ok(Json(svc::get_lag(&state).await?))
}

/// Manual corrections applied on top of the projected dates.
#[utoipa::path(
    get,
    path = "/release/overrides",
    operation_id = "release_list_overrides",
    tag = "release",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Every stored override.", body = Vec<ReleaseOverride>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list_overrides(
    State(state): State<AppState>,
) -> Result<Json<Vec<ReleaseOverride>>, ApiError> {
    Ok(Json(svc::list_overrides(&state).await?))
}

fn require_super_admin(auth: &AuthUser) -> Result<(), ApiError> {
    if auth.role == GlobalRole::SuperAdmin {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

/// Create or replace one manual date override.
#[utoipa::path(
    put,
    path = "/release/overrides",
    operation_id = "release_put_override",
    tag = "release",
    request_body = PutOverride,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The override as stored.", body = ReleaseOverride),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn put_override(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<PutOverride>,
) -> Result<Json<ReleaseOverride>, ApiError> {
    require_super_admin(&auth)?;
    let by = auth.user_uuid().ok();
    Ok(Json(svc::put_override(&state, body, by).await?))
}

/// Drop one manual override, returning that entry to its projected date.
#[utoipa::path(
    delete,
    path = "/release/overrides/{kind}/{cn_id}",
    operation_id = "release_delete_override",
    tag = "release",
    params(
        ("kind" = String, Path, description = "Override kind: `event`, `banner` or `skin`."),
        ("cn_id" = String, Path, description = "The CN-side id the override applies to.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The override is gone.", body = StatusOk),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn delete_override(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((kind, cn_id)): Path<(String, String)>,
) -> Result<Json<StatusOk>, ApiError> {
    require_super_admin(&auth)?;
    svc::delete_override(&state, &kind, &cn_id).await?;
    Ok(ok_status())
}

/// The editorial release plan, or null when none is stored.
#[utoipa::path(
    get,
    path = "/release/plan",
    tag = "release",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The plan, or null.", body = Option<ReleasePlan>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_plan(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Option<ReleasePlan>>, ApiError> {
    Ok(Json(svc::get_plan(&state, auth.user_uuid()?).await?))
}

/// Replace the editorial release plan.
#[utoipa::path(
    put,
    path = "/release/plan",
    tag = "release",
    request_body = PutReleasePlan,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The plan as stored.", body = ReleasePlan),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn put_plan(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<PutReleasePlan>,
) -> Result<Json<ReleasePlan>, ApiError> {
    Ok(Json(svc::put_plan(&state, auth.user_uuid()?, body).await?))
}
