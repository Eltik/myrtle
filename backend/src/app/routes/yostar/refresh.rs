use crate::{
    app::{error::ApiError, state::AppState},
    core::{
        authentication::constants::{AuthSession, Server},
        user::{self},
    },
    database::models::user::{CreateUser, User},
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RefreshQuery {
    pub uid: Option<String>,
    pub secret: Option<String>,
    pub seqnum: Option<u32>,
    pub server: Option<Server>,
}

// /refresh?uid={uid}&secret={secret}&seqnum={seqnum}&server={server}
pub async fn refresh_by_query(
    State(state): State<AppState>,
    Query(params): Query<RefreshQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let uid = params
        .uid
        .ok_or(ApiError::BadRequest("Missing 'uid' parameter.".into()))?;
    let secret = params
        .secret
        .ok_or(ApiError::BadRequest("Missing 'secret' parameter.".into()))?;
    let seqnum = params
        .seqnum
        .ok_or(ApiError::BadRequest("Missing 'seqnum' parameter.".into()))?;
    let server = params.server.unwrap_or(Server::EN);
    refresh_impl(&state, &uid, &secret, &seqnum, server).await
}

// /refresh/{uid}/{secret}/{seqnum} - server defaults to EN
pub async fn refresh_no_server(
    State(state): State<AppState>,
    Path((uid, secret, seqnum)): Path<(String, String, u32)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    refresh_impl(&state, &uid, &secret, &seqnum, Server::EN).await
}

// /refresh/{uid}/{secret}/{seqnum}/{server}
pub async fn refresh_by_server(
    State(state): State<AppState>,
    Path((uid, secret, seqnum, server)): Path<(String, String, u32, Server)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    refresh_impl(&state, &uid, &secret, &seqnum, server).await
}

async fn refresh_impl(
    state: &AppState,
    uid: &str,
    secret: &str,
    seqnum: &u32,
    server: Server,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut session = AuthSession::new(Some(uid), Some(secret), Some(*seqnum), None);

    let data = user::get::get(&state.client, &state.config, &mut session, server)
        .await
        .map_err(|e| {
            eprintln!("Fetch error: {:?}", e);
            ApiError::Internal("Internal server error.".into())
        })?;

    let user_data = data.ok_or(ApiError::NotFound("User not found.".into()))?;

    let user_json = serde_json::to_value(&user_data)
        .map_err(|_| ApiError::Internal("Failed to serialize user.".into()))?;

    let existing = User::find_by_uid(&state.db, uid)
        .await
        .map_err(|_| ApiError::Internal("Database error.".into()))?;

    if let Some(existing_user) = existing {
        User::update_data(&state.db, existing_user.id, user_json.clone())
            .await
            .map_err(|_| ApiError::Internal("Failed to update user.".into()))?;
    } else {
        User::create(
            &state.db,
            CreateUser {
                uid: uid.to_string(),
                server: server.as_str().to_string(),
                data: user_json.clone(),
            },
        )
        .await
        .map_err(|_| ApiError::Internal("Failed to create user.".into()))?;
    }

    Ok(Json(user_json))
}
