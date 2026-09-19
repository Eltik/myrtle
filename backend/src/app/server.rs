use anyhow::Result;
use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderName, header};
use axum::routing::get;
use tower_http::compression::predicate::{DefaultPredicate, NotForContentType, Predicate};
use tower_http::compression::{CompressionLayer, CompressionLevel};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::app::metrics::METRICS;
use crate::app::openapi::ApiDoc;
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

/// The `/api` route tree paired with the `OpenAPI` document describing it.
///
/// Building the tree is what produces the document: both come from the same
/// `#[utoipa::path]` annotations, so `api` describes the routes this process
/// actually serves rather than a file kept in step with them by hand.
///
/// Lives here, called from both [`run`] and `tests/openapi_snapshot_test.rs`,
/// so the document the test pins is the document the server serves. Composing
/// it separately in the test would pin a second, parallel API.
pub fn api_parts() -> (Router<AppState>, utoipa::openapi::OpenApi) {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", router())
        .split_for_parts()
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

    let (api_router, api) = api_parts();

    // Serialized once at startup. `Bytes` clones by refcount, so each request
    // for the spec hands out the same buffer rather than re-rendering it.
    let spec_body = Bytes::from(api.to_json()?);

    // Layer order, outermost first. `.layer` wraps what came before it, so this
    // list reads bottom-up against the builder below:
    //
    //   observe  -> outermost, so it sees every request including preflights,
    //               and the latency it records covers compression.
    //   cors     -> answers preflights.
    //   compress -> encodes the body.
    //   limit    -> innermost, where the matched route is known.
    //
    // The docs and the spec are added AFTER the rate-limit layer and so sit
    // outside it: `.layer` wraps only the routes already on the builder.
    // Reading the documentation should not spend the caller's API budget, and
    // a docs page pulling its own spec must not cost them two requests either.
    let app = api_router
        .route("/metrics", get(metrics_handler))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::rate_limit,
        ))
        .merge(Scalar::with_url("/docs", api))
        .route(
            "/api/openapi.json",
            get(move || {
                let body = spec_body.clone();
                async move { ([(header::CONTENT_TYPE, "application/json")], body) }
            }),
        )
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
        "listening; metrics on /metrics, docs on /docs"
    );
    // The graceful drain waits for open connections; a request parked on a
    // running base search (tens of seconds in a debug build) held the process
    // for its whole duration after Ctrl+C. The drain gets a bounded grace
    // period past the signal, then the process leaves - `main` bounds the
    // runtime's own wait on blocking threads the same way.
    let (signalled_tx, signalled_rx) = tokio::sync::oneshot::channel::<()>();
    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        shutdown_signal().await;
        let _ = signalled_tx.send(());
    });
    tokio::select! {
        result = server => result?,
        () = async {
            let _ = signalled_rx.await;
            tokio::time::sleep(SHUTDOWN_GRACE).await;
        } => tracing::warn!(
            grace_secs = SHUTDOWN_GRACE.as_secs(),
            "shutdown grace period elapsed; abandoning in-flight requests"
        ),
    }
    Ok(())
}

/// How long the graceful drain may run past the shutdown signal.
const SHUTDOWN_GRACE: std::time::Duration = std::time::Duration::from_secs(5);

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
