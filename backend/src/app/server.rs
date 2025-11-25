use axum::{Router, response::Json, routing::get};
use serde::Serialize;
use std::net::SocketAddr;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn root() -> &'static str {
    "Myrtle API"
}

fn create_router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
}

pub async fn run() {
    let app = create_router();
    let addr = SocketAddr::from(([0, 0, 0, 0], 3060));

    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
