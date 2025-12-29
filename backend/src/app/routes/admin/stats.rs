use axum::{extract::State, http::HeaderMap, Json};
use serde::Serialize;
use std::collections::HashMap;

use crate::app::error::ApiError;
use crate::app::routes::tier_lists::middleware::{extract_auth_context, require_tier_list_admin};
use crate::app::state::AppState;
use crate::database::models::tier_lists::{Tier, TierChangeLog, TierList, TierPlacement};
use crate::database::models::user::User;

#[derive(Serialize)]
pub struct AdminStatsResponse {
    pub users: UserStats,
    #[serde(rename = "tierLists")]
    pub tier_lists: TierListStats,
    #[serde(rename = "recentActivity")]
    pub recent_activity: Vec<RecentActivity>,
}

#[derive(Serialize)]
pub struct UserStats {
    pub total: i64,
    #[serde(rename = "byRole")]
    pub by_role: RoleCounts,
    #[serde(rename = "byServer")]
    pub by_server: HashMap<String, i64>,
    #[serde(rename = "recentUsers")]
    pub recent_users: Vec<RecentUser>,
}

#[derive(Serialize)]
pub struct RoleCounts {
    pub user: i64,
    pub tier_list_editor: i64,
    pub tier_list_admin: i64,
    pub super_admin: i64,
}

#[derive(Serialize)]
pub struct RecentUser {
    pub id: String,
    pub uid: String,
    pub server: String,
    pub nickname: String,
    pub level: i64,
    pub role: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Serialize)]
pub struct TierListStats {
    pub total: i64,
    pub active: i64,
    #[serde(rename = "totalVersions")]
    pub total_versions: i64,
    #[serde(rename = "totalPlacements")]
    pub total_placements: i64,
    #[serde(rename = "tierLists")]
    pub tier_lists: Vec<TierListSummary>,
}

#[derive(Serialize)]
pub struct TierListSummary {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "tierCount")]
    pub tier_count: i64,
    #[serde(rename = "operatorCount")]
    pub operator_count: i64,
    #[serde(rename = "versionCount")]
    pub version_count: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct RecentActivity {
    pub id: String,
    #[serde(rename = "tierListId")]
    pub tier_list_id: String,
    #[serde(rename = "tierListName")]
    pub tier_list_name: String,
    #[serde(rename = "changeType")]
    pub change_type: String,
    #[serde(rename = "operatorId")]
    pub operator_id: Option<String>,
    #[serde(rename = "operatorName")]
    pub operator_name: Option<String>,
    #[serde(rename = "changedBy")]
    pub changed_by: Option<String>,
    #[serde(rename = "changedByNickname")]
    pub changed_by_nickname: Option<String>,
    #[serde(rename = "changedAt")]
    pub changed_at: String,
    pub reason: Option<String>,
}

/// GET /admin/stats
/// Get admin dashboard statistics (requires tier_list_admin or super_admin role)
pub async fn get_stats(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AdminStatsResponse>, ApiError> {
    // Authenticate and authorize
    let auth = extract_auth_context(&headers, &state.jwt_secret)?;
    require_tier_list_admin(&auth)?;

    // Fetch user statistics
    let user_stats = fetch_user_stats(&state).await?;

    // Fetch tier list statistics
    let tier_list_stats = fetch_tier_list_stats(&state).await?;

    // Fetch recent activity
    let recent_activity = fetch_recent_activity(&state).await?;

    Ok(Json(AdminStatsResponse {
        users: user_stats,
        tier_lists: tier_list_stats,
        recent_activity,
    }))
}

async fn fetch_user_stats(state: &AppState) -> Result<UserStats, ApiError> {
    // Get total user count
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Failed to fetch user count".into())
        })?;

    // Get counts by role
    let role_counts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT role, COUNT(*) as count FROM users GROUP BY role",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {e:?}");
        ApiError::Internal("Failed to fetch role counts".into())
    })?;

    let mut by_role = RoleCounts {
        user: 0,
        tier_list_editor: 0,
        tier_list_admin: 0,
        super_admin: 0,
    };

    for (role, count) in role_counts {
        match role.as_str() {
            "user" => by_role.user = count,
            "tier_list_editor" => by_role.tier_list_editor = count,
            "tier_list_admin" => by_role.tier_list_admin = count,
            "super_admin" => by_role.super_admin = count,
            _ => by_role.user += count, // Unknown roles count as user
        }
    }

    // Get counts by server
    let server_counts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT server, COUNT(*) as count FROM users GROUP BY server",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {e:?}");
        ApiError::Internal("Failed to fetch server counts".into())
    })?;

    let by_server: HashMap<String, i64> = server_counts.into_iter().collect();

    // Get recent users (last 10)
    let recent_users_raw = User::find_all(&state.db, 10, 0).await.map_err(|e| {
        eprintln!("Database error: {e:?}");
        ApiError::Internal("Failed to fetch recent users".into())
    })?;

    let recent_users: Vec<RecentUser> = recent_users_raw
        .into_iter()
        .map(|user| {
            let nickname = user
                .data
                .get("status")
                .and_then(|s| s.get("nickName"))
                .and_then(|n| n.as_str())
                .unwrap_or("Unknown")
                .to_string();

            let level = user
                .data
                .get("status")
                .and_then(|s| s.get("level"))
                .and_then(|l| l.as_i64())
                .unwrap_or(0);

            RecentUser {
                id: user.id.to_string(),
                uid: user.uid,
                server: user.server,
                nickname,
                level,
                role: user.role,
                created_at: user.created_at.to_rfc3339(),
            }
        })
        .collect();

    Ok(UserStats {
        total,
        by_role,
        by_server,
        recent_users,
    })
}

