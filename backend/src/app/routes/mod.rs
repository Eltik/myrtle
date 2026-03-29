use axum::{Router, routing::get};

use crate::app::state::AppState;

pub mod health;

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health::health))
}
