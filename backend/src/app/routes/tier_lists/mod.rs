use crate::app::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
};

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
        .route("/tier-lists/mine", get(crud::mine))
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
            "/tier-lists/{slug}/permissions/{user_id}",
            delete(permissions::revoke),
        )
        // Stats & engagement
        .route("/tier-lists/{slug}/view", post(stats::record_view))
        .route("/tier-lists/{slug}/stats", get(stats::get_stats))
        .route("/tier-lists/{slug}/favorite", post(stats::toggle_favorite))
        .route("/tier-lists/{slug}/flair", put(stats::set_flair))
        // Flair catalog (admin + public read)
        .route("/tier-list-flairs", get(stats::list_flairs))
        .route("/tier-list-flairs", post(stats::create_flair))
}
