use axum::{Router, routing::get};

use crate::app::state::AppState;

pub mod health;
pub mod leaderboard;
pub mod search;
pub mod user;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/get-user", get(user::get_user))
        .route("/leaderboard", get(leaderboard::leaderboard))
        .route("/search", get(search::search))
}
