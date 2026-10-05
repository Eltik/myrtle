use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app::state::AppState;

pub mod crud;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(crud::list, crud::create))
        .routes(routes!(crud::mine))
        .routes(routes!(crud::get, crud::update, crud::delete))
        .routes(routes!(crud::fork))
}
