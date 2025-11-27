#[tokio::main]
async fn main() {
    if let Err(e) = backend::app::server::run().await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}
