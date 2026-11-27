use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::catalog::{operators, recruitment};
use myrtle_gamedata::model::skin::SkinData;
use myrtle_gamedata_raw::tables::voice::Voices;
use myrtle_types::Server;

use crate::error::ApiResult;
use crate::extract::ServerSel;
use crate::respond::Cached;
use crate::router::ApiRouter;
use crate::server_prefix::PrefixServer;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/operators/index", get(index))
        .route("/operators/ownership", get(ownership))
        .route("/operators/recruitment", get(recruitment))
        .route("/operators/roster", get(roster))
        .route("/operators/cn-changes", get(cn_changes))
        .route("/operators/{id}", get(detail))
        .route("/operators/{id}/build-stats", get(build_stats))
        .route("/upcoming", get(upcoming))
        .route("/voices/{id}", get(voices_detail))
        .route("/skins/{id}", get(skins_detail))
}

async fn index(State(ctx): State<AppContext>, ServerSel(server): ServerSel) -> ApiResult<Cached> {
    Ok(operators::index(&ctx, server).await?.into())
}

async fn ownership(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
) -> ApiResult<Cached> {
    Ok(operators::ownership(&ctx, server).await?.into())
}

async fn recruitment(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
) -> ApiResult<Cached> {
    Ok(Cached::gamedata(
        recruitment::recruitment(&ctx, server).await?,
    ))
}

async fn roster(State(ctx): State<AppContext>, ServerSel(server): ServerSel) -> ApiResult<Cached> {
    Ok(Cached::gamedata(operators::roster(&ctx, server).await?))
}

async fn cn_changes(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
) -> ApiResult<Cached> {
    Ok(operators::cn_changes(&ctx, server).await?.into())
}

async fn detail(
    State(ctx): State<AppContext>,
    prefix: Option<Extension<PrefixServer>>,
    Path(id): Path<String>,
) -> ApiResult<Cached> {
    let json = match prefix {
        Some(Extension(PrefixServer(server))) => operators::detail(&ctx, server, &id).await?,
        None => operators::resolve_detail(&ctx, ctx.snapshots.default_server(), &id).await?,
    };
    Ok(Cached::gamedata(json))
}

async fn build_stats(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(id): Path<String>,
) -> ApiResult<Cached> {
    Ok(operators::build_stats(&ctx, server, &id).await?.into())
}

async fn upcoming(
    State(ctx): State<AppContext>,
    prefix: Option<Extension<PrefixServer>>,
) -> ApiResult<Cached> {
    let viewer = prefix.map_or(Server::CN, |Extension(PrefixServer(server))| server);
    Ok(operators::upcoming(&ctx, viewer).await?.into())
}

async fn voices_detail(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(id): Path<String>,
) -> ApiResult<Json<Voices>> {
    Ok(Json(operators::voices(&ctx, server, &id)?))
}

async fn skins_detail(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(id): Path<String>,
) -> ApiResult<Json<SkinData>> {
    Ok(Json(operators::skins(&ctx, server, &id)?))
}
