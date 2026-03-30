use axum::{
    Router,
    routing::{get, post},
};

use crate::app::state::AppState;

pub mod assets;
pub mod auth;
pub mod gacha;
pub mod health;
pub mod leaderboard;
pub mod roster;
pub mod search;
pub mod static_data;
pub mod user;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/get-user", get(user::get_user))
        .route("/leaderboard", get(leaderboard::leaderboard))
        .route("/search", get(search::search))
        .route("/static/{resource}", get(static_data::get_static))
        .route("/avatar/{id}", get(assets::avatar))
        .route("/portrait/{id}", get(assets::portrait))
        .route("/login/send-code", post(auth::send_code))
        .route("/login", post(auth::login))
        .route("/auth/verify", get(auth::verify))
        .route("/auth/update-settings", post(auth::update_settings))
        .route("/gacha/history", get(gacha::history))
        .route("/gacha/stats", get(gacha::stats))
        .route("/refresh", post(auth::refresh))
        .route("/roster", get(roster::get_roster))
        .route("/roster/{operator_id}", get(roster::get_operator))
        .route("/gacha/fetch", post(gacha::fetch))
        .route("/gacha/global-stats", get(gacha::global_stats))
}
