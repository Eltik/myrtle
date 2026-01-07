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
        // Handbook
        .route("/skins", get(endpoints::skins::get_all_skins))
        .route("/skins/{id}", get(endpoints::skins::get_skin_by_id))
        .route(
            "/skins/char/{char_id}",
            get(endpoints::skins::get_skins_by_char_id),
        )
        // Voices
        .route("/voices", get(endpoints::voices::get_all_voices))
        .route(
            "/voices/char/{char_id}",
            get(endpoints::voices::get_voices_by_char_id),
        )
        .route("/voices/{id}", get(endpoints::voices::get_voice_by_id))
        // Gacha
        .route("/gacha", get(endpoints::gacha::get_all_gacha))
        .route("/gacha/recruitment", get(endpoints::gacha::get_recruitment))
        .route(
            "/gacha/recruitable",
            get(endpoints::gacha::get_recruitable_operators),
        )
        .route(
            "/gacha/calculate",
            get(endpoints::gacha::calculate_recruitment),
        )
        .route(
            "/gacha/calculate/{recruitment}",
            get(endpoints::gacha::calculate_recruitment_by_path),
        )
        .route("/gacha/pools", get(endpoints::gacha::get_gacha_pools))
        .route("/gacha/tags", get(endpoints::gacha::get_gacha_tags))
        // Chibis
        .route("/chibis", get(endpoints::chibis::get_all_chibis))
        .route(
            "/chibis/operators",
            get(endpoints::chibis::get_chibi_operators),
        )
        .route(
            "/chibis/{operator_code}",
            get(endpoints::chibis::get_chibi_by_operator),
        )
        // Zones
        .route("/zones", get(endpoints::zones::get_all_zones))
        .route("/zones/{id}", get(endpoints::zones::get_zone_by_id))
        // Stages
        .route("/stages", get(endpoints::stages::get_all_stages))
        .route("/stages/{id}", get(endpoints::stages::get_stage_by_id))
        .route(
            "/stages/zone/{zone_id}",
            get(endpoints::stages::get_stages_by_zone),
        )
        .layer(CompressionLayer::new())
}
