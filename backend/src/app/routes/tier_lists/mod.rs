use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::hypergryph::constants::Server;
use crate::database::models::tier_list::TierList;
use crate::database::queries::tier_lists::find_by_slug;
use serde::Deserialize;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

/// Active tier list by slug, or `404`.
pub(crate) async fn load_tier_list(state: &AppState, slug: &str) -> Result<TierList, ApiError> {
    find_by_slug(&state.db, slug)
        .await?
        .ok_or(ApiError::NotFound)
}

/// `?server=`: whose game data names the entities on a list. A tier list is
/// ours and the same on every server; only its tiles' names, icons and
/// filters follow the reader's locale.
#[derive(Deserialize)]
pub struct ServerQuery {
    pub server: Option<Server>,
}

impl ServerQuery {
    pub fn or_default(&self, state: &AppState) -> Server {
        self.server.unwrap_or(state.default_server)
    }
}

pub mod catalogue;
pub mod crud;
pub mod permissions;
pub mod placements;
pub mod stats;
pub mod tiers;
pub mod versions;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(crud::create, crud::list))
        .routes(routes!(crud::list_details))
        .routes(routes!(crud::mine))
        .routes(routes!(crud::favorites))
        .routes(routes!(crud::get, crud::update, crud::delete))
        .routes(routes!(tiers::create))
        .routes(routes!(tiers::update, tiers::delete))
        .routes(routes!(catalogue::get))
        .routes(routes!(placements::add))
        .routes(routes!(placements::remove, placements::update_description))
        .routes(routes!(placements::move_to))
        .routes(routes!(versions::list))
        .routes(routes!(versions::publish))
        .routes(routes!(permissions::list, permissions::grant))
        .routes(routes!(permissions::revoke))
        .routes(routes!(stats::record_view))
        .routes(routes!(stats::get_stats))
        .routes(routes!(stats::get_favorite, stats::toggle_favorite))
        .routes(routes!(stats::set_flair))
        .routes(routes!(stats::set_visibility))
        .routes(routes!(stats::list_flairs, stats::create_flair))
}
