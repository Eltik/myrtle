use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::routes::resolve_uid;
use crate::app::routes::static_data::json_response;
use crate::app::services::improvements::get_improvements;
use crate::app::state::AppState;

#[derive(Deserialize)]
pub struct ImprovementsParams {
    pub uid: Option<String>,
}

pub async fn get_user_improvements(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    headers: HeaderMap,
    Query(params): Query<ImprovementsParams>,
) -> Result<Response, ApiError> {
    // The access gate stays HERE, ahead of the cache read inside
    // `get_improvements`, so a cached body can never be served to a viewer who
    // would not have been allowed to build it.
    let uid = resolve_uid(&state, &auth, params.uid.as_deref()).await?;
    // Admission control lives inside the service now, around the one synchronous
    // pass that actually burns CPU. Taking it here held a permit across five
    // database round-trips too, which is not what it is for and is what made this
    // route shed under trivial concurrency.
    let cached = get_improvements(&state, &uid).await?;
    Ok(json_response(cached, &headers))
}
