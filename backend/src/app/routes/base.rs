use axum::Json;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use serde::Deserialize;

use crate::app::cache::cached_json_detached;
use crate::app::cache::keys::CacheKey;
use crate::app::cpu;
use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::routes::resolve_uid;
use crate::app::routes::static_data::json_response;
use crate::app::services::base_planner::{
    AccountFactsReq, CatalogResponse, EvaluateRequest, EvaluateResponse, LayoutResponse,
    OptimizeRequest, RotationRequest, catalog, evaluate, layout, optimize, rotation, save_facts,
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

// Which roster to plan against comes from the shared `resolve_uid` gate in
// `routes::mod`, which must run here in the handler rather than inside the
// service: `rotation_plan` can answer from cache without entering the service
// at all, so a gate further in would not see that request.

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
    let _admission = cpu::admit("base_evaluate").await?;
    Ok(Json(evaluate(&state, &uid, viewer, body).await?))
}

pub async fn optimize_layout(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    headers: HeaderMap,
    Query(params): Query<PlannerParams>,
    Json(body): Json<OptimizeRequest>,
) -> Result<Response, ApiError> {
    let uid = resolve_uid(&state, &auth, params.uid.as_deref()).await?;
    let viewer = viewer_id(&state, &auth).await;
    let key = CacheKey::BaseOptimize {
        uid: &uid,
        request_hash: request_hash(&body),
    };
    let owner = state.clone();
    let owner_uid = uid.clone();
    let cached = cached_json_detached(&state, &key, move || async move {
        let _admission = cpu::admit("base_optimize").await?;
        let resp = optimize(&owner, &owner_uid, viewer, body).await?;
        serde_json::to_string(&resp)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("serialize optimize: {e}")))
    })
    .await?;
    Ok(json_response(cached, &headers))
}

/// A two-squad, three-shift rotation for the drafted layout. The heaviest
/// planner endpoint, so identical (uid, request) pairs serve from cache: the
/// computation is deterministic given its inputs, and the short TTL only
/// bounds staleness against a fresh account sync.
///
/// Both searches run DETACHED and single-flight (`cached_json_detached`): a
/// request the handler timeout drops still fills the cache, and a retry
/// arriving while the search runs joins it instead of starting another. The
/// admission permit is taken inside the build task, so a joiner never holds
/// one and a cache hit never runs a search.
pub async fn rotation_plan(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    headers: HeaderMap,
    Query(params): Query<PlannerParams>,
    Json(body): Json<RotationRequest>,
) -> Result<Response, ApiError> {
    let uid = resolve_uid(&state, &auth, params.uid.as_deref()).await?;
    let viewer = viewer_id(&state, &auth).await;
    let key = CacheKey::BaseRotation {
        uid: &uid,
        request_hash: request_hash(&body),
    };
    let owner = state.clone();
    let owner_uid = uid.clone();
    let cached = cached_json_detached(&state, &key, move || async move {
        let _admission = cpu::admit("base_rotation").await?;
        let resp = rotation(&owner, &owner_uid, viewer, body).await?;
        serde_json::to_string(&resp)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("serialize rotation: {e}")))
    })
    .await?;
    Ok(json_response(cached, &headers))
}

/// The cache identity of a planner request: its serialized body.
fn request_hash<T: serde::Serialize>(body: &T) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    serde_json::to_string(body).unwrap_or_default().hash(&mut h);
    h.finish()
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
