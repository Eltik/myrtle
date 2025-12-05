use axum::{Router, routing::get};
use tower_http::compression::CompressionLayer;

use crate::app::state::AppState;

mod compression;
mod endpoints;
mod fields;
mod handler;
mod pagination;

pub fn router() -> Router<AppState> {
    Router::new()
        // Operators
        .route("/operators", get(endpoints::operators::get_all_operators))
        .route(
            "/operators/{id}",
            get(endpoints::operators::get_operator_by_id),
        )
        // Materials
        .route("/materials", get(endpoints::materials::get_all_materials))
        .route(
            "/materials/{id}",
            get(endpoints::materials::get_material_by_id),
        )
        // Modules
        .route("/modules", get(endpoints::modules::get_all_modules))
        .route(
            "/modules/details/{id}",
            get(endpoints::modules::get_module_details),
        )
        .route("/modules/{id}", get(endpoints::modules::get_module_by_id))
        // Ranges
        .route("/ranges", get(endpoints::ranges::get_all_ranges))
        .route("/ranges/{id}", get(endpoints::ranges::get_range_by_id))
        // Skills
        .route("/skills", get(endpoints::skills::get_all_skills))
        .route("/skills/{id}", get(endpoints::skills::get_skill_by_id))
        // Trust
        .route("/trust", get(endpoints::trust::get_favor_table))
        .route(
            "/trust/calculate",
            get(endpoints::trust::calculate_trust_level),
        )
        .route(
            "/trust/calculate/{trust}",
            get(endpoints::trust::calculate_trust_by_path),
        )
        // Handbook
        .route("/handbook", get(endpoints::handbook::get_all_handbook))
        .route(
            "/handbook/{id}",
            get(endpoints::handbook::get_handbook_by_id),
        )
        // ... other endpoints
        // Compression fallback (if not cached as gzip)
        .layer(CompressionLayer::new())
}
