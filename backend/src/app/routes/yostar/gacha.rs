use crate::{
    app::{error::ApiError, state::AppState},
    core::user::gacha::{self, GachaRecords, GachaType, GachaTypeRecords},
};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GachaQuery {
    pub yssid: Option<String>,
    pub yssid_sig: Option<String>,
    #[serde(rename = "type")]
    pub gacha_type: Option<GachaType>,
}

// GET /gacha?yssid={yssid}&yssid_sig={yssid_sig}
// Returns all gacha types (limited, regular, special) in parallel
pub async fn get_all_gacha(
    State(state): State<AppState>,
    Query(params): Query<GachaQuery>,
) -> Result<Json<GachaRecords>, ApiError> {
    let yssid = params
        .yssid
        .ok_or(ApiError::BadRequest("Missing 'yssid' parameter.".into()))?;
    let yssid_sig = params.yssid_sig.ok_or(ApiError::BadRequest(
        "Missing 'yssid_sig' parameter.".into(),
    ))?;

    let records = gacha::get_gacha(&state.client, &yssid, &yssid_sig)
        .await
        .map_err(|e| {
            eprintln!("Gacha fetch error: {e:?}");
            ApiError::Internal("Failed to fetch gacha records.".into())
        })?;

    Ok(Json(records))
}

// GET /gacha/{type}?yssid={yssid}&yssid_sig={yssid_sig}
// Returns gacha records for a specific type only
pub async fn get_gacha_by_type(
    State(state): State<AppState>,
    Query(params): Query<GachaQuery>,
) -> Result<Json<GachaTypeRecords>, ApiError> {
    let yssid = params
        .yssid
        .ok_or(ApiError::BadRequest("Missing 'yssid' parameter.".into()))?;
    let yssid_sig = params.yssid_sig.ok_or(ApiError::BadRequest(
        "Missing 'yssid_sig' parameter.".into(),
    ))?;
    let gacha_type = params
        .gacha_type
        .ok_or(ApiError::BadRequest("Missing 'type' parameter.".into()))?;

    let records = gacha::get_gacha_by_type(&state.client, &yssid, &yssid_sig, gacha_type)
        .await
        .map_err(|e| {
            eprintln!("Gacha fetch error: {e:?}");
            ApiError::Internal("Failed to fetch gacha records.".into())
        })?;

    Ok(Json(records))
}
