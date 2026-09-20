use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::models::tier_list::TierList;
use crate::database::queries::tier_lists::find_by_slug;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

/// Load an active tier list by slug or return `404`. Shared by the tier-list
/// route handlers, which all resolve the `{slug}` path param the same way.
pub(crate) async fn load_tier_list(state: &AppState, slug: &str) -> Result<TierList, ApiError> {
    find_by_slug(&state.db, slug)
        .await?
        .ok_or(ApiError::NotFound)
}

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
        // Flair catalog (admin + public read)
        .routes(routes!(stats::list_flairs, stats::create_flair))
}
