mod app;

#[tokio::main]
async fn main() {
    app::server::run().await;
}
