use axum::Json;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::catalog::static_data::{self, EncounteredEnemiesResponse};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extract::{MaybeCaller, ServerSel};
use crate::respond::Cached;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/enemies/{id}", get(enemy_detail))
        .route("/enemies/{id}/stages", get(enemy_stages))
        .route("/encountered-enemies", get(encountered_enemies))
        .route(
            "/encountered-enemies/community-average",
            get(community_average),
        )
}

async fn enemy_detail(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(id): Path<String>,
) -> ApiResult<Cached> {
    Ok(Cached::gamedata(
        static_data::enemy_detail(&ctx, server, &id).await?,
    ))
}

async fn enemy_stages(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(id): Path<String>,
) -> ApiResult<Cached> {
    Ok(Cached::gamedata(
        static_data::enemy_stages(&ctx, server, &id).await?,
    ))
}

#[derive(Deserialize)]
struct UidQuery {
    uid: Option<String>,
}

async fn encountered_enemies(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(query): Query<UidQuery>,
) -> ApiResult<Json<EncounteredEnemiesResponse>> {
    let response = static_data::encountered_enemies(&ctx, &caller, query.uid.as_deref()).await?;
    Ok(Json(response))
}

async fn community_average(State(ctx): State<AppContext>) -> ApiResult<Cached> {
    Ok(static_data::community_enemy_average(&ctx).await?.into())
}
