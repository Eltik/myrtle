use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::models::tier_list::TierList;
use crate::database::queries::tier_lists::find_by_slug;
use axum::{
    Router,
    routing::{delete, get, patch, post, put},
};

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

pub fn router() -> Router<AppState> {
    Router::new()
        // CRUD
        .route("/tier-lists", post(crud::create))
        .route("/tier-lists", get(crud::list))
        .route("/tier-lists/details", get(crud::list_details))
        .route("/tier-lists/mine", get(crud::mine))
        .route("/tier-lists/favorites", get(crud::favorites))
        .route("/tier-lists/{slug}", get(crud::get))
        .route("/tier-lists/{slug}", put(crud::update))
        .route("/tier-lists/{slug}", delete(crud::delete))
        // Tiers
        .route("/tier-lists/{slug}/tiers", post(tiers::create))
        .route("/tier-lists/{slug}/tiers/{tier_id}", put(tiers::update))
        .route("/tier-lists/{slug}/tiers/{tier_id}", delete(tiers::delete))
        // Placements
        .route("/tier-lists/{slug}/placements", post(placements::add))
        .route(
            "/tier-lists/{slug}/placements/{operator_id}",
            delete(placements::remove),
        )
        .route(
            "/tier-lists/{slug}/placements/{operator_id}",
            patch(placements::update_description),
        )
        .route(
            "/tier-lists/{slug}/placements/{operator_id}/move",
            post(placements::move_to),
        )
        // Versions
        .route("/tier-lists/{slug}/versions", get(versions::list))
        .route("/tier-lists/{slug}/publish", post(versions::publish))
        // Permissions
        .route("/tier-lists/{slug}/permissions", get(permissions::list))
        .route("/tier-lists/{slug}/permissions", post(permissions::grant))
        .route(
            "/tier-lists/{slug}/permissions/{user_id}/{permission}",
            delete(permissions::revoke),
        )
        // Stats & engagement
        .route("/tier-lists/{slug}/view", post(stats::record_view))
        .route("/tier-lists/{slug}/stats", get(stats::get_stats))
        .route("/tier-lists/{slug}/favorite", get(stats::get_favorite))
        .route("/tier-lists/{slug}/favorite", post(stats::toggle_favorite))
        .route("/tier-lists/{slug}/flair", put(stats::set_flair))
        .route("/tier-lists/{slug}/visibility", put(stats::set_visibility))
        // Flair catalog (admin + public read)
        .route("/tier-list-flairs", get(stats::list_flairs))
        .route("/tier-list-flairs", post(stats::create_flair))
}
