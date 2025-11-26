#[tokio::main]
async fn main() {
    backend::app::server::run().await;
}
