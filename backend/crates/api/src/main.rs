use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use tracing_subscriber::EnvFilter;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    let config = std::env::args().nth(1).map(PathBuf::from);
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(e) => {
            tracing::error!(error = %e, "runtime did not start");
            return ExitCode::FAILURE;
        }
    };
    let served = runtime.block_on(myrtle_api::run(config.as_deref()));
    runtime.shutdown_timeout(Duration::from_secs(2));
    match served {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error = %e, "server stopped");
            ExitCode::FAILURE
        }
    }
}
