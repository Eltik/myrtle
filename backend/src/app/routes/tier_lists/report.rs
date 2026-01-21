use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::models::tier_lists::{CreateReport, ReviewReport, TierListReport};

use super::middleware::{
    can_moderate, get_tier_list_by_slug, require_auth, require_tier_list_admin,
};

#[derive(Deserialize)]
pub struct ReportRequest {
    /// Reason for the report: "inappropriate_content", "spam", "harassment", "other"
    pub reason: String,
    /// Optional description with more details
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct ReportResponse {
    pub success: bool,
    pub report_id: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct ListReportsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Serialize)]
pub struct ReportSummary {
    pub id: String,
    pub tier_list_id: String,
    pub reporter_id: String,
    pub reason: String,
    pub description: Option<String>,
    pub status: String,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<String>,
    pub action_taken: Option<String>,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct ListReportsResponse {
    pub reports: Vec<ReportSummary>,
    pub count: usize,
}

#[derive(Deserialize)]
pub struct ReviewReportRequest {
    /// Status: "reviewed", "dismissed", "actioned"
    pub status: String,
    /// Optional description of action taken
    pub action_taken: Option<String>,
}

#[derive(Serialize)]
pub struct ReviewReportResponse {
    pub success: bool,
    pub report: ReportSummary,
}

/// POST /tier-lists/{slug}/report
/// Report a tier list for inappropriate content (requires authentication)
pub async fn report_tier_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Json(body): Json<ReportRequest>,
) -> Result<Json<ReportResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    // Check if tier list is deleted
    if tier_list.is_deleted {
        return Err(ApiError::NotFound("Tier list not found".into()));
    }

    // Can only report community tier lists
    if !tier_list.is_community() {
        return Err(ApiError::BadRequest(
            "Only community tier lists can be reported".into(),
        ));
    }

    // Cannot report your own tier list
    if tier_list.is_owner(auth.user_id) {
        return Err(ApiError::BadRequest(
            "You cannot report your own tier list".into(),
        ));
    }

    // Validate reason
    let valid_reasons = ["inappropriate_content", "spam", "harassment", "other"];
    if !valid_reasons.contains(&body.reason.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "Invalid reason '{}'. Valid reasons: {}",
            body.reason,
            valid_reasons.join(", ")
        )));
    }

    // Check if user has already reported this tier list
    let already_reported = TierListReport::has_reported(&state.db, tier_list.id, auth.user_id)
        .await
        .map_err(|e| {
            eprintln!("Database error checking existing report: {e:?}");
            ApiError::Internal("Failed to check existing report".into())
        })?;

    if already_reported {
        return Err(ApiError::BadRequest(
            "You have already reported this tier list".into(),
        ));
    }

    // Create the report
    let input = CreateReport {
        reason: body.reason,
        description: body.description,
    };

    let report = TierListReport::create(&state.db, tier_list.id, auth.user_id, input)
        .await
        .map_err(|e| {
            eprintln!("Failed to create report: {e:?}");
            ApiError::Internal("Failed to create report".into())
        })?;

    Ok(Json(ReportResponse {
        success: true,
        report_id: report.id.to_string(),
        message: "Report submitted successfully. It will be reviewed by moderators.".into(),
    }))
}

/// GET /admin/tier-lists/reports
/// List pending reports (requires admin)
pub async fn list_reports(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListReportsQuery>,
) -> Result<Json<ListReportsResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;

    // Require moderator role
    if !can_moderate(&auth) {
        return Err(ApiError::BadRequest(
            "This action requires moderator privileges".into(),
        ));
    }

    let reports = TierListReport::find_pending(&state.db, query.limit)
        .await
        .map_err(|e| {
            eprintln!("Database error fetching reports: {e:?}");
            ApiError::Internal("Failed to fetch reports".into())
        })?;

    let count = reports.len();

    let summaries = reports
        .into_iter()
        .map(|r| ReportSummary {
            id: r.id.to_string(),
            tier_list_id: r.tier_list_id.to_string(),
            reporter_id: r.reporter_id.to_string(),
            reason: r.reason,
            description: r.description,
            status: r.status,
            reviewed_by: r.reviewed_by.map(|id| id.to_string()),
            reviewed_at: r.reviewed_at.map(|dt| dt.to_rfc3339()),
            action_taken: r.action_taken,
            created_at: r.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(ListReportsResponse {
        reports: summaries,
        count,
    }))
}

/// POST /admin/tier-lists/reports/{report_id}/review
/// Review and resolve a report (requires admin)
pub async fn review_report(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(report_id): Path<Uuid>,
    Json(body): Json<ReviewReportRequest>,
) -> Result<Json<ReviewReportResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;
    require_tier_list_admin(&auth)?;

    // Validate status
    let valid_statuses = ["reviewed", "dismissed", "actioned"];
    if !valid_statuses.contains(&body.status.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "Invalid status '{}'. Valid statuses: {}",
            body.status,
            valid_statuses.join(", ")
        )));
    }

    // Check if report exists
    let existing = TierListReport::find_by_id(&state.db, report_id)
        .await
        .map_err(|e| {
            eprintln!("Database error finding report: {e:?}");
            ApiError::Internal("Failed to find report".into())
        })?
        .ok_or_else(|| ApiError::NotFound("Report not found".into()))?;

    if existing.status != "pending" {
        return Err(ApiError::BadRequest(
            "This report has already been reviewed".into(),
        ));
    }

    // Review the report
    let input = ReviewReport {
        status: body.status,
        action_taken: body.action_taken,
    };

    let updated = TierListReport::review(&state.db, report_id, auth.user_id, input)
        .await
        .map_err(|e| {
            eprintln!("Failed to review report: {e:?}");
            ApiError::Internal("Failed to review report".into())
        })?
        .ok_or_else(|| ApiError::NotFound("Report not found".into()))?;

    Ok(Json(ReviewReportResponse {
        success: true,
        report: ReportSummary {
            id: updated.id.to_string(),
            tier_list_id: updated.tier_list_id.to_string(),
            reporter_id: updated.reporter_id.to_string(),
            reason: updated.reason,
            description: updated.description,
            status: updated.status,
            reviewed_by: updated.reviewed_by.map(|id| id.to_string()),
            reviewed_at: updated.reviewed_at.map(|dt| dt.to_rfc3339()),
            action_taken: updated.action_taken,
            created_at: updated.created_at.to_rfc3339(),
        },
    }))
}
