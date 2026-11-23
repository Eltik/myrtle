use axum::Router;
use axum::extract::Request;
use axum::http::HeaderName;
use axum::middleware::{from_fn, from_fn_with_state};
use axum::routing::get;
use myrtle_app::AppContext;
use tower::ServiceExt;
use tower_http::compression::predicate::{DefaultPredicate, NotForContentType, Predicate};
use tower_http::compression::{CompressionLayer, CompressionLevel};
use tower_http::cors::CorsLayer;

use crate::middleware::{RateLimiter, conditional, observe, rate_limit};
use crate::server_prefix::strip_server;

pub type ApiRouter = Router<AppContext>;

pub fn build(ctx: AppContext) -> Router {
    let compression = CompressionLayer::new()
        .quality(CompressionLevel::Precise(6)) // brotli 6 is the knee: 9 saves 0.5% for 4x the time
        .compress_when(
            DefaultPredicate::new()
                .and(NotForContentType::const_new("audio/"))
                .and(NotForContentType::const_new("video/")),
        );
    let cors = CorsLayer::permissive().vary::<[HeaderName; 0]>([]); // `*` never depends on Origin, and a Vary would split the edge cache
    let limiter = RateLimiter::new(
        ctx.config.rate_limit.rpm,
        ctx.config.server.trusted_proxy_hops,
        &ctx.config.rate_limit.exempt_ips,
    );

    let api = Router::new()
        .route("/health", get(crate::health::health))
        .merge(crate::routes::routes());
    let routed = Router::new()
        .nest("/api", api)
        .layer(from_fn_with_state((ctx.clone(), limiter), rate_limit))
        .layer(from_fn(conditional))
        .layer(compression)
        .layer(cors)
        .layer(from_fn(observe))
        .with_state(ctx);

    let rewritten = move |req: Request| routed.clone().oneshot(strip_server(req)); // inside `Router::layer` it would run after routing
    Router::new().fallback(rewritten)
}
