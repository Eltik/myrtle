use axum::Json;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use chrono::{DateTime, Utc};
use myrtle_app::AppContext;
use myrtle_app::services::community::operator_notes::{
    self as notes, GlobalAuditLogResponse, OperatorNote, OperatorNoteAuditEntry, UpdateFields,
};
use serde::Deserialize;
use serde_json::Value;

use crate::error::ApiResult;
use crate::extract::{self, Caller};
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/operator-notes", get(list))
        .route("/operator-notes/{operator_id}", get(one).put(update))
        .route("/operator-notes/{operator_id}/audit", get(audit_log))
        .route("/admin/operator-notes/audit", get(global_audit_log))
}

async fn list(State(ctx): State<AppContext>) -> ApiResult<Json<Vec<OperatorNote>>> {
    Ok(Json(notes::list(&ctx).await?))
}

async fn one(
    State(ctx): State<AppContext>,
    Path(operator_id): Path<String>,
) -> ApiResult<Json<OperatorNote>> {
    Ok(Json(notes::get(&ctx, &operator_id).await?))
}

async fn audit_log(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(operator_id): Path<String>,
) -> ApiResult<Json<Vec<OperatorNoteAuditEntry>>> {
    Ok(Json(notes::audit_log(&ctx, &caller, &operator_id).await?))
}

#[derive(Deserialize)]
struct GlobalAuditQuery {
    limit: Option<i64>,
    before: Option<DateTime<Utc>>,
    actor: Option<String>,
}

async fn global_audit_log(
    State(ctx): State<AppContext>,
    caller: Caller,
    Query(query): Query<GlobalAuditQuery>,
) -> ApiResult<Json<GlobalAuditLogResponse>> {
    let GlobalAuditQuery {
        limit,
        before,
        actor,
    } = query;
    let log = notes::global_audit_log(&ctx, &caller, limit, before, actor.as_deref()).await?;
    Ok(Json(log))
}

#[derive(Deserialize)]
struct UpdateNoteRequest {
    pros: Option<String>,
    cons: Option<String>,
    notes: Option<String>,
    trivia: Option<String>,
    summary: Option<String>,
    tags: Option<Value>,
}

async fn update(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(operator_id): Path<String>,
    extract::Json(body): extract::Json<UpdateNoteRequest>,
) -> ApiResult<Json<OperatorNote>> {
    let fields = UpdateFields {
        pros: body.pros,
        cons: body.cons,
        notes: body.notes,
        trivia: body.trivia,
        summary: body.summary,
        tags: body.tags,
    };
    Ok(Json(
        notes::update(&ctx, &caller, &operator_id, fields).await?,
    ))
}
