// Use jemalloc as the global allocator for better memory management
// jemalloc is much better at returning freed memory to the OS
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() {
    if let Err(e) = backend::app::server::run().await {
        eprintln!("Server error: {e}");
        std::process::exit(1);
    }
}
