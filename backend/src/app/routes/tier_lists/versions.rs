use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::authentication::permissions::Permission;
use crate::database::models::tier_lists::{
    PublishVersion, TierChangeLog, TierListSnapshot, TierListVersion,
};

use super::middleware::{check_permission, get_tier_list_by_slug, require_auth};

#[derive(Serialize)]
pub struct VersionSummary {
    pub id: String,
    pub version: i32,
    pub change_summary: Option<String>,
    pub published_at: String,
    pub published_by: Option<String>,
}

#[derive(Serialize)]
pub struct ListVersionsResponse {
    pub versions: Vec<VersionSummary>,
}

#[derive(Serialize)]
pub struct VersionDetail {
    pub id: String,
    pub version: i32,
    pub snapshot: serde_json::Value,
    pub changelog: String,
    pub change_summary: Option<String>,
    pub published_at: String,
    pub published_by: Option<String>,
}

#[derive(Serialize)]
pub struct ChangeLogEntry {
    pub id: String,
    pub change_type: String,
    pub operator_id: Option<String>,
    pub operator_name: Option<String>,
    pub old_tier_name: Option<String>,
    pub new_tier_name: Option<String>,
    pub reason: Option<String>,
    pub changed_at: String,
}

#[derive(Serialize)]
pub struct ChangelogResponse {
    pub changes: Vec<ChangeLogEntry>,
}

#[derive(Deserialize)]
pub struct ListVersionsQuery {
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct ChangelogQuery {
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct PublishVersionRequest {
    pub changelog: String,
    pub change_summary: Option<String>,
}

#[derive(Serialize)]
pub struct PublishVersionResponse {
    pub success: bool,
    pub version: VersionSummary,
}

/// GET /tier-lists/{slug}/versions
/// List all versions of a tier list
pub async fn list_versions(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<ListVersionsQuery>,
) -> Result<Json<ListVersionsResponse>, ApiError> {
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    let limit = query.limit.unwrap_or(50);

    let versions = TierListVersion::find_by_tier_list(&state.db, tier_list.id, limit)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Failed to fetch versions".into())
        })?;

    let summaries: Vec<VersionSummary> = versions
        .into_iter()
        .map(|v| VersionSummary {
            id: v.id.to_string(),
            version: v.version,
            change_summary: v.change_summary,
            published_at: v.published_at.to_rfc3339(),
            published_by: v.published_by.map(|id| id.to_string()),
        })
        .collect();

    Ok(Json(ListVersionsResponse {
        versions: summaries,
    }))
}

/// GET /tier-lists/{slug}/versions/{version}
/// Get a specific version with full snapshot
pub async fn get_version(
    State(state): State<AppState>,
    Path((slug, version)): Path<(String, i32)>,
) -> Result<Json<VersionDetail>, ApiError> {
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    let version_data = TierListVersion::find_by_version(&state.db, tier_list.id, version)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Failed to fetch version".into())
        })?
        .ok_or_else(|| ApiError::NotFound("Version not found".into()))?;

    Ok(Json(VersionDetail {
        id: version_data.id.to_string(),
        version: version_data.version,
        snapshot: version_data.snapshot,
        changelog: version_data.changelog,
        change_summary: version_data.change_summary,
        published_at: version_data.published_at.to_rfc3339(),
        published_by: version_data.published_by.map(|id| id.to_string()),
    }))
}

