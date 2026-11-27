use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get};
use myrtle_app::AppContext;
use myrtle_app::services::oauth::consent;
use myrtle_app::services::oauth::dto::{
    AuthorizeCheck, AuthorizeDecision, AuthorizeDecisionInput, AuthorizeParams, OAuthAuthorization,
};
use uuid::Uuid;

use crate::error::ApiResult;
use crate::extract::{self, Caller};
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/oauth/authorize", get(check).post(decide))
        .route("/oauth/authorizations", get(authorizations))
        .route("/oauth/authorizations/{app_id}", delete(withdraw))
}

async fn check(
    State(ctx): State<AppContext>,
    caller: Caller,
    Query(params): Query<AuthorizeParams>,
) -> ApiResult<Json<AuthorizeCheck>> {
    Ok(Json(consent::check(&ctx, &caller, &params).await?))
}

async fn decide(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<AuthorizeDecisionInput>,
) -> ApiResult<Json<AuthorizeDecision>> {
    Ok(Json(consent::decide(&ctx, &caller, body).await?))
}

async fn authorizations(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<Vec<OAuthAuthorization>>> {
    Ok(Json(consent::authorizations(&ctx, &caller).await?))
}

async fn withdraw(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(app_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    consent::withdraw(&ctx, &caller, app_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
