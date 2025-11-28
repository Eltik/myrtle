use crate::{
    app::{error::ApiError, state::AppState},
    core::authentication::{constants::Server, yostar},
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SendCodeQuery {
    pub email: Option<String>,
    pub server: Option<Server>,
}

// /send-code?email={email}&server={server}
pub async fn send_code_by_query(
    State(state): State<AppState>,
    Query(params): Query<SendCodeQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let email = params
        .email
        .ok_or(ApiError::BadRequest("Missing 'email' parameter.".into()))?;
    let server = params.server.unwrap_or(Server::EN);
    send_code_impl(&state, &email, server).await
}

// /send-code/{email} - server defaults to EN
pub async fn send_code_by_email(
    State(state): State<AppState>,
    Path(email): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    send_code_impl(&state, &email, Server::EN).await
}

// /send-code/{email}/{server}
pub async fn send_code_by_email_and_server(
    State(state): State<AppState>,
    Path((email, server)): Path<(String, Server)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    send_code_impl(&state, &email, server).await
}

async fn send_code_impl(
    state: &AppState,
    email: &str,
    server: Server,
) -> Result<Json<serde_json::Value>, ApiError> {
    let data = yostar::send_code(&state.client, email, server)
        .await
        .map_err(|e| {
            eprintln!("Fetch error: {:?}", e);
            ApiError::Internal("Internal server error.".into())
        })?;
    Ok(Json(data))
}
