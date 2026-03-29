use anyhow::Result;
use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::app::{routes, state::AppState};

pub async fn run(state: AppState) -> Result<()> {
    let app = Router::new()
        .nest("/api", routes::router())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3060").await?;
    tracing::info!("listening on :3060");
    axum::serve(listener, app).await?;
    Ok(())
}
