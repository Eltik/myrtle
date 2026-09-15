use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    app::{
        error::ApiError, extractors::auth::AuthUser, routes::ok_status, services::release as svc,
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

pub async fn events(State(state): State<AppState>) -> Result<Json<EventsResponse>, ApiError> {
    Ok(Json(svc::get_events(&state).await?))
}

pub async fn banners(State(state): State<AppState>) -> Result<Json<BannersResponse>, ApiError> {
    Ok(Json(svc::get_banners(&state).await?))
}

pub async fn skins(State(state): State<AppState>) -> Result<Json<SkinsResponse>, ApiError> {
    Ok(Json(svc::get_skins(&state).await?))
}

pub async fn lag(State(state): State<AppState>) -> Result<Json<LagResponse>, ApiError> {
    Ok(Json(svc::get_lag(&state).await?))
}

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

pub async fn put_override(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<PutOverride>,
) -> Result<Json<ReleaseOverride>, ApiError> {
    require_super_admin(&auth)?;
    let by = auth.user_uuid().ok();
    Ok(Json(svc::put_override(&state, body, by).await?))
}

pub async fn delete_override(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((kind, cn_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_super_admin(&auth)?;
    svc::delete_override(&state, &kind, &cn_id).await?;
    Ok(ok_status())
}

pub async fn get_plan(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Option<ReleasePlan>>, ApiError> {
    Ok(Json(svc::get_plan(&state, auth.user_uuid()?).await?))
}

pub async fn put_plan(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<PutReleasePlan>,
) -> Result<Json<ReleasePlan>, ApiError> {
    Ok(Json(svc::put_plan(&state, auth.user_uuid()?, body).await?))
}