/// GET /tier-lists/{slug}/changelog
/// Get recent changes
pub async fn get_changelog(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<ChangelogQuery>,
) -> Result<Json<ChangelogResponse>, ApiError> {
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    let limit = query.limit.unwrap_or(100);

    let changes = TierChangeLog::find_by_tier_list(&state.db, tier_list.id, limit)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Failed to fetch changelog".into())
        })?;

    // Get tier names for the changes
    let tiers =
        crate::database::models::tier_lists::Tier::find_by_tier_list(&state.db, tier_list.id)
            .await
            .unwrap_or_default();

    let tier_map: std::collections::HashMap<uuid::Uuid, String> =
        tiers.into_iter().map(|t| (t.id, t.name)).collect();

    let entries: Vec<ChangeLogEntry> = changes
        .into_iter()
        .map(|c| {
            let operator_name = c
                .operator_id
                .as_ref()
                .and_then(|id| state.game_data.operators.get(id))
                .map(|o| o.name.clone());

            ChangeLogEntry {
                id: c.id.to_string(),
                change_type: c.change_type,
                operator_id: c.operator_id,
                operator_name,
                old_tier_name: c.old_tier_id.and_then(|id| tier_map.get(&id).cloned()),
                new_tier_name: c.new_tier_id.and_then(|id| tier_map.get(&id).cloned()),
                reason: c.reason,
                changed_at: c.changed_at.to_rfc3339(),
            }
        })
        .collect();

    Ok(Json(ChangelogResponse { changes: entries }))
}

/// GET /tier-lists/{slug}/operator/{operator_id}/history
/// Get history for a specific operator
pub async fn get_operator_history(
    State(state): State<AppState>,
    Path((slug, operator_id)): Path<(String, String)>,
) -> Result<Json<ChangelogResponse>, ApiError> {
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    let changes = TierChangeLog::find_by_operator(&state.db, tier_list.id, &operator_id)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Failed to fetch operator history".into())
        })?;

    let tiers =
        crate::database::models::tier_lists::Tier::find_by_tier_list(&state.db, tier_list.id)
            .await
            .unwrap_or_default();

    let tier_map: std::collections::HashMap<uuid::Uuid, String> =
        tiers.into_iter().map(|t| (t.id, t.name)).collect();

    let operator_name = state
        .game_data
        .operators
        .get(&operator_id)
        .map(|o| o.name.clone());

    let entries: Vec<ChangeLogEntry> = changes
        .into_iter()
        .map(|c| ChangeLogEntry {
            id: c.id.to_string(),
            change_type: c.change_type,
            operator_id: c.operator_id,
            operator_name: operator_name.clone(),
            old_tier_name: c.old_tier_id.and_then(|id| tier_map.get(&id).cloned()),
            new_tier_name: c.new_tier_id.and_then(|id| tier_map.get(&id).cloned()),
            reason: c.reason,
            changed_at: c.changed_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(ChangelogResponse { changes: entries }))
}

/// POST /tier-lists/{slug}/publish
/// Publish a new version (requires Publish permission)
pub async fn publish_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Json(body): Json<PublishVersionRequest>,
) -> Result<Json<PublishVersionResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    check_permission(&state, &auth, tier_list.id, Permission::Publish).await?;

    // Build current snapshot
    let snapshot = TierListSnapshot::build(&state.db, tier_list.id)
        .await
        .map_err(|e| {
            eprintln!("Failed to build snapshot: {e:?}");
            ApiError::Internal("Failed to build snapshot".into())
        })?;

    let snapshot_json = serde_json::to_value(&snapshot).map_err(|e| {
        eprintln!("Failed to serialize snapshot: {e:?}");
        ApiError::Internal("Failed to serialize snapshot".into())
    })?;

    let input = PublishVersion {
        changelog: body.changelog,
        change_summary: body.change_summary,
    };

    let version = TierListVersion::create(
        &state.db,
        tier_list.id,
        snapshot_json,
        input,
        Some(auth.user_id),
    )
    .await
    .map_err(|e| {
        eprintln!("Failed to create version: {e:?}");
        ApiError::Internal("Failed to publish version".into())
    })?;

    // Link pending changes to this version
    let last_version =
        TierListVersion::find_by_version(&state.db, tier_list.id, version.version - 1)
            .await
            .ok()
            .flatten();

    let since = last_version
        .map(|v| v.published_at)
        .unwrap_or_else(|| Utc::now() - chrono::Duration::days(365));

    let _ = TierChangeLog::link_to_version(&state.db, tier_list.id, version.id, since).await;

    Ok(Json(PublishVersionResponse {
        success: true,
        version: VersionSummary {
            id: version.id.to_string(),
            version: version.version,
            change_summary: version.change_summary,
            published_at: version.published_at.to_rfc3339(),
            published_by: Some(auth.user_id.to_string()),
        },
    }))
}
