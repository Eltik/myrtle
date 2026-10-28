use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::RoleCheckedUser;
use crate::app::services::content_audit::{ContentAuditResponse, feed};
use crate::app::state::AppState;
use crate::database::models::content_audit::ContentTargetKind;
use crate::database::queries::content_audit::ContentAuditFilter;

#[derive(Deserialize)]
pub struct ContentAuditQuery {
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub before: Option<chrono::DateTime<chrono::Utc>>,
    /// Only this account's actions.
    #[serde(default)]
    pub actor: Option<Uuid>,
    #[serde(default)]
    pub target_kind: Option<ContentTargetKind>,
    /// Only rows about this tier list or grid (its id, not its slug: a
    /// deleted target keeps its rows).
    #[serde(default)]
    pub target_id: Option<Uuid>,
}

/// Who created, edited, published, hid, flaired, shared or deleted which tier
/// list or grid, newest first.
///
/// A run of edits by one account on one target within 15 minutes is one row,
/// its `details` the sum of the run and `event_count` the requests in it.
/// Tier list admin or super-admin.
#[utoipa::path(
    get,
    path = "/admin/content-audit",
    operation_id = "content_audit_log",
    tag = "admin",
    params(
        ("limit" = Option<i64>, Query, description = "Page size. Defaults to 100, clamped to 1..=500."),
        ("before" = Option<String>, Query, description = "RFC 3339 timestamp: only rows last touched strictly earlier, for paging."),
        ("actor" = Option<String>, Query, description = "Account id (UUID): only that account's actions."),
        ("target_kind" = Option<ContentTargetKind>, Query, description = "`tier_list` or `grid`."),
        ("target_id" = Option<String>, Query, description = "Tier list or grid id (UUID): only rows about it.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Audit rows, newest first.", body = ContentAuditResponse),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn content_audit_log(
    State(state): State<AppState>,
    auth: RoleCheckedUser,
    Query(params): Query<ContentAuditQuery>,
) -> Result<Json<ContentAuditResponse>, ApiError> {
    if !auth.role.is_tier_list_admin() {
        return Err(ApiError::Forbidden);
    }
    let filter = ContentAuditFilter {
        actor: params.actor,
        target_kind: params.target_kind,
        target_id: params.target_id,
    };
    let response = feed(&state, filter, params.limit.unwrap_or(100), params.before).await?;
    Ok(Json(response))
}