async fn fetch_tier_list_stats(state: &AppState) -> Result<TierListStats, ApiError> {
    // Get total tier list count
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tier_lists")
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Failed to fetch tier list count".into())
        })?;

    // Get active tier list count
    let active: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tier_lists WHERE is_active = true")
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Failed to fetch active tier list count".into())
        })?;

    // Get total version count
    let total_versions: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tier_list_versions")
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Failed to fetch version count".into())
        })?;

    // Get total placement count
    let total_placements: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tier_placements")
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Failed to fetch placement count".into())
        })?;

    // Get tier list summaries
    let tier_lists_raw = TierList::find_all(&state.db, 50, 0).await.map_err(|e| {
        eprintln!("Database error: {e:?}");
        ApiError::Internal("Failed to fetch tier lists".into())
    })?;

    let mut tier_lists = Vec::new();
    for tl in tier_lists_raw {
        // Count tiers
        let tiers = Tier::find_by_tier_list(&state.db, tl.id).await.unwrap_or_default();
        let tier_count = tiers.len() as i64;

        // Count operators
        let placements = TierPlacement::find_by_tier_list(&state.db, tl.id)
            .await
            .unwrap_or_default();
        let operator_count = placements.len() as i64;

        // Count versions
        let version_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM tier_list_versions WHERE tier_list_id = $1",
        )
        .bind(tl.id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

        tier_lists.push(TierListSummary {
            id: tl.id.to_string(),
            name: tl.name,
            slug: tl.slug,
            is_active: tl.is_active,
            tier_count,
            operator_count,
            version_count,
            created_at: tl.created_at.to_rfc3339(),
            updated_at: tl.updated_at.to_rfc3339(),
        });
    }

    Ok(TierListStats {
        total,
        active,
        total_versions,
        total_placements,
        tier_lists,
    })
}

async fn fetch_recent_activity(state: &AppState) -> Result<Vec<RecentActivity>, ApiError> {
    // Get recent changes across all tier lists
    let changes: Vec<TierChangeLog> = sqlx::query_as(
        "SELECT * FROM tier_change_log ORDER BY changed_at DESC LIMIT 50",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {e:?}");
        ApiError::Internal("Failed to fetch recent activity".into())
    })?;

    let mut activities = Vec::new();

    for change in changes {
        // Get tier list name
        let tier_list_name = TierList::find_by_id(&state.db, change.tier_list_id)
            .await
            .ok()
            .flatten()
            .map(|tl| tl.name)
            .unwrap_or_else(|| "Unknown".to_string());

        // Get operator name from game data
        let operator_name = change.operator_id.as_ref().and_then(|op_id| {
            state
                .game_data
                .operators
                .get(op_id)
                .map(|op| op.name.clone())
        });

        // Get user nickname who made the change
        let changed_by_nickname = if let Some(user_id) = change.changed_by {
            User::find_by_id(&state.db, user_id)
                .await
                .ok()
                .flatten()
                .and_then(|user| {
                    user.data
                        .get("status")
                        .and_then(|s| s.get("nickName"))
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string())
                })
        } else {
            None
        };

        activities.push(RecentActivity {
            id: change.id.to_string(),
            tier_list_id: change.tier_list_id.to_string(),
            tier_list_name,
            change_type: change.change_type,
            operator_id: change.operator_id,
            operator_name,
            changed_by: change.changed_by.map(|id| id.to_string()),
            changed_by_nickname,
            changed_at: change.changed_at.to_rfc3339(),
            reason: change.reason,
        });
    }

    Ok(activities)
}
