use crate::{app::state::AppState, core::authentication::jwt};
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub token: String,
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub user_id: Option<String>,
    pub uid: Option<String>,
    pub role: Option<String>,
}

pub async fn verify_token(
    State(state): State<AppState>,
    Json(body): Json<VerifyRequest>,
) -> Json<VerifyResponse> {
    match jwt::verify_token(&state.jwt_secret, &body.token) {
        Ok(claims) => Json(VerifyResponse {
            valid: true,
            user_id: Some(claims.sub),
            uid: Some(claims.uid),
            role: Some(claims.role),
        }),
        Err(_) => Json(VerifyResponse {
            valid: false,
            user_id: None,
            uid: None,
            role: None,
        }),
    }
}
