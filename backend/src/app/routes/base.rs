use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::services::base_planner::{
    AccountFactsReq, CatalogResponse, EvaluateRequest, EvaluateResponse, LayoutResponse,
    OptimizeRequest, OptimizeResponse, RotationRequest, RotationResponse, catalog, evaluate,
    layout, optimize, rotation, save_facts,
};
use crate::app::state::AppState;
use crate::database::queries::users::find_by_id;

#[derive(Deserialize)]
pub struct PlannerParams {
    /// Whose roster to plan with. Omitted = the signed-in user's own.
    pub uid: Option<String>,
}

/// The viewer's own profile id, when signed in. Used to decide whether a
/// private profile may be planned against.
async fn viewer_id(state: &AppState, auth: &MaybeAuthUser) -> Option<uuid::Uuid> {
    let auth = auth.0.as_ref()?;
    let user_uuid = auth.user_uuid().ok()?;
    find_by_id(&state.db, user_uuid)
        .await
        .ok()
        .flatten()
        .map(|u| u.id)
}

/// Resolve which roster to plan against: the requested uid, or the caller's own
/// when none is given (which then requires being signed in).
async fn resolve_uid(
    state: &AppState,
    auth: &MaybeAuthUser,
    uid_param: Option<&str>,
) -> Result<String, ApiError> {
    if let Some(uid) = uid_param {
        return Ok(uid.to_string());
    }
    let auth = auth.0.as_ref().ok_or(ApiError::Unauthorized)?;
    let user_uuid: uuid::Uuid = auth.user_uuid()?;
    let profile = find_by_id(&state.db, user_uuid)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    Ok(profile.uid)
}

/// The player's real stationed base, as the planner's starting draft.
pub async fn get_layout(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<PlannerParams>,
) -> Result<Json<LayoutResponse>, ApiError> {
    let uid = resolve_uid(&state, &auth, params.uid.as_deref()).await?;
    let viewer = viewer_id(&state, &auth).await;
    Ok(Json(layout(&state, &uid, viewer).await?))
}

pub async fn evaluate_layout(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<PlannerParams>,
    Json(body): Json<EvaluateRequest>,
) -> Result<Json<EvaluateResponse>, ApiError> {
    let uid = resolve_uid(&state, &auth, params.uid.as_deref()).await?;
    let viewer = viewer_id(&state, &auth).await;
    Ok(Json(evaluate(&state, &uid, viewer, body).await?))
}

pub async fn optimize_layout(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<PlannerParams>,
    Json(body): Json<OptimizeRequest>,
) -> Result<Json<OptimizeResponse>, ApiError> {
    let uid = resolve_uid(&state, &auth, params.uid.as_deref()).await?;
    let viewer = viewer_id(&state, &auth).await;
    Ok(Json(optimize(&state, &uid, viewer, body).await?))
}

/// A two-squad, three-shift rotation for the drafted layout. The heaviest
/// planner endpoint, so identical (uid, request) pairs serve from cache: the
/// computation is deterministic given its inputs, and the short TTL only
/// bounds staleness against a fresh account sync.
pub async fn rotation_plan(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<PlannerParams>,
    Json(body): Json<RotationRequest>,
) -> Result<axum::response::Response, ApiError> {
    use axum::response::IntoResponse;
    let uid = resolve_uid(&state, &auth, params.uid.as_deref()).await?;
    let viewer = viewer_id(&state, &auth).await;
    let request_hash = {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        serde_json::to_string(&body).unwrap_or_default().hash(&mut h);
        h.finish()
    };
    let key = crate::app::cache::keys::CacheKey::BaseRotation {
        uid: &uid,
        request_hash,
    };
    if let Some(raw) = state.cache.get_raw(&key).await {
        return Ok((
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            raw,
        )
            .into_response());
    }
    let resp = rotation(&state, &uid, viewer, body).await?;
    let json = serde_json::to_string(&resp)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("serialize rotation: {e}")))?;
    state.cache.set_raw(&key, json.clone()).await;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        json,
    )
        .into_response())
}

/// Facility definitions from `building_data`. Roster-independent and stable, so
/// it needs no auth and the client can cache it hard.
pub async fn get_catalog(State(state): State<AppState>) -> Json<CatalogResponse> {
    Json(catalog(&state))
}

/// The signed-in user saving their OWN account facts (recruit slots etc.).
/// Facts always attach to the caller's profile - there is no setting another
/// player's facts; viewers get per-request overrides instead.
pub async fn put_facts(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Json(body): Json<AccountFactsReq>,
) -> Result<Json<AccountFactsReq>, ApiError> {
    let auth = auth.0.as_ref().ok_or(ApiError::Unauthorized)?;
    let user_uuid: uuid::Uuid = auth.user_uuid()?;
    Ok(Json(save_facts(&state, user_uuid, body).await?))
}
