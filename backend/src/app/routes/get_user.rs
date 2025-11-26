use axum::{
    Json,
    extract::{Path, State},
};
use reqwest::StatusCode;
use sqlx::PgPool;

use crate::database::models::user::User;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(uid): Path<String>,
) -> Result<Json<User>, StatusCode> {
    User::find_by_uid(&state.db, &uid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}
