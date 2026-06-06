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
