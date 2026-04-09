use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};

use crate::app::{error::ApiError, state::AppState};
use crate::core::gamedata::assets::AssetKind;

pub async fn avatar(
    State(state): State<AppState>,
    Path(avatar_id): Path<String>,
) -> Result<Response, ApiError> {
    let idx = state.asset_index.load();
    let rel_path = idx
        .path(AssetKind::Avatar, &avatar_id)
        .ok_or(ApiError::NotFound)?;
    serve_file(&state.config.assets_dir, rel_path).await
}

pub async fn portrait(
    State(state): State<AppState>,
    Path(char_id): Path<String>,
) -> Result<Response, ApiError> {
    let idx = state.asset_index.load();
    let rel_path = idx.portrait_path(&char_id).ok_or(ApiError::NotFound)?;

    serve_file(&state.config.assets_dir, rel_path).await
}

async fn serve_file(assets_dir: &str, rel_path: &str) -> Result<Response, ApiError> {
    let full_path = format!("{assets_dir}{rel_path}");

    let bytes = tokio::fs::read(&full_path)
        .await
        .map_err(|_| ApiError::NotFound)?;

    Ok(([(header::CONTENT_TYPE, "image/png")], bytes).into_response())
}
