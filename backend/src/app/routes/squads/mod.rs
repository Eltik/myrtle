use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app::state::AppState;

pub mod analyze;
pub mod crud;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(crud::get_all))
        .routes(routes!(crud::save, crud::delete))
        .routes(routes!(analyze::analyze))
        .routes(routes!(analyze::rules))
}
