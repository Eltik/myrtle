use axum::{Json, extract::State, http::HeaderMap};
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::routes::tier_lists::middleware::{extract_auth_context, require_tier_list_admin};
use crate::app::state::AppState;

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
    #[serde(rename = "tierListType")]
    pub tier_list_type: String,
    #[serde(rename = "createdBy")]
    pub created_by: Option<String>,
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

    // Run all three fetches concurrently
    let (user_stats, tier_list_stats, recent_activity) = tokio::try_join!(
        fetch_user_stats(&state),
        fetch_tier_list_stats(&state),
        fetch_recent_activity(&state),
    )?;

    Ok(Json(AdminStatsResponse {
        users: user_stats,
        tier_lists: tier_list_stats,
        recent_activity,
    }))
}

/// Row type for the lightweight user query (only the columns we need)
#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    uid: String,
    server: String,
    role: String,
    data: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
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
    let role_counts: Vec<(String, i64)> =
        sqlx::query_as("SELECT role, COUNT(*) as count FROM users GROUP BY role")
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
            _ => by_role.user += count,
        }
    }

    // Get counts by server
    let server_counts: Vec<(String, i64)> =
        sqlx::query_as("SELECT server, COUNT(*) as count FROM users GROUP BY server")
            .fetch_all(&state.db)
            .await
            .map_err(|e| {
                eprintln!("Database error: {e:?}");
                ApiError::Internal("Failed to fetch server counts".into())
            })?;

    let by_server: HashMap<String, i64> = server_counts.into_iter().collect();

    // Fetch only the columns needed for the admin table (skip score which can be huge)
    let recent_users_raw: Vec<UserRow> = sqlx::query_as(
        r#"
        SELECT id, uid, server, role, data->'status' as data, created_at
        FROM users
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {e:?}");
        ApiError::Internal("Failed to fetch users".into())
    })?;

    let recent_users: Vec<RecentUser> = recent_users_raw
        .into_iter()
        .map(|user| {
            let nickname = user
                .data
                .get("nickName")
                .and_then(|n| n.as_str())
                .unwrap_or("Unknown")
                .to_string();

            let level = user.data.get("level").and_then(|l| l.as_i64()).unwrap_or(0);

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

/// Row type for tier list summary with counts computed in a single query
#[derive(sqlx::FromRow)]
struct TierListSummaryRow {
    id: Uuid,
    name: String,
    slug: String,
    is_active: bool,
    tier_list_type: String,
    created_by: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    tier_count: Option<i64>,
    operator_count: Option<i64>,
    version_count: Option<i64>,
}

async fn fetch_tier_list_stats(state: &AppState) -> Result<TierListStats, ApiError> {
    // Get aggregate counts in parallel
    let (total, active, total_versions, total_placements) = tokio::try_join!(
        async {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tier_lists")
                .fetch_one(&state.db)
                .await
                .map_err(|e| {
                    eprintln!("Database error: {e:?}");
                    ApiError::Internal("Failed to fetch tier list count".into())
                })
        },
        async {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tier_lists WHERE is_active = true")
                .fetch_one(&state.db)
                .await
                .map_err(|e| {
                    eprintln!("Database error: {e:?}");
                    ApiError::Internal("Failed to fetch active tier list count".into())
                })
        },
        async {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tier_list_versions")
                .fetch_one(&state.db)
                .await
                .map_err(|e| {
                    eprintln!("Database error: {e:?}");
                    ApiError::Internal("Failed to fetch version count".into())
                })
        },
        async {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tier_placements")
                .fetch_one(&state.db)
                .await
                .map_err(|e| {
                    eprintln!("Database error: {e:?}");
                    ApiError::Internal("Failed to fetch placement count".into())
                })
        },
    )?;

    // Get tier list summaries with counts in a single query (no N+1)
    let rows: Vec<TierListSummaryRow> = sqlx::query_as(
        r#"
        SELECT
            tl.id, tl.name, tl.slug, tl.is_active, tl.tier_list_type, tl.created_by, tl.created_at, tl.updated_at,
            (SELECT COUNT(*) FROM tiers t WHERE t.tier_list_id = tl.id) as tier_count,
            (SELECT COUNT(*) FROM tier_placements tp
                JOIN tiers t2 ON tp.tier_id = t2.id
                WHERE t2.tier_list_id = tl.id) as operator_count,
            (SELECT COUNT(*) FROM tier_list_versions v WHERE v.tier_list_id = tl.id) as version_count
        FROM tier_lists tl
        ORDER BY tl.created_at DESC
        LIMIT 50
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {e:?}");
        ApiError::Internal("Failed to fetch tier lists".into())
    })?;

    let tier_lists: Vec<TierListSummary> = rows
        .into_iter()
        .map(|row| TierListSummary {
            id: row.id.to_string(),
            name: row.name,
            slug: row.slug,
            is_active: row.is_active,
            tier_list_type: row.tier_list_type,
            created_by: row.created_by.map(|id| id.to_string()),
            tier_count: row.tier_count.unwrap_or(0),
            operator_count: row.operator_count.unwrap_or(0),
            version_count: row.version_count.unwrap_or(0),
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(TierListStats {
        total,
        active,
        total_versions,
        total_placements,
        tier_lists,
    })
}

/// Row type for recent activity joined with tier list names and user nicknames
#[derive(sqlx::FromRow)]
struct ActivityRow {
    id: Uuid,
    tier_list_id: Uuid,
    tier_list_name: Option<String>,
    change_type: String,
    operator_id: Option<String>,
    changed_by: Option<Uuid>,
    changed_by_nickname: Option<String>,
    changed_at: chrono::DateTime<chrono::Utc>,
    reason: Option<String>,
}

async fn fetch_recent_activity(state: &AppState) -> Result<Vec<RecentActivity>, ApiError> {
    // Single query with JOINs instead of N+1 lookups
    let rows: Vec<ActivityRow> = sqlx::query_as(
        r#"
        SELECT
            cl.id,
            cl.tier_list_id,
            tl.name as tier_list_name,
            cl.change_type,
            cl.operator_id,
            cl.changed_by,
            u.data->'status'->>'nickName' as changed_by_nickname,
            cl.changed_at,
            cl.reason
        FROM tier_change_log cl
        LEFT JOIN tier_lists tl ON tl.id = cl.tier_list_id
        LEFT JOIN users u ON u.id = cl.changed_by
        ORDER BY cl.changed_at DESC
        LIMIT 50
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error: {e:?}");
        ApiError::Internal("Failed to fetch recent activity".into())
    })?;

    let activities: Vec<RecentActivity> = rows
        .into_iter()
        .map(|row| {
            let operator_name = row.operator_id.as_ref().and_then(|op_id| {
                state
                    .game_data
                    .operators
                    .get(op_id)
                    .map(|op| op.name.clone())
            });

            RecentActivity {
                id: row.id.to_string(),
                tier_list_id: row.tier_list_id.to_string(),
                tier_list_name: row.tier_list_name.unwrap_or_else(|| "Unknown".to_string()),
                change_type: row.change_type,
                operator_id: row.operator_id,
                operator_name,
                changed_by: row.changed_by.map(|id| id.to_string()),
                changed_by_nickname: row.changed_by_nickname,
                changed_at: row.changed_at.to_rfc3339(),
                reason: row.reason,
            }
        })
        .collect();

    Ok(activities)
}
