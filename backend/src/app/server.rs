use anyhow::Result;
use axum::Router;
use axum::http::HeaderName;
use tower_http::compression::predicate::{DefaultPredicate, NotForContentType, Predicate};
use tower_http::compression::{CompressionLayer, CompressionLevel};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::app::routes::router;
use crate::app::state::AppState;

pub async fn run(state: AppState) -> Result<()> {
    let compression_predicate = DefaultPredicate::new()
        .and(NotForContentType::const_new("audio/"))
        .and(NotForContentType::const_new("video/"));

    // Brotli quality 6, measured on the assets this layer compresses (2026-09-11, the
    // 793,854-byte Kal'tsit skeleton and a 24,828-byte scene JSON, single-shot medians):
    // quality 4 writes 247,158 bytes in 5.7 ms, 6 writes 226,575 in 11.7 ms, 9 writes
    // 225,491 in 50.4 ms, 11 writes 204,530 in 1,892.9 ms; the JSON reads 6,335 / 6,293 /
    // 6,295 / 5,666 bytes at 0.3 / 0.5 / 0.4 / 18.8 ms. Six is the knee: past it the bytes
    // barely move until 11, which costs two seconds of a core per skeleton and, behind the
    // edge cache, that is the first visitor's wait at every PoP each week.
    let compression = CompressionLayer::new()
        .quality(CompressionLevel::Precise(6))
        .compress_when(compression_predicate);
    // Permissive CORS answers every origin with `*`, so no response depends on the
    // request's Origin or preflight headers; the layer's default `Vary` still names all
    // three, which splits the edge cache key per requesting origin (the same skeleton read
    // HIT from one page and MISS from another). An empty `Vary` is the truthful one.
    let cors = CorsLayer::permissive().vary::<[HeaderName; 0]>([]);
    let app = Router::new()
        .nest("/api", router())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(compression)
        .layer(cors);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3060);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!("listening on :{port}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => tracing::info!("received Ctrl+C, shutting down"),
        () = terminate => tracing::info!("received SIGTERM, shutting down"),
    }
}
