use axum::Json;
use axum::extract::{Query, State};
use axum::routing::get;
use chrono::{DateTime, Utc};
use myrtle_app::AppContext;
use myrtle_app::audit::ContentAuditFeed;
use myrtle_db::repos::content_audit::ContentAuditFilter;
use myrtle_types::{ContentTargetKind, UserId};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::extract::Caller;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new().route("/admin/content-audit", get(feed))
}

#[derive(Deserialize)]
struct ContentAuditQuery {
    limit: Option<i64>,
    before: Option<DateTime<Utc>>,
    actor: Option<Uuid>,
    target_kind: Option<ContentTargetKind>,
    target_id: Option<Uuid>,
}

async fn feed(
    State(ctx): State<AppContext>,
    caller: Caller,
    Query(query): Query<ContentAuditQuery>,
) -> ApiResult<Json<ContentAuditFeed>> {
    let filter = ContentAuditFilter {
        actor: query.actor.map(UserId),
        target_kind: query.target_kind,
        target_id: query.target_id,
    };
    let limit = query.limit.unwrap_or(100);
    let feed = ctx
        .audit
        .feed(&ctx.policy, &caller, filter, limit, query.before)
        .await?;
    Ok(Json(feed))
}
