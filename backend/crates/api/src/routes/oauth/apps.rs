use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use myrtle_app::AppContext;
use myrtle_app::services::oauth::apps;
use myrtle_app::services::oauth::dto::{OAuthApp, OAuthAppInput, OAuthAppWithSecret};
use uuid::Uuid;

use crate::error::ApiResult;
use crate::extract::{self, Caller};
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/oauth/apps", get(list).post(create))
        .route("/oauth/apps/{id}", put(update).delete(remove))
        .route("/oauth/apps/{id}/secret", post(rotate_secret))
        .route("/admin/oauth/apps/{id}", delete(take_down))
}

async fn list(State(ctx): State<AppContext>, caller: Caller) -> ApiResult<Json<Vec<OAuthApp>>> {
    Ok(Json(apps::list(&ctx, &caller).await?))
}

async fn create(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<OAuthAppInput>,
) -> ApiResult<(StatusCode, Json<OAuthAppWithSecret>)> {
    let app = apps::create(&ctx, &caller, body).await?;
    Ok((StatusCode::CREATED, Json(app)))
}

async fn update(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(id): Path<Uuid>,
    extract::Json(body): extract::Json<OAuthAppInput>,
) -> ApiResult<Json<OAuthApp>> {
    Ok(Json(apps::update(&ctx, &caller, id, body).await?))
}

async fn remove(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    apps::delete(&ctx, &caller, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn rotate_secret(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<OAuthAppWithSecret>> {
    Ok(Json(apps::rotate_secret(&ctx, &caller, id).await?))
}

async fn take_down(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    apps::take_down(&ctx, &caller, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
