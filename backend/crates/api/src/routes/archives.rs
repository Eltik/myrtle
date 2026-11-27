use axum::extract::{Path, State};
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::catalog::archives::{self, ArchiveList};

use crate::error::ApiResult;
use crate::extract::ServerSel;
use crate::respond::Cached;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/archives/network", get(network))
        .route("/archives/operators", get(dossiers))
        .route("/archives/demographics", get(demographics))
        .route("/archives/hidden-factions", get(hidden_factions))
        .route("/archives/operator/{char_id}", get(operator))
}

async fn list(ctx: &AppContext, server: ServerSel, list: ArchiveList) -> ApiResult<Cached> {
    Ok(Cached::gamedata(archives::list(ctx, server.0, list).await?))
}

async fn network(State(ctx): State<AppContext>, server: ServerSel) -> ApiResult<Cached> {
    list(&ctx, server, ArchiveList::Network).await
}

async fn dossiers(State(ctx): State<AppContext>, server: ServerSel) -> ApiResult<Cached> {
    list(&ctx, server, ArchiveList::Dossiers).await
}

async fn demographics(State(ctx): State<AppContext>, server: ServerSel) -> ApiResult<Cached> {
    list(&ctx, server, ArchiveList::Demographics).await
}

async fn hidden_factions(State(ctx): State<AppContext>, server: ServerSel) -> ApiResult<Cached> {
    list(&ctx, server, ArchiveList::HiddenFactions).await
}

async fn operator(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(char_id): Path<String>,
) -> ApiResult<Cached> {
    Ok(Cached::new(
        archives::operator(&ctx, server, &char_id).await?,
    ))
}
