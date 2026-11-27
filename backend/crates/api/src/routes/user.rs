use axum::Json;
use axum::extract::{Path, Query, State};
use axum::routing::{get, put};
use myrtle_app::AppContext;
use myrtle_app::services::accounts::account::{
    self, AdminUsersPage, AdminUsersQuery, SetRoleRequest, UserCheckin, UserProfile, UserScore,
};
use myrtle_app::services::community::showcase::{self, ShowcaseView};
use myrtle_types::UserId;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::extract::{self, Caller, MaybeCaller, Paging, ServerSel};
use crate::respond::{StatusOk, ok};
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/get-user", get(get_user))
        .route("/get-user-showcase", get(get_user_showcase))
        .route("/get-user-score", get(get_user_score))
        .route("/get-user-checkin", get(get_user_checkin))
        .route("/admin/users", get(list_users))
        .route("/admin/users/{user_id}/role", put(set_user_role))
}

#[derive(Deserialize)]
struct UidQuery {
    uid: String,
}

async fn get_user(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(query): Query<UidQuery>,
) -> ApiResult<Json<UserProfile>> {
    Ok(Json(account::get_user(&ctx, &caller, &query.uid).await?))
}

async fn get_user_showcase(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    ServerSel(server): ServerSel,
    Query(query): Query<UidQuery>,
) -> ApiResult<Json<ShowcaseView>> {
    let view = showcase::user_showcase(&ctx, &caller, &query.uid, Some(server)).await?;
    Ok(Json(view))
}

async fn get_user_score(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(query): Query<UidQuery>,
) -> ApiResult<Json<Option<UserScore>>> {
    Ok(Json(
        account::get_user_score(&ctx, &caller, &query.uid).await?,
    ))
}

async fn get_user_checkin(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(query): Query<UidQuery>,
) -> ApiResult<Json<Option<UserCheckin>>> {
    Ok(Json(
        account::get_user_checkin(&ctx, &caller, &query.uid).await?,
    ))
}

async fn list_users(
    State(ctx): State<AppContext>,
    caller: Caller,
    Paging(page): Paging,
    Query(query): Query<AdminUsersQuery>,
) -> ApiResult<Json<AdminUsersPage>> {
    Ok(Json(
        account::list_users(&ctx, &caller, &query, page).await?,
    ))
}

async fn set_user_role(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(user_id): Path<Uuid>,
    extract::Json(body): extract::Json<SetRoleRequest>,
) -> ApiResult<Json<StatusOk>> {
    account::set_user_role(&ctx, &caller, UserId(user_id), &body.role).await?;
    Ok(ok())
}
