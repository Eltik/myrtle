use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::app::services::auth::parse_server;
use crate::app::{error::ApiError, extractors::auth::AuthUser, services, state::AppState};

#[derive(Deserialize)]
pub struct FriendsParams {
    pub limit: Option<usize>,
}

/// The caller's in-game friends list.
#[utoipa::path(
    get,
    path = "/friends",
    tag = "player",
    params(
        ("limit" = Option<u32>, Query, description = "Maximum friends to return.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "`{ \"friends\": [...] }`. The array elements are the game's own player records, passed through unchanged, so their fields are the game's and are not modelled here.", content_type = "application/json"),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_friends(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<FriendsParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let server = parse_server(&auth.server)?;
    let data = services::social::get_friends(&state, &auth.uid, server, params.limit).await?;
    Ok(Json(data))
}

#[derive(Deserialize)]
pub struct SearchPlayersParams {
    pub q: String,
    pub limit: Option<usize>,
}

/// Look up other players by name or uid.
#[utoipa::path(
    get,
    path = "/players/search",
    tag = "player",
    params(
        ("q" = String, Query, description = "Query text."),
        ("limit" = Option<u32>, Query, description = "Maximum results to return.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "`{ \"friends\": [...] }`, the same envelope as `/friends`. The array elements are the game's own player records, passed through unchanged.", content_type = "application/json"),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn search_players(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<SearchPlayersParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let server = parse_server(&auth.server)?;
    let data = services::social::search_players(&state, &auth.uid, &params.q, server, params.limit)
        .await?;
    Ok(Json(data))
}
