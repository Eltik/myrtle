use axum::{
    Router,
    routing::{get, post, put},
};

use crate::app::state::AppState;

pub mod assets;
pub mod auth;
pub mod dps;
pub mod gacha;
pub mod health;
pub mod leaderboard;
pub mod operator_notes;
pub mod roster;
pub mod search;
pub mod static_data;
pub mod stats;
pub mod tier_lists;
pub mod user;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/get-user", get(user::get_user))
        .route("/get-user-score", get(user::get_user_score))
        .route("/leaderboard", get(leaderboard::leaderboard))
        .route("/search", get(search::search))
        .route("/static/{resource}", get(static_data::get_static))
        .route("/avatar/{id}", get(assets::avatar))
        .route("/portrait/{id}", get(assets::portrait))
        .route("/skill-icon/{id}", get(assets::skill_icon))
        .route("/module-icon/{id}", get(assets::module_icon))
        .route("/module-big/{id}", get(assets::module_big))
        .route("/enemy-icon/{id}", get(assets::enemy_icon))
        .route("/item-icon/{id}", get(assets::item_icon))
        .route("/charart/{id}", get(assets::charart))
        .route("/skin-portrait/{id}", get(assets::skin_portrait))
        .route("/assets/{*path}", get(assets::generic))
        .route("/login/send-code", post(auth::send_code))
        .route("/login", post(auth::login))
        .route("/auth/verify", get(auth::verify))
        .route("/auth/update-settings", post(auth::update_settings))
        .route("/gacha/history", get(gacha::history))
        .route("/gacha/history/{char_id}", get(gacha::history_by_char))
        .route("/gacha/stored-records", get(gacha::stored_records))
        .route("/gacha/stats", get(gacha::stats))
        .route(
            "/gacha/settings",
            get(gacha::get_settings).post(gacha::update_settings),
        )
        .route("/refresh", post(auth::refresh))
        .route("/roster", get(roster::get_roster))
        .route("/roster/{operator_id}", get(roster::get_operator))
        .route("/gacha/fetch", post(gacha::fetch))
        .route("/gacha/global-stats", get(gacha::global_stats))
        .route("/stats", get(stats::stats))
        .route("/admin/stats", get(stats::admin_stats))
        .route("/dps/operators", get(dps::operators))
        .route("/dps/calculate", post(dps::calculate))
        .route("/operator-notes", get(operator_notes::list))
        .route("/operator-notes/{operator_id}", get(operator_notes::get))
        .route("/operator-notes/{operator_id}", put(operator_notes::update))
        .route(
            "/operator-notes/{operator_id}/audit",
            get(operator_notes::audit_log),
        )
        .merge(tier_lists::router())
}
