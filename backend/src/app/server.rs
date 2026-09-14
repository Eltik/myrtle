use anyhow::Result;
use axum::Router;
use axum::extract::State;
use axum::http::HeaderName;
use axum::routing::get;
use tower_http::compression::predicate::{DefaultPredicate, NotForContentType, Predicate};
use tower_http::compression::{CompressionLayer, CompressionLevel};
use tower_http::cors::CorsLayer;

use crate::app::metrics::METRICS;
use crate::app::routes::router;
use crate::app::state::AppState;
use crate::app::{cpu, middleware};

/// Prometheus scrape endpoint.
///
/// Deliberately outside `/api`: it serves the monitoring system, not the
/// public API.
async fn metrics_handler(State(state): State<AppState>) -> String {
    METRICS.render(Some(&state.db))
}

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

    // Layer order, outermost first. `.layer` wraps what came before it, so this
    // list reads bottom-up against the builder below:
    //
    //   observe  -> outermost, so it sees every request including preflights,
    //               and the latency it records covers compression.
    //   cors     -> answers preflights.
    //   compress -> encodes the body.
    //   limit    -> innermost of the four, where the matched route is known.
    let app = Router::new()
        .nest("/api", router())
        .route("/metrics", get(metrics_handler))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::rate_limit,
        ))
        .layer(compression)
        .layer(cors)
        .layer(axum::middleware::from_fn(middleware::observe))
        .with_state(state);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3060);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(
        port,
        cpu_permits = cpu::permits(),
        "listening; metrics on /metrics"
    );
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
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
