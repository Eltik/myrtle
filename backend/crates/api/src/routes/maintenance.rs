use axum::Json;
use axum::extract::State;
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::catalog::maintenance::{self, MaintenanceStatus};

use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new().route("/maintenance", get(status))
}

async fn status(State(ctx): State<AppContext>) -> Json<MaintenanceStatus> {
    Json(maintenance::status(&ctx).await)
}
