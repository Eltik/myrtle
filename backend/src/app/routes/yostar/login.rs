use crate::{
    app::{error::ApiError, state::AppState},
    core::authentication::{
        constants::Server,
        login::{self},
    },
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginQuery {
    pub email: Option<String>,
    pub code: Option<String>,
    pub server: Option<Server>,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub uid: String,
    pub secret: String,
    pub seqnum: u32,
}

// /login?email={email}&code={code}&server={server}
pub async fn login_by_query(
    State(state): State<AppState>,
    Query(params): Query<LoginQuery>,
) -> Result<Json<LoginResponse>, ApiError> {
    let email = params
        .email
        .ok_or(ApiError::BadRequest("Missing 'email' parameter.".into()))?;
    let code = params
        .code
        .ok_or(ApiError::BadRequest("Missing 'code' parameter.".into()))?;
    let server = params.server.unwrap_or(Server::EN);
    login_impl(&state, &email, &code, server).await
}

// /login/{email}/{code} - server defaults to EN
pub async fn login_no_server(
    State(state): State<AppState>,
    Path((email, code)): Path<(String, String)>,
) -> Result<Json<LoginResponse>, ApiError> {
    login_impl(&state, &email, &code, Server::EN).await
}

// /login/{email}/{code}/{server}
pub async fn login_by_server(
    State(state): State<AppState>,
    Path((email, code, server)): Path<(String, String, Server)>,
) -> Result<Json<LoginResponse>, ApiError> {
    login_impl(&state, &email, &code, server).await
}

async fn login_impl(
    state: &AppState,
    email: &str,
    code: &str,
    server: Server,
) -> Result<Json<LoginResponse>, ApiError> {
    let session = login::login(
        &state.client,
        &state.config,
        &state.events,
        email,
        code,
        server,
        None,
    )
    .await
    .map_err(|e| {
        eprintln!("Login error: {e:?}");
        ApiError::Internal("Internal server error.".into())
    })?;

    Ok(Json(LoginResponse {
        uid: session.uid,
        secret: session.secret,
        seqnum: session.seqnum,
    }))
}
