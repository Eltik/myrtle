use crate::{
    app::{error::ApiError, state::AppState},
    core::{
        authentication::jwt,
        user::gacha::{self, GachaRecords, GachaType, GachaTypeRecords},
    },
    database::models::{
        gacha::{
            GachaRecord, GlobalPullRates, UpdateGachaSettings, UserGachaSettings,
            get_global_pull_rates,
        },
        user::User,
    },
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
    /// JWT token for authenticated storage (optional - if not provided, records won't be stored)
    pub token: Option<String>,
}

/// GET /gacha?yssid={yssid}&yssid_sig={yssid_sig}&token={token}
/// Returns all gacha types (limited, regular, special) in parallel.
/// If a valid JWT token is provided and user hasn't opted out, records are automatically stored.
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

    // Fetch records from Yostar API
    let records = gacha::get_gacha(&state.client, &yssid, &yssid_sig)
        .await
        .map_err(|e| {
            eprintln!("Gacha fetch error: {e:?}");
            ApiError::Internal("Failed to fetch gacha records.".into())
        })?;

    // If token provided, attempt to store records (opt-out model)
    if let Some(token) = &params.token
        && let Ok(claims) = jwt::verify_token(&state.jwt_secret, token)
        && let Ok(Some(user)) = User::find_by_uid(&state.db, &claims.uid).await
    {
        // Get or create settings (defaults to store=true)
        if let Ok(settings) = UserGachaSettings::find_or_create(&state.db, user.id).await {
            // Only store if user hasn't opted out
            if settings.store_records {
                store_records_for_user(&state, user.id, &records, &settings).await;
            }
        }
    }

    Ok(Json(records))
}

/// GET /gacha/type?yssid={yssid}&yssid_sig={yssid_sig}&type={type}&token={token}
/// Returns gacha records for a specific type only.
/// If a valid JWT token is provided and user hasn't opted out, records are automatically stored.
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

    // Fetch records from Yostar API
    let records = gacha::get_gacha_by_type(&state.client, &yssid, &yssid_sig, gacha_type)
        .await
        .map_err(|e| {
            eprintln!("Gacha fetch error: {e:?}");
            ApiError::Internal("Failed to fetch gacha records.".into())
        })?;

    // If token provided, attempt to store records (opt-out model)
    if let Some(token) = &params.token
        && let Ok(claims) = jwt::verify_token(&state.jwt_secret, token)
        && let Ok(Some(user)) = User::find_by_uid(&state.db, &claims.uid).await
        && let Ok(settings) = UserGachaSettings::find_or_create(&state.db, user.id).await
        && settings.store_records
    {
        store_type_records_for_user(&state, user.id, &records, gacha_type, &settings).await;
    }

    Ok(Json(records))
}

/// Internal: Store all gacha records for a user
async fn store_records_for_user(
    state: &AppState,
    user_id: uuid::Uuid,
    records: &GachaRecords,
    settings: &UserGachaSettings,
) {
    // Store each type, filtering by last sync timestamp
    for (gacha_type, type_records, last_sync) in [
        (
            GachaType::Limited,
            &records.limited.records,
            settings.last_sync_limited_at,
        ),
        (
            GachaType::Regular,
            &records.regular.records,
            settings.last_sync_regular_at,
        ),
        (
            GachaType::Special,
            &records.special.records,
            settings.last_sync_special_at,
        ),
    ] {
        // Filter to only new records
        let new_records: Vec<_> = type_records
            .iter()
            .filter(|r| last_sync.is_none() || r.at > last_sync.unwrap())
            .cloned()
            .collect();

        if !new_records.is_empty()
            && GachaRecord::store_batch(&state.db, user_id, &new_records, gacha_type)
                .await
                .is_ok()
        {
            // Update sync timestamp
            if let Some(max_ts) = new_records.iter().map(|r| r.at).max() {
                let _ = UserGachaSettings::update_sync_timestamp(
                    &state.db, user_id, gacha_type, max_ts,
                )
                .await;
            }
        }
    }

    // Recalculate counts (ignore errors)
    let _ = UserGachaSettings::recalculate_counts(&state.db, user_id).await;
}

/// Internal: Store single type gacha records for a user
async fn store_type_records_for_user(
    state: &AppState,
    user_id: uuid::Uuid,
    records: &GachaTypeRecords,
    gacha_type: GachaType,
    settings: &UserGachaSettings,
) {
    let last_sync = match gacha_type {
        GachaType::Limited => settings.last_sync_limited_at,
        GachaType::Regular => settings.last_sync_regular_at,
        GachaType::Special => settings.last_sync_special_at,
    };

    // Filter to only new records
    let new_records: Vec<_> = records
        .records
        .iter()
        .filter(|r| last_sync.is_none() || r.at > last_sync.unwrap())
        .cloned()
        .collect();

    if !new_records.is_empty()
        && let Ok(_) = GachaRecord::store_batch(&state.db, user_id, &new_records, gacha_type).await
        && let Some(max_ts) = new_records.iter().map(|r| r.at).max()
    {
        let _ =
            UserGachaSettings::update_sync_timestamp(&state.db, user_id, gacha_type, max_ts).await;
    }

    let _ = UserGachaSettings::recalculate_counts(&state.db, user_id).await;
}

// ============================================================================
// Settings & Stats Endpoints
// ============================================================================

#[derive(Deserialize)]
pub struct SettingsQuery {
    pub token: String,
}

#[derive(Deserialize)]
pub struct UpdateSettingsRequest {
    pub token: String,
    pub store_records: Option<bool>,
    pub share_anonymous_stats: Option<bool>,
}

/// GET /gacha/settings?token={token}
/// Gets user's gacha storage settings
pub async fn get_gacha_settings(
    State(state): State<AppState>,
    Query(params): Query<SettingsQuery>,
) -> Result<Json<UserGachaSettings>, ApiError> {
    let claims = jwt::verify_token(&state.jwt_secret, &params.token)
        .map_err(|_| ApiError::BadRequest("Invalid token.".into()))?;

    let user = User::find_by_uid(&state.db, &claims.uid)
        .await?
        .ok_or(ApiError::NotFound("User not found.".into()))?;

    let settings = UserGachaSettings::find_or_create(&state.db, user.id).await?;
    Ok(Json(settings))
}

/// POST /gacha/settings
/// Updates user's gacha storage settings (opt-out)
pub async fn update_gacha_settings(
    State(state): State<AppState>,
    Json(body): Json<UpdateSettingsRequest>,
) -> Result<Json<UserGachaSettings>, ApiError> {
    let claims = jwt::verify_token(&state.jwt_secret, &body.token)
        .map_err(|_| ApiError::BadRequest("Invalid token.".into()))?;

    let user = User::find_by_uid(&state.db, &claims.uid)
        .await?
        .ok_or(ApiError::NotFound("User not found.".into()))?;

    // Ensure settings exist
    UserGachaSettings::find_or_create(&state.db, user.id).await?;

    let updated = UserGachaSettings::update_settings(
        &state.db,
        user.id,
        UpdateGachaSettings {
            store_records: body.store_records,
            share_anonymous_stats: body.share_anonymous_stats,
        },
    )
    .await?;

    Ok(Json(updated))
}

/// GET /gacha/stats
/// Gets global anonymous pull rate statistics
pub async fn get_global_stats(
    State(state): State<AppState>,
) -> Result<Json<GlobalPullRates>, ApiError> {
    let rates = get_global_pull_rates(&state.db).await?;
    Ok(Json(rates))
}
